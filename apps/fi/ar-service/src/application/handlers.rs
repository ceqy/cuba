use crate::domain::{Customer, OpenItem};
use crate::infrastructure::repository::{CustomerRepository, OpenItemRepository};
use crate::application::commands::{PostCustomerCommand, ListOpenItemsQuery};
use std::sync::Arc;
use chrono::Utc;

pub struct PostCustomerHandler {
    repo: Arc<CustomerRepository>,
}

impl PostCustomerHandler {
    pub fn new(repo: Arc<CustomerRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, cmd: PostCustomerCommand) -> anyhow::Result<Customer> {
        let customer = Customer {
            customer_id: cmd.customer_id,
            business_partner_id: cmd.business_partner_id,
            name: cmd.name,
            account_group: cmd.account_group,
            street: cmd.street,
            city: cmd.city,
            postal_code: cmd.postal_code,
            country: cmd.country,
            company_code: cmd.company_code,
            reconciliation_account: cmd.reconciliation_account,
            payment_terms: cmd.payment_terms,
            sales_organization: cmd.sales_organization,
            distribution_channel: None,
            division: None,
            order_currency: cmd.order_currency,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repo.save(&customer).await?;
        Ok(customer)
    }
}

pub struct ListOpenItemsHandler {
    repo: Arc<OpenItemRepository>,
}

impl ListOpenItemsHandler {
    pub fn new(repo: Arc<OpenItemRepository>) -> Self {
        Self { repo }
    }

    pub async fn handle(&self, query: ListOpenItemsQuery) -> anyhow::Result<Vec<OpenItem>> {
        // Simple offset pagination based on page_token (as int) if needed, defaults to page 1
        let page = query.page_token.as_ref().and_then(|t| t.parse::<i64>().ok()).unwrap_or(1);
        let limit = query.page_size as i64;
        let offset = (page - 1) * limit;

        self.repo.list_by_customer(&query.customer_id, query.include_cleared, limit, offset).await
    }
}
