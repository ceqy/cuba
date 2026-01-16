fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Compile GL service protos
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("gl_descriptor.bin"))
        .compile_protos(
            &["../../../protos/fi/gl/gl.proto", "../../../protos/common/common.proto"],
            &["../../../protos", "../../../third_party"],
        )?;

    // Compile COA service protos (client only)
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &["../../../protos/fi/coa/coa.proto"],
            &["../../../protos", "../../../third_party"],
        )?;

    Ok(())
}
