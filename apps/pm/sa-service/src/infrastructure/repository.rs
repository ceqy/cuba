use sqlx::PgPool;
use crate::domain::{SpendDimension, TimeSeriesDataPoint};
use anyhow::Result;
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct SpendRepository {
    pool: PgPool,
}

impl SpendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_spend_by_category(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SpendDimension>> {
        let rows = sqlx::query(
            "SELECT category as id, category as name, SUM(spend_amount) as spend_amount, COUNT(*) as document_count FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 AND category IS NOT NULL GROUP BY category ORDER BY spend_amount DESC LIMIT $3")
            .bind(start_date)
            .bind(end_date)
            .bind(top_n)
            .fetch_all(&self.pool).await?;
        
        use sqlx::Row;
        Ok(rows.into_iter().map(|r| SpendDimension {
            id: r.get::<Option<String>, _>("id").unwrap_or_default(),
            name: r.get::<Option<String>, _>("name").unwrap_or_default(),
            spend_amount: r.get::<Option<Decimal>, _>("spend_amount").unwrap_or_default(),
            currency: "CNY".to_string(),
            document_count: r.get::<Option<i64>, _>("document_count").unwrap_or(0),
        }).collect())
    }

    pub async fn get_spend_by_supplier(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SpendDimension>> {
        let rows = sqlx::query(
            "SELECT supplier as id, supplier as name, SUM(spend_amount) as spend_amount, COUNT(*) as document_count FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 AND supplier IS NOT NULL GROUP BY supplier ORDER BY spend_amount DESC LIMIT $3")
            .bind(start_date)
            .bind(end_date)
            .bind(top_n)
            .fetch_all(&self.pool).await?;
        
        use sqlx::Row;
        Ok(rows.into_iter().map(|r| SpendDimension {
            id: r.get::<Option<String>, _>("id").unwrap_or_default(),
            name: r.get::<Option<String>, _>("name").unwrap_or_default(),
            spend_amount: r.get::<Option<Decimal>, _>("spend_amount").unwrap_or_default(),
            currency: "CNY".to_string(),
            document_count: r.get::<Option<i64>, _>("document_count").unwrap_or(0),
        }).collect())
    }

    pub async fn get_spend_trend(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<TimeSeriesDataPoint>> {
        let rows = sqlx::query(
            "SELECT TO_CHAR(spend_date, 'YYYY-MM') as period, SUM(spend_amount) as spend_amount FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 GROUP BY TO_CHAR(spend_date, 'YYYY-MM') ORDER BY period")
            .bind(start_date)
            .bind(end_date)
            .fetch_all(&self.pool).await?;
        
        use sqlx::Row;
        Ok(rows.into_iter().map(|r| TimeSeriesDataPoint {
            period: r.get::<Option<String>, _>("period").unwrap_or_default(),
            spend_amount: r.get::<Option<Decimal>, _>("spend_amount").unwrap_or_default(),
            currency: "CNY".to_string(),
        }).collect())
    }
}
