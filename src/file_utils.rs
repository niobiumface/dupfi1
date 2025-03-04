use std::{
    fs,
    io,
    path::{Path, PathBuf},
    collections::HashMap,
};
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub size: u64,
    pub hash: Option<Vec<u8>>,
}

impl FileInfo {
    pub fn new(path: PathBuf) -> io::Result<Self> {
        let metadata = fs::metadata(&path)?;
        Ok(Self {
            path,
            size: metadata.len(),
            hash: None,
        })
    }

    pub fn calculate_hash(&mut self) -> io::Result<()> {
        let contents = fs::read(&self.path)?;
        let hash = Sha256::digest(&contents);
        self.hash = Some(hash.to_vec());
        Ok(())
    }
}

pub fn collect_files(dir: &Path, filters: &[String]) -> Vec<FileInfo> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file() && 
            !should_ignore(&e.path(), filters)
        })
        .filter_map(|e| FileInfo::new(e.path().to_path_buf()).ok())
        .collect()
}

pub fn find_duplicates(files: &mut [FileInfo]) -> HashMap<Vec<u8>, Vec<PathBuf>> {
    // First group by size to reduce hash calculations
    let size_groups: HashMap<u64, Vec<&mut FileInfo>> = files
        .iter_mut()
        .fold(HashMap::new(), |mut map, file| {
            map.entry(file.size).or_default().push(file);
            map
        });

    // Only hash files of the same size
    let mut hash_map: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();
    
    size_groups.into_values()
        .filter(|group| group.len() > 1)
        .for_each(|group| {
            group.into_par_iter()
                .for_each(|file| {
                    let _ = file.calculate_hash();
                });
        });

    // Group files by hash
    files.iter()
        .filter_map(|file| {
            file.hash.as_ref().map(|hash| (hash.clone(), file.path.clone()))
        })
        .for_each(|(hash, path)| {
            hash_map.entry(hash).or_default().push(path);
        });

    // Only keep groups with duplicates
    hash_map.retain(|_, paths| paths.len() > 1);
    hash_map
}

pub fn should_ignore(path: &Path, filters: &[String]) -> bool {
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            return filters.iter().any(|filter| filter == ext_str);
        }
    }
    false
}

pub fn create_hardlink(src: &Path, dst: &Path) -> io::Result<()> {
    fs::hard_link(src, dst)
}

pub fn move_file(src: &Path, dst: &Path) -> io::Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(src, dst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_find_duplicates() {
        let temp_dir = tempdir().unwrap();
        
        // Create some test files
        let file1_path = temp_dir.path().join("file1.txt");
        let file2_path = temp_dir.path().join("file2.txt");
        let file3_path = temp_dir.path().join("file3.txt");
        
        let content1 = b"test content";
        let content2 = b"different content";
        
        fs::write(&file1_path, content1).unwrap();
        fs::write(&file2_path, content1).unwrap(); // Duplicate of file1
        fs::write(&file3_path, content2).unwrap();
        
        let mut files: Vec<FileInfo> = vec![
            FileInfo::new(file1_path).unwrap(),
            FileInfo::new(file2_path).unwrap(),
            FileInfo::new(file3_path).unwrap(),
        ];
        
        let duplicates = find_duplicates(&mut files);
        
        assert_eq!(duplicates.len(), 1); // One group of duplicates
        
        for paths in duplicates.values() {
            assert_eq!(paths.len(), 2); // Each group should have 2 files
        }
    }

    #[test]
    fn test_file_filters() {
        let temp_dir = tempdir().unwrap();
        
        // Create test files with different extensions
        let txt_file = temp_dir.path().join("test.txt");
        let doc_file = temp_dir.path().join("test.doc");
        
        fs::File::create(&txt_file).unwrap();
        fs::File::create(&doc_file).unwrap();
        
        let filters = vec!["doc".to_string()];
        let files = collect_files(temp_dir.path(), &filters);
        
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path.extension().unwrap(), "txt");
    }

    #[test]
    fn test_hardlink_creation() {
        let temp_dir = tempdir().unwrap();
        
        let original = temp_dir.path().join("original.txt");
        let linked = temp_dir.path().join("linked.txt");
        
        let content = b"test content";
        fs::write(&original, content).unwrap();
        
        create_hardlink(&original, &linked).unwrap();
        
        assert!(linked.exists());
        assert_eq!(fs::read(&original).unwrap(), fs::read(&linked).unwrap());
    }
}
