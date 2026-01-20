use std::sync::Arc;
use ta_service::api::grpc_server::TaServiceImpl;
use ta_service::api::proto::hr::ta::v1::talent_acquisition_service_server::TalentAcquisitionServiceServer;
use ta_service::application::handlers::TalentHandler;
use ta_service::infrastructure::repository::TalentRepository;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50062).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;

    let repo = Arc::new(TalentRepository::new(pool.clone()));
    let handler = Arc::new(TalentHandler::new(repo.clone()));
    let service = TaServiceImpl::new(handler, repo);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(
            ta_service::api::proto::hr::ta::v1::FILE_DESCRIPTOR_SET,
        )
        .build_v1()?;

    info!("HR Talent Acquisition Service listening on {}", addr);

    Server::builder()
        .add_service(TalentAcquisitionServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
