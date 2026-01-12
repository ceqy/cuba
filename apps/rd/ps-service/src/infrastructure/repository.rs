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
        sqlx::query(
            "INSERT INTO project_budgets (budget_id, wbs_element, fiscal_year, budget_amount, currency, version) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (wbs_element, fiscal_year, version) DO UPDATE SET budget_amount = EXCLUDED.budget_amount")
            .bind(b.budget_id)
            .bind(&b.wbs_element)
            .bind(b.fiscal_year)
            .bind(b.budget_amount)
            .bind(&b.currency)
            .bind(&b.version)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn save_posting(&self, p: &CostPosting) -> Result<()> {
        sqlx::query(
            "INSERT INTO cost_postings (posting_id, wbs_element, cost_element, cost_element_type, amount, currency, posting_date, description, document_number) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(p.posting_id)
            .bind(&p.wbs_element)
            .bind(&p.cost_element)
            .bind(&p.cost_element_type)
            .bind(p.amount)
            .bind(&p.currency)
            .bind(p.posting_date)
            .bind(&p.description)
            .bind(&p.document_number)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_cost_report(&self, wbs: &str) -> Result<(Decimal, Decimal)> {
        use sqlx::Row;
        // Get budget
        let budget_row = sqlx::query("SELECT COALESCE(SUM(budget_amount), 0) as total FROM project_budgets WHERE wbs_element = $1")
            .bind(wbs)
            .fetch_one(&self.pool).await?;
        let budget_total: Decimal = budget_row.get(0);

        // Get actual costs
        let actual_row = sqlx::query("SELECT COALESCE(SUM(amount), 0) as total FROM cost_postings WHERE wbs_element = $1")
            .bind(wbs)
            .fetch_one(&self.pool).await?;
        let actual_total: Decimal = actual_row.get(0);

        Ok((budget_total, actual_total))
    }
}
