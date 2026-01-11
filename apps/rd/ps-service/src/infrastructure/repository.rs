use sqlx::PgPool;
use crate::domain::{ProjectBudget, CostPosting};
use anyhow::Result;
use rust_decimal::Decimal;

pub struct ProjectCostRepository {
    pool: PgPool,
}

impl ProjectCostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_budget(&self, b: &ProjectBudget) -> Result<()> {
        sqlx::query!(
            "INSERT INTO project_budgets (budget_id, wbs_element, fiscal_year, budget_amount, currency, version) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (wbs_element, fiscal_year, version) DO UPDATE SET budget_amount = $4",
            b.budget_id, b.wbs_element, b.fiscal_year, b.budget_amount, b.currency, b.version
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn save_posting(&self, p: &CostPosting) -> Result<()> {
        sqlx::query!(
            "INSERT INTO cost_postings (posting_id, wbs_element, cost_element, cost_element_type, amount, currency, posting_date, description, document_number) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            p.posting_id, p.wbs_element, p.cost_element, p.cost_element_type, p.amount, p.currency, p.posting_date, p.description, p.document_number
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_cost_report(&self, wbs: &str) -> Result<(Decimal, Decimal)> {
        // Get budget
        let budget = sqlx::query!("SELECT COALESCE(SUM(budget_amount), 0) as total FROM project_budgets WHERE wbs_element = $1", wbs)
            .fetch_one(&self.pool).await?;
        // Get actual costs
        let actual = sqlx::query!("SELECT COALESCE(SUM(amount), 0) as total FROM cost_postings WHERE wbs_element = $1", wbs)
            .fetch_one(&self.pool).await?;
        Ok((budget.total.unwrap_or_default(), actual.total.unwrap_or_default()))
    }
}
