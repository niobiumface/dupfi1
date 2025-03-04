use std::{
    fs,
    path::Path,
    io::{self, Read},
};
use image::ImageFormat;

pub enum Preview {
    Text(String),
    Image(Vec<u8>, ImageFormat),
    Binary,
}

impl Preview {
    pub fn from_file(path: &Path) -> io::Result<Self> {
        // First check if it's an image
        if let Ok(format) = image::ImageFormat::from_path(path) {
            let bytes = fs::read(path)?;
            return Ok(Preview::Image(bytes, format));
        }

        // Try to read as text
        let mut file = fs::File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check if the content appears to be text
        if is_text_content(&buffer) {
            if let Ok(text) = String::from_utf8(buffer) {
                return Ok(Preview::Text(text));
            }
        }

        // If not text or image, treat as binary
        Ok(Preview::Binary)
    }

    pub fn to_string(&self) -> String {
        match self {
            Preview::Text(content) => {
                if content.len() > 1000 {
                    format!("{}...", &content[..1000])
                } else {
                    content.clone()
                }
            }
            Preview::Image(_, format) => {
                format!("Image file ({})", format.extensions_str()[0])
            }
            Preview::Binary => "Binary file".to_string(),
        }
    }
}

fn is_text_content(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return true;
    }

    // Check for null bytes and control characters
    let mut null_count = 0;
    let mut control_count = 0;

    for &byte in buffer.iter().take(1024) {
        if byte == 0 {
            null_count += 1;
        } else if byte < 32 && byte != b'\n' && byte != b'\r' && byte != b'\t' {
            control_count += 1;
        }
    }

    // Allow some control characters but very few null bytes
    let sample_size = buffer.len().min(1024) as f32;
    let null_ratio = null_count as f32 / sample_size;
    let control_ratio = control_count as f32 / sample_size;

    null_ratio < 0.01 && control_ratio < 0.1
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_text_preview() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Hello, world!\nThis is a test.").unwrap();
        
        let preview = Preview::from_file(file.path()).unwrap();
        match preview {
            Preview::Text(content) => {
                assert!(content.contains("Hello, world!"));
                assert!(content.contains("This is a test."));
            }
            _ => panic!("Expected text preview"),
        }
    }

    #[test]
    fn test_binary_preview() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(&[0, 159, 146, 150]).unwrap();
        
        let preview = Preview::from_file(file.path()).unwrap();
        match preview {
            Preview::Binary => {}
            _ => panic!("Expected binary preview"),
        }
    }
}
