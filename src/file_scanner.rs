use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
    collections::HashMap,
};
use crossbeam_channel::{bounded, Sender, Receiver};
use notify::{Watcher, RecursiveMode, Event};
use crate::file_utils::{collect_files, find_duplicates};

pub enum ScannerMessage {
    Progress(f32),
    Found(HashMap<Vec<u8>, Vec<PathBuf>>),
    Error(String),
}

pub struct Scanner {
    tx: Sender<ScannerMessage>,
    rx: Receiver<ScannerMessage>,
    total_files: Arc<AtomicUsize>,
    processed_files: Arc<AtomicUsize>,
}

impl Scanner {
    pub fn new() -> Self {
        let (tx, rx) = bounded(100);
        Self {
            tx,
            rx,
            total_files: Arc::new(AtomicUsize::new(0)),
            processed_files: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn receiver(&self) -> Receiver<ScannerMessage> {
        self.rx.clone()
    }

    pub fn start_scan(&self, path: &Path, filters: Vec<String>) {
        let tx = self.tx.clone();
        let path = path.to_path_buf();
        let total_files = self.total_files.clone();
        let processed_files = self.processed_files.clone();

        std::thread::spawn(move || {
            // Collect all files
            let mut files = collect_files(&path, &filters);
            total_files.store(files.len(), Ordering::SeqCst);
            
            // Set up progress tracking
            let progress_tx = tx.clone();
            let progress_processed = processed_files.clone();
            let progress_total = total_files.clone();
            
            std::thread::spawn(move || {
                loop {
                    let processed = progress_processed.load(Ordering::SeqCst);
                    let total = progress_total.load(Ordering::SeqCst);
                    
                    if total == 0 {
                        return;
                    }
                    
                    let progress = processed as f32 / total as f32;
                    if progress_tx.send(ScannerMessage::Progress(progress)).is_err() {
                        return;
                    }
                    
                    if processed == total {
                        return;
                    }
                    
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            // Find duplicates
            match find_duplicates(&mut files) {
                dups => {
                    let _ = tx.send(ScannerMessage::Found(dups));
                }
            }
        });
    }

    pub fn watch_directory(&self, path: &Path) -> notify::Result<impl Watcher> {
        let tx = self.tx.clone();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(_) => {
                    let _ = tx.send(ScannerMessage::Progress(-1.0)); // Trigger a rescan
                }
                Err(e) => {
                    let _ = tx.send(ScannerMessage::Error(e.to_string()));
                }
            }
        })?;

        watcher.watch(path, RecursiveMode::Recursive)?;
        Ok(watcher)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_scanner() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        
        let content = b"test content";
        fs::write(&file1_path, content).unwrap();
        fs::write(&file2_path, content).unwrap();
        
        let scanner = Scanner::new();
        let receiver = scanner.receiver();
        
        scanner.start_scan(temp_dir.path(), vec![]);
        
        let mut found_duplicates = false;
        while let Ok(message) = receiver.recv_timeout(std::time::Duration::from_secs(5)) {
            match message {
                ScannerMessage::Found(duplicates) => {
                    assert_eq!(duplicates.len(), 1);
                    found_duplicates = true;
                    break;
                }
                ScannerMessage::Error(e) => {
                    panic!("Scanner error: {}", e);
                }
                _ => {}
            }
        }
        
        assert!(found_duplicates, "Should have found duplicates");
    }
}
