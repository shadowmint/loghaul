/// Standard buffer implementation to deliver chunks
pub struct StreamBuffer<'a> {
    data: &'a mut Vec<u8>
}

impl<'a> StreamBuffer<'a> {
    pub fn new(inner: &'a mut Vec<u8>) -> StreamBuffer {
        return StreamBuffer {
            data: inner
        };
    }

    /// Clear the internal buffer without modifying its capacity
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Write a string to the internal buffer, growing it as required
    pub fn push_str(&mut self, data: &str) {
        let bytes = data.as_bytes();
        self.data.resize(bytes.len(), 0);
        self.data[..bytes.len()].copy_from_slice(bytes);
    }
}