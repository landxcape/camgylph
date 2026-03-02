use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Config(String),
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Config(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for AppError {}
