use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum MinionError {
    Message(String),
    BrokenPipe,
}

impl MinionError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl Display for MinionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::BrokenPipe => f.write_str("broken pipe"),
        }
    }
}

impl Error for MinionError {}
