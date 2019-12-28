fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src",
        includes: &["protocol", "include"],
        input: &[
            "protocol/core/Tron.proto",
            "protocol/core/Contract.proto",
            "protocol/core/Discover.proto",
        ],
        #[cfg(feature = "with-serde")]
        customize: protoc_rust::Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc-rust");

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src",
        includes: &["protocol", "include"],
        input: &["protocol/api/api.proto"],
        #[cfg(feature = "with-serde")]
        customize: protoc_rust::Customize {
            serde_derive: Some(true),
            ..Default::default()
        },
    })
    .expect("protoc-rust");

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src",
        includes: &["protocol", "include"],
        input: &["protocol/api/api.proto"],
        rust_protobuf: false,
        ..Default::default()
    })
    .expect("protoc-rust-grpc");
}
