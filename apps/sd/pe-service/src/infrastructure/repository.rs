use crate::domain::PricingCondition;
use anyhow::Result;
use sqlx::PgPool;

pub struct PricingRepository {
    pool: PgPool,
}

impl PricingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, c: &PricingCondition) -> Result<()> {
        sqlx::query(
            "INSERT INTO pricing_conditions (condition_id, condition_type, material, customer, sales_org, amount, currency, valid_from, valid_to) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (condition_id) DO UPDATE SET amount = EXCLUDED.amount, valid_from = EXCLUDED.valid_from, valid_to = EXCLUDED.valid_to")
            .bind(c.condition_id)
            .bind(&c.condition_type)
            .bind(&c.material)
            .bind(&c.customer)
            .bind(&c.sales_org)
            .bind(c.amount)
            .bind(&c.currency)
            .bind(c.valid_from)
            .bind(c.valid_to)
        .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn find_conditions(
        &self,
        material: &str,
        customer: Option<&str>,
        sales_org: &str,
        pricing_date: chrono::NaiveDate,
    ) -> Result<Vec<PricingCondition>> {
        let conditions = sqlx::query_as::<_, PricingCondition>(
            "SELECT condition_id, condition_type, material, customer, sales_org, amount, COALESCE(currency, '') as currency, valid_from, valid_to, created_at FROM pricing_conditions WHERE (material = $1 OR material IS NULL) AND (customer = $2 OR customer IS NULL) AND sales_org = $3 AND (valid_from IS NULL OR valid_from <= $4) AND (valid_to IS NULL OR valid_to >= $4)"
        )
        .bind(material)
        .bind(customer)
        .bind(sales_org)
        .bind(pricing_date)
        .fetch_all(&self.pool).await?;
        Ok(conditions)
    }
}
