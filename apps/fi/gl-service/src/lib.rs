//! GL Service - 总账凭证管理服务
//!
//! 提供复式记账凭证的创建、查询、过账、冲销等功能。

pub mod api;
pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export for convenience
pub use api::grpc_server::GlServiceImpl;
pub use infrastructure::persistence::postgres_journal_repository::PostgresJournalRepository;

use cuba_database::DbPool;
use domain::services::AccountValidationService;
use infrastructure::clients::CoaClient;
use std::sync::Arc;

/// Factory function to create and wire up the GL Service with all its dependencies.
///
/// This encapsulates the complex initialization logic and makes main.rs cleaner.
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `coa_endpoint` - Optional COA service endpoint for account validation
///
/// # Returns
/// Fully initialized GlServiceImpl ready to serve gRPC requests
pub async fn create_gl_service(
    pool: DbPool,
    coa_endpoint: Option<String>,
) -> Result<GlServiceImpl<PostgresJournalRepository>, Box<dyn std::error::Error>> {
    use application::handlers::*;

    // Initialize Repository
    let journal_repo = Arc::new(PostgresJournalRepository::new(pool.clone()));

    // Initialize COA client (optional - gracefully degrade if unavailable)
    let account_validation = if let Some(endpoint) = coa_endpoint {
        match CoaClient::connect(&endpoint).await {
            Ok(coa_client) => {
                tracing::info!("Connected to COA service at {}", endpoint);
                let chart_code =
                    std::env::var("CHART_OF_ACCOUNTS").unwrap_or_else(|_| "CN01".to_string());
                Some(Arc::new(AccountValidationService::new(
                    coa_client, chart_code,
                )))
            },
            Err(e) => {
                tracing::warn!(
                    "Failed to connect to COA service: {}. Account validation will be skipped.",
                    e
                );
                None
            },
        }
    } else {
        None
    };

    // Initialize Handlers
    let mut create_handler = CreateJournalEntryHandler::new(journal_repo.clone());
    if let Some(validator) = account_validation {
        create_handler = create_handler.with_account_validation(validator);
    }
    let create_handler = Arc::new(create_handler);

    let get_handler = Arc::new(GetJournalEntryHandler::new(journal_repo.clone()));
    let list_handler = Arc::new(ListJournalEntriesHandler::new(journal_repo.clone()));
    let post_handler = Arc::new(PostJournalEntryHandler::new(journal_repo.clone()));
    let reverse_handler = Arc::new(ReverseJournalEntryHandler::new(journal_repo.clone()));
    let delete_handler = Arc::new(DeleteJournalEntryHandler::new(journal_repo.clone()));
    let park_handler = Arc::new(ParkJournalEntryHandler::new(journal_repo.clone()));
    let update_handler = Arc::new(UpdateJournalEntryHandler::new(journal_repo.clone()));

    // Assemble Service
    Ok(GlServiceImpl::new(
        create_handler,
        get_handler,
        list_handler,
        post_handler,
        reverse_handler,
        delete_handler,
        park_handler,
        update_handler,
    ))
}
