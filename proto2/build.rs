fn main() {
    prost_build::compile_protos(
        &[
            "proto/common.proto",
            "proto/discovery.proto",
            "proto/chain.proto",
            "proto/channel.proto",
            "proto/contract.proto",
        ],
        &["proto/"],
    )
    .unwrap();
}
