fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic-build 0.14+ 需要使用 tonic-prost-build
    tonic_prost_build::compile_protos("../../protos/auth/auth_service.proto")?;
    Ok(())
}
