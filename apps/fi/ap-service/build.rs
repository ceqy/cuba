fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    
    // Generate File Descriptor Set for Reflection & Envoy
    // Include gl.proto for gRPC client to call GL service
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true) // Need client for GL service
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(
            &[
                "../../../protos/fi/ap/ap.proto",
                "../../../protos/fi/gl/gl.proto",
                "../../../protos/common/common.proto",
            ],
            &["../../../protos", "../../../third_party"],
        )?;
    Ok(())
}
