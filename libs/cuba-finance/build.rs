fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Generate protobuf client for GL service
    tonic_prost_build::configure()
        .build_server(false) // Only build client
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(
            &[
                "../../protos/fi/gl/gl.proto",
                "../../protos/common/common.proto",
            ],
            &["../../protos", "../../third_party"],
        )?;
    Ok(())
}
