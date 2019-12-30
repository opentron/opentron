use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0:}")]
    Io(#[from] std::io::Error),
    #[error("{0:}")]
    Serde(#[from] ::serde_json::error::Error),
    #[error("error: {0:?}")]
    Keys(#[from] ::keys::Error),
    #[error("{0:}")]
    Runtime(&'static str),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Runtime(s)
    }
}
