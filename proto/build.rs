fn main() {
    const PROTOS: &[&str] = &[
        "proto/common.proto",
        "proto/discovery.proto",
        "proto/chain.proto",
        "proto/channel.proto",
        "proto/contract.proto",
        "proto/state.proto",
    ];

    for f in PROTOS {
        println!("cargo:rerun-if-changed={}", f);
    }
    prost_build::Config::new()
        .type_attribute("proto.common.SmartContract.ABI", "#[derive(serde::Serialize)]")
        .type_attribute("proto.common.SmartContract.ABI.Entry", "#[derive(serde::Serialize)]")
        .type_attribute("proto.common.SmartContract.ABI.Param", "#[derive(serde::Serialize)]")
        .out_dir("src")
        .compile_protos(PROTOS, &["proto/"])
        .unwrap();
}
