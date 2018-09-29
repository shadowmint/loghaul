use Source;
use Target;
use LoghaulErrorAggregate;
use std::mem;
use StreamEntry;

pub struct Stream {
    sources: Vec<SourceBucket>,
    targets: Vec<Box<Target + Send + 'static>>,
}

struct SourceBucket {
    eof: bool,
    source: Box<Source + Send + 'static>,
    buffer: Vec<u8>,
}

impl SourceBucket {
    fn to_source(self) -> Box<Source + Send + 'static> {
        self.source
    }
}

/// Stream is an explicit multi-producer, multi-consumer system.
/// A stream must be manually pumped by calling `step`:
///
/// ```
///     use loghaul::Stream;
///     let mut stream = Stream::new();
///     let mut dropped = Vec::new();
///     stream.step(&mut dropped); // Process events
/// ```
///
/// To automatically process events, use a `Keeper`
impl Stream {
    /// Create a new empty stream.
    pub fn new() -> Stream {
        return Stream {
            sources: Vec::new(),
            targets: Vec::new(),
        };
    }

    /// Add a new data source to this stream
    pub fn with_source(mut self, source: impl Source + Send + 'static) -> Stream {
        self.add_source(source);
        return self;
    }

    /// Add a new data source to this stream
    pub fn add_source(&mut self, source: impl Source + Send + 'static) {
        self.add_boxed_source(Box::new(source));
    }

    /// Add a new data source to this stream
    pub fn add_boxed_source(&mut self, source: Box<Source + Send + 'static>) {
        self.sources.push(SourceBucket {
            eof: false,
            source: source,
            buffer: Vec::new(),
        });
    }

    /// Add a new data target to this stream
    pub fn with_target(mut self, target: impl Target + Send + 'static) -> Stream {
        self.add_target(target);
        return self;
    }

    /// Add a new data target to this stream
    pub fn add_target(&mut self, target: impl Target + Send + 'static) {
        self.targets.push(Box::new(target));
    }

    /// Process every input once and pass every received value to every output
    /// Any EOF sources should be removed and added to the eof array.
    pub fn step(&mut self, eof: &mut Vec<Box<Source + Send + 'static>>) -> Result<(), LoghaulErrorAggregate> {
        eof.clear();
        let mut errors = LoghaulErrorAggregate::new();
        let mut eof_count = 0;
        for source in self.sources.iter_mut() {
            source.buffer.clear();
            match source.source.poll(&mut source.buffer) {
                Ok(entry) => {
                    for target in self.targets.iter_mut() {
                        match target.consume(entry, &source.buffer) {
                            Ok(_) => {}
                            Err(e) => {
                                errors.push(e);
                            }
                        }
                    }
                    match entry {
                        StreamEntry::EOF => {
                            source.eof = true;
                            eof_count += 1;
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        // Remove eof sources, they can never generate again
        if eof_count > 0 {
            let mut source_list = Vec::new();
            mem::swap(&mut self.sources, &mut source_list);
            let (mut active, completed): (Vec<_>, Vec<_>) = source_list.into_iter().partition(|ref e| !e.eof);
            mem::swap(&mut self.sources, &mut active);
            for source in completed.into_iter() {
                eof.push(source.to_source())
            }
        }

        return errors.to_result();
    }
}

#[cfg(test)]
mod tests {
    use super::Stream;
    use mock::mock_source::MockSource;
    use streams::stream_entry::StreamEntry;
    use errors::loghaul_error::{LoghaulErrorCode, LoghaulError};
    use std::convert::From;
    use mock::mock_target::MockTarget;
    use streams::stream_buffer::StreamBuffer;

    #[test]
    fn test_create_stream() {
        Stream::new()
            .with_source(MockSource::new(vec!("1", "2")))
            .with_source(MockSource::empty())
            .with_target(MockTarget::new(|entry, data| -> Result<(), LoghaulError> {
                Err(LoghaulError::from(LoghaulErrorCode::NotImplemented))
            }));
    }

    #[test]
    fn test_stream_step() {
        let mut s = Stream::new()
            .with_source(MockSource::new(vec!("1", "2")))
            .with_source(MockSource::new(vec!("3", "4")));

        s.add_target(MockTarget::new(|entry, data| -> Result<(), LoghaulError> {
            Ok(())
        }));

        let mut dropped = Vec::new();
        assert!(s.step(&mut dropped).is_ok());
    }

    #[test]
    fn test_stream_drops_eof_sources() {
        let mut s = Stream::new()
            .with_source(MockSource::closed(vec!("1", "2")))
            .with_source(MockSource::closed(vec!("3")));

        let mut dropped = Vec::new();
        assert!(s.step(&mut dropped).is_ok());
        assert_eq!(dropped.len(), 0);

        assert!(s.step(&mut dropped).is_ok());
        assert_eq!(dropped.len(), 1);

        assert!(s.step(&mut dropped).is_ok());
        assert_eq!(dropped.len(), 1);

        assert!(s.step(&mut dropped).is_ok());
        assert_eq!(dropped.len(), 0);
    }
}
