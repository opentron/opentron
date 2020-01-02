use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error: {0:}")]
    Io(#[from] ::std::io::Error),
    #[error("{0:}")]
    Fmt(#[from] ::std::fmt::Error),
    #[error("serde error: {0:?}")]
    Serde(#[from] ::serde_json::error::Error),
    #[error("grpc error: {0:?}")]
    Grpc(#[from] ::grpc::Error),
    #[error("grpc: {0:}")]
    TonicTransport(#[from] ::tonic::transport::Error),
    #[error("grpc: {0:}")]
    TonicStatus(#[from] ::tonic::Status),
    #[error("protobuf error: {0:}")]
    Protobuf(#[from] ::protobuf::error::ProtobufError),
    #[error("runtime error: {0:}")]
    Runtime(&'static str),
    #[error("error: {0:?}")]
    Keys(#[from] ::keys::Error),
    #[error("{0:}")]
    FromHex(#[from] ::hex::FromHexError),
    #[error("parsing error: {0:}")]
    ParseInt(#[from] ::std::num::ParseIntError),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Runtime(s)
    }
}
