use Target;
use streams::stream_entry::StreamEntry;
use errors::loghaul_error::LoghaulError;

pub struct MockTarget {
    consumer: Box<Fn(StreamEntry, &Vec<u8>) -> Result<(), LoghaulError> + Send>
}

impl Target for MockTarget {
    fn consume(&mut self, entry: StreamEntry, data: &Vec<u8>) -> Result<(), LoghaulError> {
        (self.consumer)(entry, data)
    }
}

impl MockTarget {
    /// Create a new mock targettest
    pub fn new(consumer: impl Fn(StreamEntry, &Vec<u8>) -> Result<(), LoghaulError> + Send + 'static) -> MockTarget {
        return MockTarget {
            consumer: Box::new(consumer)
        };
    }
}