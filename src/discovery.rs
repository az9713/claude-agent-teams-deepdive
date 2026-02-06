use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::error::Result;

const BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "so", "dylib", "o", "obj", "bin", "a", "lib", "png", "jpg", "jpeg", "gif",
    "bmp", "ico", "tiff", "webp", "pdf", "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "wasm",
    "class", "pyc", "pdb",
];

pub struct FileDiscovery {
    root: PathBuf,
    max_file_size: u64,
    respect_gitignore: bool,
}

impl FileDiscovery {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_file_size: 1_048_576,
            respect_gitignore: true,
        }
    }

    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    pub fn with_gitignore(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn discover(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        let walker = WalkBuilder::new(&self.root)
            .git_ignore(self.respect_gitignore)
            .add_custom_ignore_filename(".todoignore")
            .build();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            // Skip directories
            if entry.file_type().map_or(true, |ft| !ft.is_file()) {
                continue;
            }

            let path = entry.path().to_path_buf();

            // Filter by file size
            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(_) => continue,
            };
            if metadata.len() > self.max_file_size {
                continue;
            }

            // Skip files with known binary extensions
            if is_binary_extension(&path) {
                continue;
            }

            // Skip binary files by checking for null bytes in first 512 bytes
            if is_binary_content(&path) {
                continue;
            }

            files.push(path);
        }

        files.sort();
        Ok(files)
    }
}

fn is_binary_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| BINARY_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

fn is_binary_content(path: &Path) -> bool {
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buffer = [0u8; 512];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };
    buffer[..bytes_read].contains(&0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        // Create some text files
        fs::write(dir.path().join("main.rs"), "// TODO: fix this\nfn main() {}").unwrap();
        fs::write(dir.path().join("lib.rs"), "pub fn hello() {}").unwrap();
        // Create a subdirectory with a file
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("sub").join("mod.rs"), "// FIXME: broken").unwrap();
        dir
    }

    #[test]
    fn test_discover_finds_text_files() {
        let dir = create_test_dir();
        let discovery = FileDiscovery::new(dir.path());
        let files = discovery.discover().unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_discover_skips_binary_extensions() {
        let dir = create_test_dir();
        fs::write(dir.path().join("image.png"), &[0x89, 0x50, 0x4E, 0x47]).unwrap();
        fs::write(dir.path().join("program.exe"), &[0x4D, 0x5A, 0x00]).unwrap();

        let discovery = FileDiscovery::new(dir.path());
        let files = discovery.discover().unwrap();
        // Should only have the 3 text files, not the binary ones
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_discover_skips_large_files() {
        let dir = create_test_dir();
        // Create a file larger than 100 bytes
        let big_content = "x".repeat(200);
        fs::write(dir.path().join("big.txt"), &big_content).unwrap();

        let discovery = FileDiscovery::new(dir.path()).with_max_file_size(100);
        let files = discovery.discover().unwrap();
        // big.txt should be excluded
        assert!(!files.iter().any(|p| p.file_name().unwrap() == "big.txt"));
    }

    #[test]
    fn test_discover_skips_binary_content() {
        let dir = create_test_dir();
        // Create a file with null bytes (binary content)
        let mut content = Vec::new();
        content.extend_from_slice(b"some text");
        content.push(0); // null byte
        content.extend_from_slice(b"more text");
        fs::write(dir.path().join("binary.dat"), &content).unwrap();

        let discovery = FileDiscovery::new(dir.path());
        let files = discovery.discover().unwrap();
        assert!(!files.iter().any(|p| p.file_name().unwrap() == "binary.dat"));
    }

    #[test]
    fn test_discover_respects_todoignore() {
        let dir = create_test_dir();
        // Create a .todoignore file that ignores lib.rs
        fs::write(dir.path().join(".todoignore"), "lib.rs\n").unwrap();

        let discovery = FileDiscovery::new(dir.path());
        let files = discovery.discover().unwrap();
        assert!(!files.iter().any(|p| p.file_name().unwrap() == "lib.rs"));
    }

    #[test]
    fn test_discover_results_sorted() {
        let dir = create_test_dir();
        fs::write(dir.path().join("zzz.rs"), "// last").unwrap();
        fs::write(dir.path().join("aaa.rs"), "// first").unwrap();

        let discovery = FileDiscovery::new(dir.path());
        let files = discovery.discover().unwrap();
        let is_sorted = files.windows(2).all(|w| w[0] <= w[1]);
        assert!(is_sorted);
    }

    #[test]
    fn test_builder_methods() {
        let dir = TempDir::new().unwrap();
        let discovery = FileDiscovery::new(dir.path())
            .with_max_file_size(500)
            .with_gitignore(false);
        assert_eq!(discovery.max_file_size, 500);
        assert!(!discovery.respect_gitignore);
    }
}
