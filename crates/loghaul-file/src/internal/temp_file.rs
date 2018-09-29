use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::fs::remove_file;
use rand::Rng;
use rand;
use std::fs::OpenOptions;
use std::io::Write;

pub struct TempFile {
    pub path: String
}

impl TempFile {
    pub fn new() -> TempFile {
        return TempFile {
            path: TempFile::random_filename()
        };
    }

    pub fn write(&self, line: &str) {
        match OpenOptions::new().create(true).append(true).open(&self.path) {
            Ok(mut fp) => {
                fp.write_all(line.as_bytes()).unwrap();
            }
            Err(_) => {}
        }
    }

    /// Generate a random filename
    fn random_filename() -> String {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
        let in_ms = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
        let random_id = rand::thread_rng().gen_range(0, 100000);
        return format!("test_output_{}_{}.txt", in_ms, random_id);
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        remove_file(&self.path).unwrap();
    }
}