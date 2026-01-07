fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic-build 0.14+ 需要使用 tonic-prost-build
    // Include third_party protos for google.api.http annotations
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(
            &["../../protos/auth/auth_service.proto"],
            &["../../protos", "../../protos/third_party"],
        )?;
    Ok(())
}
