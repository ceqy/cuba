//! AR/AP Service - Integration Tests
//!
//! 集成测试（需要数据库连接）

use std::sync::Arc;

/// Integration test helper for setting up test database
pub struct TestContext {
    pub pool: sqlx::PgPool,
}

impl TestContext {
    /// Create a new test context with a test database connection
    pub async fn new() -> Self {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_finance_test".to_string());
        
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to connect to test database");
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");
        
        Self { pool }
    }
    
    /// Clean up test data after tests
    pub async fn cleanup(&self) {
        // Truncate test tables in reverse dependency order
        sqlx::query("TRUNCATE open_items, customers, suppliers, business_partners CASCADE")
            .execute(&self.pool)
            .await
            .ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Basic connectivity test
    #[tokio::test]
    #[ignore] // Ignore by default since it requires database
    async fn test_database_connection() {
        let ctx = TestContext::new().await;
        let result: (i64,) = sqlx::query_as("SELECT 1")
            .fetch_one(&ctx.pool)
            .await
            .expect("Query failed");
        assert_eq!(result.0, 1);
    }
    
    /// Test customer CRUD operations
    #[tokio::test]
    #[ignore]
    async fn test_customer_repository() {
        let ctx = TestContext::new().await;
        ctx.cleanup().await;
        
        // Create a business partner first
        let partner_id = "TEST_BP_001";
        sqlx::query(
            "INSERT INTO business_partners (partner_id, partner_type, name_org1) 
             VALUES ($1, 'ORGANIZATION', 'Test Company')"
        )
        .bind(partner_id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to create partner");
        
        // Create a customer
        let customer_id = "TEST_C_001";
        sqlx::query(
            "INSERT INTO customers (customer_id, partner_id, company_code, reconciliation_account) 
             VALUES ($1, $2, '1000', '113100')"
        )
        .bind(customer_id)
        .bind(partner_id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to create customer");
        
        // Verify customer exists
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT customer_id FROM customers WHERE customer_id = $1"
        )
        .bind(customer_id)
        .fetch_optional(&ctx.pool)
        .await
        .expect("Query failed");
        
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, customer_id);
        
        ctx.cleanup().await;
    }
    
    /// Test open items query
    #[tokio::test]
    #[ignore]
    async fn test_open_items_balance() {
        let ctx = TestContext::new().await;
        ctx.cleanup().await;
        
        // Create test data
        let partner_id = "TEST_BP_002";
        sqlx::query(
            "INSERT INTO business_partners (partner_id, partner_type, name_org1) 
             VALUES ($1, 'ORGANIZATION', 'Test Company 2')"
        )
        .bind(partner_id)
        .execute(&ctx.pool)
        .await
        .expect("Failed to create partner");
        
        // Create open items
        for i in 1..=3 {
            sqlx::query(
                "INSERT INTO open_items (company_code, document_number, fiscal_year, line_item, 
                 account_type, partner_id, posting_date, amount, currency, open_amount)
                 VALUES ('1000', $1, 2024, 1, 'CUSTOMER', $2, CURRENT_DATE, 1000, 'CNY', 1000)"
            )
            .bind(format!("DOC{:03}", i))
            .bind(partner_id)
            .execute(&ctx.pool)
            .await
            .expect("Failed to create open item");
        }
        
        // Query balance
        let result: (rust_decimal::Decimal,) = sqlx::query_as(
            "SELECT COALESCE(SUM(open_amount), 0) FROM open_items 
             WHERE partner_id = $1 AND clearing_date IS NULL"
        )
        .bind(partner_id)
        .fetch_one(&ctx.pool)
        .await
        .expect("Query failed");
        
        assert_eq!(result.0, rust_decimal::Decimal::from(3000));
        
        ctx.cleanup().await;
    }
}

/// gRPC service integration tests
#[cfg(test)]
mod grpc_tests {
    // These would test the actual gRPC endpoints
    // Requires running the service
    
    #[tokio::test]
    #[ignore]
    async fn test_grpc_health_check() {
        // Would test gRPC health check endpoint
        // tonic_health::pb::health_client::HealthClient::connect(...)
    }
    
    #[tokio::test]
    #[ignore] 
    async fn test_grpc_reflection() {
        // Would test gRPC reflection is working
    }
}
