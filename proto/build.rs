extern crate protoc_rust_grpc;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src",
        includes: &["protocol", "include"],
        input: &[
            "protocol/core/Tron.proto",
            "protocol/core/Contract.proto",
            "protocol/core/Discover.proto",
        ],
        rust_protobuf: true,
        ..Default::default()
    })
    .expect("protoc-rust-grpc");

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src",
        includes: &["protocol", "include"],
        input: &["protocol/api/api.proto"],
        rust_protobuf: true,
        ..Default::default()
    })
    .expect("protoc-rust-grpc");
}
