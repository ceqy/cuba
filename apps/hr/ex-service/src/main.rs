use tonic::transport::Server;
use tracing::info;
use std::sync::Arc;
use ex_service::api::grpc_server::ExServiceImpl;
use ex_service::api::proto::hr::ex::v1::employee_experience_service_server::EmployeeExperienceServiceServer;
use ex_service::infrastructure::repository::ExperienceRepository;
use ex_service::application::handlers::ExperienceHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bootstrap Service
    let context = cuba_service::ServiceBootstrapper::run(50063).await?;
    let pool = context.db_pool;
    let addr = context.addr;

    let migrator = sqlx::migrate!("./migrations");
    cuba_database::run_migrations(&pool, &migrator).await?;
    
    let repo = Arc::new(ExperienceRepository::new(pool.clone()));
    let handler = Arc::new(ExperienceHandler::new(repo.clone()));
    let service = ExServiceImpl::new(handler, repo);
    
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(ex_service::api::proto::hr::ex::v1::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    info!("HR Employee Experience Service listening on {}", addr);
    
    Server::builder()
        .add_service(EmployeeExperienceServiceServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}
