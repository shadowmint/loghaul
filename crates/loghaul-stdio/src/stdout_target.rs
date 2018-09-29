use loghaul::Target;
use loghaul::LoghaulError;
use loghaul::StreamEntry;
use std::io::Write;
use std::io;

pub struct StdoutTarget {}

impl StdoutTarget {
    pub fn new() -> StdoutTarget {
        return StdoutTarget {};
    }

    fn write(&mut self, data: &Vec<u8>) {
        io::stdout().write(data).unwrap();
    }
}

impl Target for StdoutTarget {
    fn consume(&mut self, entry: StreamEntry, data: &Vec<u8>) -> Result<(), LoghaulError> {
        match entry {
            StreamEntry::NoData => {}
            StreamEntry::EOF => {}
            StreamEntry::Data => {
                self.write(data);
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::StdoutTarget;
    use loghaul::Stream;
    use loghaul::mock::MockSource;
    use std::sync::{Arc, Mutex};
    use loghaul::KeeperConfig;
    use loghaul::Keeper;
    use std::time::Duration;
    use loghaul::mock::MockKeeperLog;
    use loghaul::KeeperEofStrategy;
    use std::thread::sleep;

    #[test]
    fn test_combine_sources_to_file_target() {
        let stream = Stream::new()
            .with_source(MockSource::new(vec!("1\n", "2\n", "3\n", "4\n", "5\n")))
            .with_source(MockSource::new(vec!("11\n", "12\n", "13\n", "14\n", "15\n")))
            .with_target(StdoutTarget::new());

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::DropSource,
            logger: Some(Box::new(MockKeeperLog::new(log))),
        }));

        sleep(Duration::from_millis(100));
        keeper.halt();
    }
}
