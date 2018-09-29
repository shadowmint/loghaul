use std::error::Error;
use std::fmt;
use LoghaulError;

#[derive(Debug)]
pub struct LoghaulErrorAggregate {
    errors: Vec<LoghaulError>
}

impl LoghaulErrorAggregate {
    /// Create a new aggregate
    pub fn new() -> LoghaulErrorAggregate {
        return LoghaulErrorAggregate {
            errors: Vec::new()
        };
    }

    /// Add an error
    pub fn push(&mut self, error: LoghaulError) {
        self.errors.push(error);
    }

    /// Return the length of the internal error set
    pub fn len(&self) -> usize {
        return self.errors.len();
    }

    /// Convert into a result statement
    pub fn to_result(self) -> Result<(), LoghaulErrorAggregate> {
        if self.len() > 0 {
            return Err(self);
        }
        return Ok(());
    }
}

impl Error for LoghaulErrorAggregate {}

impl fmt::Display for LoghaulErrorAggregate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
