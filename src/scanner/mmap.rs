use std::fs::File;
use std::path::Path;

use memmap2::Mmap;

/// Memory-map a file for reading. Falls back to regular read for small files.
pub fn read_file_contents(path: &Path) -> std::io::Result<String> {
    let metadata = std::fs::metadata(path)?;
    let size = metadata.len();

    if size > 256 * 1024 {
        // > 256KB: use mmap
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };
        String::from_utf8(mmap.to_vec())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    } else {
        std::fs::read_to_string(path)
    }
}
