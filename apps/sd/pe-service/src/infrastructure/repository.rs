use sqlx::PgPool;
use crate::domain::PricingCondition;
use anyhow::Result;

pub struct PricingRepository {
    pool: PgPool,
}

impl PricingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &PricingCondition) -> Result<()> {
        sqlx::query!(
            "INSERT INTO pricing_conditions (condition_id, condition_type, material, customer, sales_org, amount, currency, valid_from, valid_to) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (condition_id) DO UPDATE SET amount = EXCLUDED.amount, valid_from = EXCLUDED.valid_from, valid_to = EXCLUDED.valid_to",
            c.condition_id, c.condition_type, c.material, c.customer, c.sales_org, c.amount, c.currency, c.valid_from, c.valid_to
        ).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_conditions(&self, material: &str, customer: Option<&str>, sales_org: &str, pricing_date: chrono::NaiveDate) -> Result<Vec<PricingCondition>> {
        let rows = sqlx::query!(
            "SELECT * FROM pricing_conditions WHERE (material = $1 OR material IS NULL) AND (customer = $2 OR customer IS NULL) AND sales_org = $3 AND (valid_from IS NULL OR valid_from <= $4) AND (valid_to IS NULL OR valid_to >= $4)",
            material, customer, sales_org, pricing_date
        ).fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| PricingCondition {
            condition_id: r.condition_id,
            condition_type: r.condition_type,
            material: r.material,
            customer: r.customer,
            sales_org: r.sales_org,
            amount: r.amount,
            currency: r.currency.unwrap_or_default(),
            valid_from: r.valid_from,
            valid_to: r.valid_to,
            created_at: r.created_at,
        }).collect())
    }
}
