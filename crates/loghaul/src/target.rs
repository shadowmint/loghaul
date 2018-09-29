use streams::stream_entry::StreamEntry;
use LoghaulError;

pub trait Target {
    fn consume(&mut self, entry: StreamEntry, data: &Vec<u8>) -> Result<(), LoghaulError>;
}