fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/lib.rs");

    protoc_rust::Codegen::new()
        .out_dir("src")
        .includes(&["protocol", "include"])
        .inputs(&[
            "protocol/core/Tron.proto",
            "protocol/core/Contract.proto",
            "protocol/core/Discover.proto",
        ])
        .customize(protoc_rust::Customize {
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc-rust");

    protoc_rust::Codegen::new()
        .out_dir("src")
        .includes(&["protocol", "include"])
        .input("protocol/api/api.proto")
        .customize(protoc_rust::Customize {
            serde_derive: Some(true),
            ..Default::default()
        })
        .run()
        .expect("protoc-rust");

    protoc_rust_grpc::Codegen::new()
        .out_dir("src")
        .includes(&["protocol", "include"])
        .input("protocol/api/api.proto")
        .rust_protobuf(false)
        .run()
        .expect("protoc-rust-grpc");
}
