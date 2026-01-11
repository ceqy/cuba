use sqlx::PgPool;
use crate::domain::{SalesDimension, TimeSeriesDataPoint};
use anyhow::Result;
use chrono::NaiveDate;

pub struct SalesRepository {
    pool: PgPool,
}

impl SalesRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_sales_by_customer(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SalesDimension>> {
        let rows = sqlx::query!(
            "SELECT customer, SUM(revenue) as total_revenue, SUM(quantity_sold) as total_qty FROM sales_facts WHERE sales_date >= $1 AND sales_date <= $2 AND customer IS NOT NULL GROUP BY customer ORDER BY total_revenue DESC LIMIT $3",
            start_date, end_date, top_n
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| SalesDimension {
            id: r.customer.clone().unwrap_or_default(),
            name: r.customer.unwrap_or_default(),
            revenue: r.total_revenue.unwrap_or_default(),
            currency: "CNY".to_string(),
            quantity_sold: r.total_qty.unwrap_or_default(),
            unit: "EA".to_string(),
        }).collect())
    }

    pub async fn get_sales_by_product(&self, start_date: NaiveDate, end_date: NaiveDate, top_n: i64) -> Result<Vec<SalesDimension>> {
        let rows = sqlx::query!(
            "SELECT product, SUM(revenue) as total_revenue, SUM(quantity_sold) as total_qty FROM sales_facts WHERE sales_date >= $1 AND sales_date <= $2 AND product IS NOT NULL GROUP BY product ORDER BY total_revenue DESC LIMIT $3",
            start_date, end_date, top_n
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| SalesDimension {
            id: r.product.clone().unwrap_or_default(),
            name: r.product.unwrap_or_default(),
            revenue: r.total_revenue.unwrap_or_default(),
            currency: "CNY".to_string(),
            quantity_sold: r.total_qty.unwrap_or_default(),
            unit: "EA".to_string(),
        }).collect())
    }

    pub async fn get_sales_trend(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<TimeSeriesDataPoint>> {
        let rows = sqlx::query!(
            "SELECT TO_CHAR(sales_date, 'YYYY-MM') as period, SUM(revenue) as total_revenue FROM sales_facts WHERE sales_date >= $1 AND sales_date <= $2 GROUP BY TO_CHAR(sales_date, 'YYYY-MM') ORDER BY period",
            start_date, end_date
        ).fetch_all(&self.pool).await?;
        
        Ok(rows.into_iter().map(|r| TimeSeriesDataPoint {
            period: r.period.unwrap_or_default(),
            revenue: r.total_revenue.unwrap_or_default(),
            currency: "CNY".to_string(),
        }).collect())
    }
}
