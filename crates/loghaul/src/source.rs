use streams::stream_entry::StreamEntry;
use LoghaulError;

pub trait Source {
    /// Poll this source for the next entry; if one is present, it should be
    /// written into the buffer provided. EOF should be returned if the source
    /// is closed, not an error.
    fn poll(&mut self, buffer: &mut Vec<u8>) -> Result<StreamEntry, LoghaulError>;

    /// If this Source has EOF, attempt to restart and source and begin
    /// reading from it again.
    ///
    /// For example, if a file has been removed, a source may EOF, but we
    /// may want to periodically attempt to restart, reading from the file
    /// again.
    fn resume(&mut self) -> Result<(), LoghaulError>;
}