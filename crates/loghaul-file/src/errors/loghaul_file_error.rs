use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LoghaulFileErrorCode {
    UnableToOpenFile,
    WrappedError
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LoghaulFileError {
    code: LoghaulFileErrorCode,
    message: String,
}

impl LoghaulFileError {
    pub fn new(code: LoghaulFileErrorCode, detail: Option<&Error>) -> LoghaulFileError {
        return LoghaulFileError {
            code,
            message: format!("{:?}", detail),
        };
    }
}

impl Error for LoghaulFileError {}

impl fmt::Display for LoghaulFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<io::Error> for LoghaulFileError {
    fn from(err: io::Error) -> Self {
        return LoghaulFileError::new(LoghaulFileErrorCode::WrappedError, Some(&err))
    }
}

