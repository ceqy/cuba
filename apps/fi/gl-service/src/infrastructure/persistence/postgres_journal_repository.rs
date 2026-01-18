use async_trait::async_trait;
use sqlx::Row;
use uuid::Uuid;
use std::error::Error;
use std::str::FromStr;
use chrono::{NaiveDate, Datelike};
use crate::domain::aggregates::journal_entry::{JournalEntry, LineItem, PostingStatus, DebitCredit};
use crate::domain::repositories::JournalRepository;
use cuba_database::DbPool;

/// Calculate fiscal period from posting date (1-12 for months, 13-16 for special periods)
fn calculate_fiscal_period(posting_date: NaiveDate) -> i32 {
    posting_date.month() as i32
}

pub struct PostgresJournalRepository {
    pool: DbPool,
}

impl PostgresJournalRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JournalRepository for PostgresJournalRepository {
    async fn save(&self, entry: &JournalEntry) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        // Upsert Header
        sqlx::query(
            r#"
            INSERT INTO journal_entries (
                id, document_number, company_code, fiscal_year, fiscal_period, 
                posting_date, document_date, status, currency, reference, 
                created_at, posted_at, tenant_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (id) DO UPDATE SET
                document_number = EXCLUDED.document_number,
                status = EXCLUDED.status,
                posted_at = EXCLUDED.posted_at
            "#
        )
        .bind(entry.id)
        .bind(&entry.document_number)
        .bind(&entry.company_code)
        .bind(entry.fiscal_year)
        .bind(calculate_fiscal_period(entry.posting_date))
        .bind(entry.posting_date)
        .bind(entry.document_date)
        .bind(entry.status.to_string())
        .bind(&entry.currency)
        .bind(&entry.reference)
        .bind(entry.created_at)
        .bind(entry.posted_at)
        .bind(&entry.tenant_id)
        .execute(&mut *tx)
        .await?;

        // Delete existing lines
        sqlx::query("DELETE FROM journal_entry_lines WHERE journal_entry_id = $1")
            .bind(entry.id)
            .execute(&mut *tx)
            .await?;

        // Insert Lines
        for line in &entry.lines {
            sqlx::query(
                r#"
                INSERT INTO journal_entry_lines (
                    id, journal_entry_id, line_number, account_id,
                    debit_credit, amount, local_amount,
                    cost_center, profit_center, line_text,
                    special_gl_indicator, ledger, ledger_type, ledger_amount
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                "#
            )
            .bind(line.id)
            .bind(entry.id)
            .bind(line.line_number)
            .bind(&line.account_id)
            .bind(line.debit_credit.as_char().to_string())
            .bind(line.amount)
            .bind(line.local_amount)
            .bind(&line.cost_center)
            .bind(&line.profit_center)
            .bind(&line.text)
            .bind(line.special_gl_indicator.to_sap_code())
            .bind(&line.ledger)
            .bind(i32::from(line.ledger_type))
            .bind(line.ledger_amount)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> Result<Option<JournalEntry>, Box<dyn Error + Send + Sync>> {
        let header_row = sqlx::query(
            r#"
            SELECT 
                id, document_number, company_code, fiscal_year, posting_date, document_date, 
                status, currency, reference, created_at, posted_at, tenant_id
            FROM journal_entries 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = header_row {
            let entry_id: Uuid = row.get("id");
            let status_str: String = row.get("status");
            let status = PostingStatus::from_str(&status_str).unwrap_or(PostingStatus::Draft);

            let lines_rows = sqlx::query(
                r#"
                SELECT
                    id, line_number, account_id, debit_credit, amount, local_amount,
                    cost_center, profit_center, line_text,
                    special_gl_indicator, ledger, ledger_type, ledger_amount
                FROM journal_entry_lines
                WHERE journal_entry_id = $1
                ORDER BY line_number ASC
                "#
            )
            .bind(entry_id)
            .fetch_all(&self.pool)
            .await?;

            let lines = lines_rows.into_iter().map(|l| {
                let dc_str: String = l.get("debit_credit");
                let dc = DebitCredit::from_char(dc_str.chars().next().unwrap()).unwrap();

                // 读取特殊总账标识
                let special_gl_code: String = l.get::<Option<String>, _>("special_gl_indicator").unwrap_or_default();
                let special_gl_indicator = crate::domain::aggregates::journal_entry::SpecialGlType::from_sap_code(&special_gl_code);

                // 读取并行会计字段
                let ledger: String = l.get::<Option<String>, _>("ledger").unwrap_or_else(|| "0L".to_string());
                let ledger_type_int: i32 = l.get::<Option<i32>, _>("ledger_type").unwrap_or(1);
                let ledger_type = crate::domain::aggregates::journal_entry::LedgerType::from(ledger_type_int);

                LineItem {
                    id: l.get("id"),
                    line_number: l.get("line_number"),
                    account_id: l.get("account_id"),
                    debit_credit: dc,
                    amount: l.get("amount"),
                    local_amount: l.get("local_amount"),
                    cost_center: l.get("cost_center"),
                    profit_center: l.get("profit_center"),
                    text: l.get("line_text"),
                    special_gl_indicator,
                    ledger,
                    ledger_type,
                    ledger_amount: l.get("ledger_amount"),
                }
            }).collect();

            Ok(Some(JournalEntry {
                id: entry_id,
                document_number: row.get("document_number"),
                company_code: row.get("company_code"),
                fiscal_year: row.get("fiscal_year"),
                posting_date: row.get("posting_date"),
                document_date: row.get("document_date"),
                currency: row.get("currency"),
                reference: row.get("reference"),
                status,
                lines,
                created_at: row.get("created_at"),
                posted_at: row.get("posted_at"),
                tenant_id: row.get("tenant_id"),
                // 并行会计默认值（从数据库读取时暂时使用默认值）
                ledger_group: None,
                default_ledger: "0L".to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn search(&self, company_code: &str, status: Option<&str>, page: u64, page_size: u64) -> Result<Vec<JournalEntry>, Box<dyn Error + Send + Sync>> {
         let offset = (page.max(1) - 1) * page_size;
         
         let mut query_builder = sqlx::QueryBuilder::new(
             "SELECT id FROM journal_entries WHERE company_code = "
         );
         query_builder.push_bind(company_code);
         
         if let Some(s) = status {
             query_builder.push(" AND status = ");
             query_builder.push_bind(s.to_uppercase()); // Ensure uppercase match
         }
         
         query_builder.push(" ORDER BY created_at DESC LIMIT ");
         query_builder.push_bind(page_size as i64);
         query_builder.push(" OFFSET ");
         query_builder.push_bind(offset as i64);
         
         let query = query_builder.build();
         let rows = query.fetch_all(&self.pool).await?;
         
         let mut entries = Vec::new();
         for row in rows {
             let id: Uuid = row.get("id");
             if let Some(entry) = self.find_by_id(&id).await? {
                 entries.push(entry);
             }
         }
         
         Ok(entries)
    }

    async fn count(&self, company_code: &str, status: Option<&str>) -> Result<i64, Box<dyn Error + Send + Sync>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT COUNT(*) as count FROM journal_entries WHERE company_code = "
        );
        query_builder.push_bind(company_code);

        if let Some(s) = status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(s.to_uppercase());
        }

        let query = query_builder.build();
        let row = query.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");

        Ok(count)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut tx = self.pool.begin().await?;

        // Delete line items first (foreign key constraint)
        sqlx::query("DELETE FROM journal_entry_line_items WHERE journal_entry_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Delete journal entry
        sqlx::query("DELETE FROM journal_entries WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
    }
}
