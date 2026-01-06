fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .compile_protos(
            &["../../protos/sales/sales_order_fulfillment_service.proto"],
            &["../../protos"],
        )?;
    Ok(())
}
