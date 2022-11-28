fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=src/network_protocol/proto/network.proto");
    protobuf_codegen::Codegen::new()
        .pure()
        .includes(["src/"])
        .input("src/network_protocol/proto/network.proto")
        .cargo_out_dir("proto")
        .run()
}
