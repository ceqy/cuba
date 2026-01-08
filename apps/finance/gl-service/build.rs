fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .file_descriptor_set_path(out_dir.join("gl_service_descriptor.bin"))
        .compile_protos(
            &["../../../protos/finance/gl/gl_journal_entry.proto"],
            &["../../../protos", "../../../protos/third_party"],
        )?;
    Ok(())
}
