use loghaul::Target;
use loghaul::LoghaulError;
use loghaul::StreamEntry;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::fs::OpenOptions;
use LoghaulFileError;
use LoghaulFileErrorCode;

pub struct FileTarget {
    path: PathBuf,
    fp: Option<File>,
}

impl FileTarget {
    pub fn new(path: impl AsRef<Path>) -> FileTarget {
        return FileTarget {
            path: PathBuf::from(path.as_ref()),
            fp: None,
        };
    }

    fn write(&mut self, data: &Vec<u8>) -> Result<(), LoghaulFileError> {
        self.open_fp()?;
        if self.fp.is_some() {
            self.fp.as_mut().unwrap().write_all(data)?;
        }
        Ok(())
    }

    fn close(&mut self) {
        self.fp = None;
    }

    fn open_fp(&mut self) -> Result<(), LoghaulFileError> {
        if self.fp.is_some() {
            return Ok(());
        }
        match OpenOptions::new().create(true).append(true).open(&self.path) {
            Ok(fp) => {
                self.fp = Some(fp);
                Ok(())
            }
            Err(err) => {
                Err(LoghaulFileError::new(LoghaulFileErrorCode::UnableToOpenFile, Some(&err)))
            }
        }
    }
}

impl Target for FileTarget {
    fn consume(&mut self, entry: StreamEntry, data: &Vec<u8>) -> Result<(), LoghaulError> {
        match entry {
            StreamEntry::NoData => {},
            StreamEntry::EOF => {},
            StreamEntry::Data => {
                match self.write(data) {
                    Ok(_) => {},
                    Err(err) => {
                        // TODO: Log the error here to our own error log
                        self.close();
                        println!("{:?}", err);
                    }
                }
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FileTarget;
    use loghaul::Stream;
    use loghaul::mock::MockSource;
    use std::sync::{Arc, Mutex};
    use loghaul::KeeperConfig;
    use loghaul::Keeper;
    use std::time::Duration;
    use loghaul::mock::MockKeeperLog;
    use loghaul::KeeperEofStrategy;
    use internal::file_test_helpers::read_entire_file;
    use std::thread::sleep;
    use internal::file_test_helpers::random_test_file;

    #[test]
    fn test_combine_sources_to_file_target() {
        let output_path = random_test_file();

        let stream = Stream::new()
            .with_source(MockSource::new(vec!("1\n", "2\n", "3\n", "4\n", "5\n")))
            .with_source(MockSource::new(vec!("11\n", "12\n", "13\n", "14\n", "15\n")))
            .with_target(FileTarget::new(&output_path.path));

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::DropSource,
            logger: Some(Box::new(MockKeeperLog::new(log))),
        }));

        sleep(Duration::from_millis(100));
        keeper.halt();

        let contents = read_entire_file(&output_path.path).unwrap();
        assert_eq!(10, contents.len());
    }
}
