use errors::loghaul_error::LoghaulError;
use streams::stream_entry::StreamEntry;
use Source;
use streams::stream_buffer::StreamBuffer;

pub struct MockSourceState {
    pub finite: bool,
    pub closed: bool,
    pub error: Option<LoghaulError>,
    pub data: Vec<String>,
    backup: Vec<String>,
}

pub struct MockSource {
    pub state: MockSourceState,
    pub handle: Box<Fn(&mut MockSourceState) + Send>,
}

impl MockSource {
    pub fn new(values: Vec<&'static str>) -> MockSource {
        let mut rtn = MockSource {
            state: MockSourceState {
                finite: false,
                closed: false,
                error: None,
                backup: values.iter().map(|x| x.to_string()).collect(),
                data: Vec::new(),
            },
            handle: Box::new(|_| {}),
        };
        rtn.restore();
        return rtn;
    }

    /// EOF when out of data
    pub fn closed(values: Vec<&'static str>) -> MockSource {
        let mut rtn = MockSource {
            state: MockSourceState {
                finite: true,
                closed: false,
                error: None,
                backup: values.iter().map(|x| x.to_string()).collect(),
                data: Vec::new(),
            },
            handle: Box::new(|_| {}),
        };
        rtn.restore();
        return rtn;
    }

    pub fn empty() -> MockSource {
        let mut rtn = MockSource {
            state: MockSourceState {
                finite: false,
                closed: false,
                error: None,
                data: Vec::new(),
                backup: Vec::new(),
            },
            handle: Box::new(|_| {}),
        };
        rtn.restore();
        return rtn;
    }

    fn restore(&mut self) {
        self.state.data = self.state.backup.iter().map(|i| i.clone()).collect();
        self.state.closed = false;
    }

    fn step(&mut self, output: &mut Vec<u8>) -> Result<StreamEntry, LoghaulError> {
        match self.state.error.take() {
            Some(e) => {
                return Err(e);
            }
            None => {}
        }

        if self.state.data.len() == 0 && self.state.finite {
            self.state.closed = true;
        }

        if self.state.closed {
            return Ok(StreamEntry::EOF);
        }

        if self.state.data.len() > 0 {
            let entry = self.state.data.remove(0);
            let mut buffer = StreamBuffer::new(output);
            buffer.clear();
            buffer.push_str(&entry);
            return Ok(StreamEntry::Data);
        } else {
            output.clear();
        }

        return Ok(StreamEntry::NoData);
    }
}

impl Source for MockSource {
    fn poll(&mut self, buffer: &mut Vec<u8>) -> Result<StreamEntry, LoghaulError> {
        self.step(buffer)
    }

    fn resume(&mut self) -> Result<(), LoghaulError> {
        self.restore();
        return Ok(());
    }
}