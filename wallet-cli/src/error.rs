use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("serde error: {0:?}")]
    Serde(#[from] ::serde_json::error::Error),
    #[error("grpc error: {0:?}")]
    Grpc(#[from] ::grpc::Error),
    #[error("runtime error: {0:}")]
    Runtime(&'static str),
    #[error("error: {0:?}")]
    Keys(#[from] ::keys::Error),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Runtime(s)
    }
}
