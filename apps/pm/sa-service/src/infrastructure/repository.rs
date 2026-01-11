use sqlx::PgPool;
use crate::domain::{SpendDimension, TimeSeriesDataPoint};
use anyhow::Result;
use chrono::NaiveDate;

pub struct SpendRepository {
    pool: PgPool,
}

impl SpendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_spend_by_category(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SpendDimension>> {
        let rows = sqlx::query!(
            "SELECT category, SUM(spend_amount) as total_spend, COUNT(*) as doc_count FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 AND category IS NOT NULL GROUP BY category ORDER BY total_spend DESC LIMIT $3",
            start_date, end_date, top_n
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| SpendDimension {
            id: r.category.clone().unwrap_or_default(),
            name: r.category.unwrap_or_default(),
            spend_amount: r.total_spend.unwrap_or_default(),
            currency: "CNY".to_string(),
            document_count: r.doc_count.unwrap_or(0),
        }).collect())
    }

    pub async fn get_spend_by_supplier(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SpendDimension>> {
        let rows = sqlx::query!(
            "SELECT supplier, SUM(spend_amount) as total_spend, COUNT(*) as doc_count FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 AND supplier IS NOT NULL GROUP BY supplier ORDER BY total_spend DESC LIMIT $3",
            start_date, end_date, top_n
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| SpendDimension {
            id: r.supplier.clone().unwrap_or_default(),
            name: r.supplier.unwrap_or_default(),
            spend_amount: r.total_spend.unwrap_or_default(),
            currency: "CNY".to_string(),
            document_count: r.doc_count.unwrap_or(0),
        }).collect())
    }

    pub async fn get_spend_trend(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<TimeSeriesDataPoint>> {
        let rows = sqlx::query!(
            "SELECT TO_CHAR(spend_date, 'YYYY-MM') as period, SUM(spend_amount) as total_spend FROM spend_facts WHERE spend_date >= $1 AND spend_date <= $2 GROUP BY TO_CHAR(spend_date, 'YYYY-MM') ORDER BY period",
            start_date, end_date
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| TimeSeriesDataPoint {
            period: r.period.unwrap_or_default(),
            spend_amount: r.total_spend.unwrap_or_default(),
            currency: "CNY".to_string(),
        }).collect())
    }
}
