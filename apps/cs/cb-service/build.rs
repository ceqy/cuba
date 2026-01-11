fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    
    // Generate File Descriptor Set for Reflection & Envoy
    tonic_prost_build::configure()
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile_protos(
            &["../../../protos/cs/cb/cb.proto", "../../../protos/common/common.proto"],
            &["../../../protos", "../../../third_party"],
        )?;
    Ok(())
}
