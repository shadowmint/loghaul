use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LoghaulErrorCode {
    NotImplemented,
    InvalidSource,
    SourceErr(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoghaulError {
    code: LoghaulErrorCode
}

impl Error for LoghaulError {}

impl fmt::Display for LoghaulError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<LoghaulErrorCode> for LoghaulError {
    fn from(code: LoghaulErrorCode) -> Self {
        return LoghaulError {
            code
        }
    }
}

