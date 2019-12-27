use grpc::ClientStub;
use proto::api_grpc::WalletClient;
use std::net::ToSocketAddrs;
use std::sync::Arc;

// const RPC_HOST: &str = "grpc.trongrid.io:50051";
const RPC_HOST: &str = "grpc.shasta.trongrid.io:50051";

pub fn new_grpc_client() -> WalletClient {
    let host = RPC_HOST
        .to_socket_addrs()
        .expect("resolve host")
        .next()
        .expect("host resolve result");

    let grpc_client = Arc::new(
        grpc::Client::new_plain(&host.ip().to_string(), host.port(), Default::default()).expect("grpc client"),
    );
    WalletClient::with_client(grpc_client)
}
