use std::fs::File;
use std::path::Path;
use std::io::BufReader;
use std::io::BufRead;
use std::fs::OpenOptions;
use std::io::Write;
use internal::temp_file::TempFile;

/// Make a new temp file
pub fn random_test_file() -> TempFile {
    return TempFile::new();
}

/// Read an entire file and split it by lines
pub fn read_entire_file(path: impl AsRef<Path>) -> Option<Vec<String>> {
    match File::open(path) {
        Ok(file) => {
            Some(BufReader::new(file).lines().into_iter().map(|i| i.unwrap_or(String::new())).collect())
        }
        Err(_) => None
    }
}

/// Write a single line to a file
pub fn write_line_to_file(path: impl AsRef<Path>, line: &str) {
    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut fp) => {
            fp.write_all(line.as_bytes()).unwrap();
        }
        Err(_) => {}
    }
}