#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, io::Write, path::PathBuf};
    use tempfile::TempDir;
    use crate::{
        file_utils::{FileInfo, find_duplicates},
        file_scanner::Scanner,
        preview::Preview,
    };

    fn create_test_files() -> (TempDir, Vec<PathBuf>) {
        let temp_dir = TempDir::new().unwrap();
        let mut paths = Vec::new();

        // Create some duplicate files
        let content1 = b"test content 1";
        let content2 = b"test content 2";

        // First group of duplicates
        let file1 = temp_dir.path().join("group1_original.txt");
        let file2 = temp_dir.path().join("group1_duplicate1.txt");
        let file3 = temp_dir.path().join("group1_duplicate2.txt");
        
        fs::write(&file1, content1).unwrap();
        fs::write(&file2, content1).unwrap();
        fs::write(&file3, content1).unwrap();
        
        paths.push(file1);
        paths.push(file2);
        paths.push(file3);

        // Second group of duplicates
        let file4 = temp_dir.path().join("group2_original.txt");
        let file5 = temp_dir.path().join("group2_duplicate.txt");
        
        fs::write(&file4, content2).unwrap();
        fs::write(&file5, content2).unwrap();
        
        paths.push(file4);
        paths.push(file5);

        // Unique file
        let file6 = temp_dir.path().join("unique.txt");
        fs::write(&file6, b"unique content").unwrap();
        paths.push(file6);

        (temp_dir, paths)
    }

    #[test]
    fn test_duplicate_detection() {
        let (temp_dir, paths) = create_test_files();
        
        let mut files: Vec<FileInfo> = paths.iter()
            .map(|p| FileInfo::new(p.clone()).unwrap())
            .collect();
        
        let duplicates = find_duplicates(&mut files);
        
        // Should find 2 groups of duplicates
        assert_eq!(duplicates.len(), 2);
        
        // First group should have 3 files
        assert!(duplicates.values().any(|group| group.len() == 3));
        
        // Second group should have 2 files
        assert!(duplicates.values().any(|group| group.len() == 2));
    }

    #[test]
    fn test_file_scanner() {
        let (temp_dir, _) = create_test_files();
        
        let scanner = Scanner::new();
        let receiver = scanner.receiver();
        
        scanner.start_scan(temp_dir.path(), vec![]);
        
        let mut found_duplicates = false;
        let mut progress_reported = false;
        
        while let Ok(message) = receiver.recv_timeout(std::time::Duration::from_secs(5)) {
            match message {
                crate::file_scanner::ScannerMessage::Progress(progress) => {
                    assert!(progress >= 0.0 && progress <= 1.0);
                    progress_reported = true;
                }
                crate::file_scanner::ScannerMessage::Found(duplicates) => {
                    assert_eq!(duplicates.len(), 2); // Should find our 2 groups
                    found_duplicates = true;
                    break;
                }
                crate::file_scanner::ScannerMessage::Error(e) => {
                    panic!("Scanner error: {}", e);
                }
            }
        }
        
        assert!(found_duplicates, "Should have found duplicates");
        assert!(progress_reported, "Should have reported progress");
    }

    #[test]
    fn test_preview_generation() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test text file preview
        let text_file = temp_dir.path().join("test.txt");
        fs::write(&text_file, "Hello, world!").unwrap();
        
        match Preview::from_file(&text_file).unwrap() {
            Preview::Text(content) => {
                assert_eq!(content, "Hello, world!");
            }
            _ => panic!("Expected text preview"),
        }
        
        // Test binary file preview
        let binary_file = temp_dir.path().join("test.bin");
        let mut file = fs::File::create(&binary_file).unwrap();
        file.write_all(&[0, 1, 2, 3, 4]).unwrap();
        
        match Preview::from_file(&binary_file).unwrap() {
            Preview::Binary => {}
            _ => panic!("Expected binary preview"),
        }
    }

    #[test]
    fn test_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create original file
        let original = temp_dir.path().join("original.txt");
        let content = "test content";
        fs::write(&original, content).unwrap();
        
        // Test hardlink
        let hardlink = temp_dir.path().join("hardlink.txt");
        crate::file_utils::create_hardlink(&original, &hardlink).unwrap();
        
        assert!(hardlink.exists());
        assert_eq!(fs::read_to_string(&hardlink).unwrap(), content);
        
        // Test move
        let moved = temp_dir.path().join("moved.txt");
        crate::file_utils::move_file(&hardlink, &moved).unwrap();
        
        assert!(!hardlink.exists());
        assert!(moved.exists());
        assert_eq!(fs::read_to_string(&moved).unwrap(), content);
    }
}
