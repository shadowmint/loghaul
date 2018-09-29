use loghaul::LoghaulError;
use loghaul::StreamEntry;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::fs::OpenOptions;
use LoghaulFileError;
use LoghaulFileErrorCode;
use loghaul::Source;
use loghaul::LoghaulErrorCode;
use std::error::Error;
use std::io::Read;

pub struct FileSource {
    path: PathBuf,
    fp: Option<File>,
}

impl FileSource {
    pub fn new(path: impl AsRef<Path>) -> FileSource {
        return FileSource {
            path: PathBuf::from(path.as_ref()),
            fp: None,
        };
    }

    fn close(&mut self) {
        self.fp = None;
    }

    fn open_fp(&mut self) -> Result<(), LoghaulFileError> {
        if self.fp.is_some() {
            return Ok(());
        }
        match OpenOptions::new().read(true).open(&self.path) {
            Ok(fp) => {
                self.fp = Some(fp);
                Ok(())
            }
            Err(err) => {
                Err(LoghaulFileError::new(LoghaulFileErrorCode::UnableToOpenFile, Some(&err)))
            }
        }
    }

    fn read_pending_lines(&mut self, buffer: &mut Vec<u8>) -> Result<StreamEntry, LoghaulFileError> {
        if self.open_fp().is_err() {
            return Ok(StreamEntry::EOF);
        }

        let fp = self.fp.as_mut().unwrap();
        match fp.read_to_end(buffer) {
            Ok(size) => {
                match size {
                    v if { v > 0 } => Ok(StreamEntry::Data),
                    _ => Ok(StreamEntry::NoData)
                }
            }
            Err(err) => {
                Err(LoghaulFileError::new(LoghaulFileErrorCode::WrappedError, Some(&err)))
            }
        }
    }
}

impl Source for FileSource {
    fn poll(&mut self, buffer: &mut Vec<u8>) -> Result<StreamEntry, LoghaulError> {
        match self.read_pending_lines(buffer) {
            Ok(v) => Ok(v),
            Err(e) => Err(LoghaulError::from(LoghaulErrorCode::SourceErr(e.description().to_string())))
        }
    }

    fn resume(&mut self) -> Result<(), LoghaulError> {
        self.close();
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use loghaul::Stream;
    use std::sync::{Arc, Mutex};
    use loghaul::KeeperConfig;
    use loghaul::Keeper;
    use std::time::Duration;
    use loghaul::mock::MockKeeperLog;
    use loghaul::KeeperEofStrategy;
    use std::thread::sleep;
    use internal::file_test_helpers::random_test_file;
    use loghaul::mock::MockTarget;
    use loghaul::StreamEntry;
    use std::str::from_utf8;
    use loghaul::LoghaulError;
    use FileSource;
    use internal::file_test_helpers::write_line_to_file;

    #[test]
    fn test_single_file_source_to_buffer() {
        let input_path = random_test_file();

        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let results_bucket = results.clone();

        let stream = Stream::new()
            .with_source(FileSource::new(&input_path.path))
            .with_target(MockTarget::new(move |value, buffer| -> Result<(), LoghaulError> {
                match value {
                    StreamEntry::Data => {
                        match from_utf8(buffer) {
                            Ok(svalue) => {
                                if svalue.len() > 0 {
                                    results_bucket.lock().unwrap().push(svalue.to_string());
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }));

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::DropSource,
            logger: Some(Box::new(MockKeeperLog::new(log))),
        }));

        write_line_to_file(&input_path.path, "One line goes here\n");
        sleep(Duration::from_millis(10));
        write_line_to_file(&input_path.path, "two\n");
        sleep(Duration::from_millis(10));
        write_line_to_file(&input_path.path, "three\n");

        sleep(Duration::from_millis(100));
        keeper.halt();

        let output: Vec<String> = results.lock().unwrap().iter().map(|i| (&i).to_string()).collect();
        assert_eq!(3, output.len());
    }
}
