use prost::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_path = out_dir.join("auth_service_descriptor.bin");

    // 1. 使用 prost-build 生成 FileDescriptorSet (支持 include paths)
    prost_build::Config::new()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &["../../protos/auth/auth_service.proto"],
            &["../../protos"],
        )?;

    let descriptor_set = std::fs::read(descriptor_path)?;
    let fds = prost_types::FileDescriptorSet::decode(&*descriptor_set)?;

    // 2. 使用 tonic-prost-build 生成 gRPC 代码
    tonic_prost_build::configure()
        .build_server(false) // Gateway 只作为 Client
        .build_client(true)
        .compile_fds(fds)?;

    Ok(())
}
