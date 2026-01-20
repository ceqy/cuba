use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Root is ../../../
    let root = PathBuf::from("../../../");
    let protos_dir = root.join("protos");
    let third_party = root.join("third_party");

    let proto_path = protos_dir.join("sc/im/im.proto");
    let common_proto = protos_dir.join("common/common.proto");

    tonic_prost_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(&[proto_path, common_proto], &[protos_dir, third_party])?;

    Ok(())
}
