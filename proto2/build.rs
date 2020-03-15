fn main() {
    prost_build::compile_protos(
        &[
            "proto/common.proto",
            "proto/discover.proto",
            "proto/handshake.proto",
            "proto/chain.proto",
            "proto/channel.proto",
            "proto/contract.proto",
        ],
        &["proto/"],
    )
    .unwrap();
}
