use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    let root = PathBuf::from("../../../");
    let protos_dir = root.join("protos");
    let third_party = root.join("third_party");

    // Protos to compile - include gl.proto for GL client
    let ap_proto = protos_dir.join("fi/ap/ap.proto");
    let gl_proto = protos_dir.join("fi/gl/gl.proto");
    let common_proto = protos_dir.join("common/common.proto");

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true) // Need client for GL service
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(
            &[ap_proto, gl_proto, common_proto],
            &[protos_dir, third_party],
        )?;

    Ok(())
}
