use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("sales_service_descriptor.bin");

    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &["../../protos/sales/sales_order_fulfillment_service.proto"],
            &["../../protos"],
        )?;

    let descriptor_set = std::fs::read(descriptor_path)?;
    let fds = prost_types::FileDescriptorSet::decode(&*descriptor_set)?;

    tonic_prost_build::configure()
        .compile_fds(fds)?;

    Ok(())
}
