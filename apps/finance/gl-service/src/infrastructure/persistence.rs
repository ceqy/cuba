//! Infrastructure persistence layer for GL Service
//!
//! PostgreSQL 仓储实现

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use rust_decimal::Decimal;

use crate::domain::entities::{JournalEntry, JournalEntryHeader, JournalEntryLine, ClearingStatus, TaxLineItem, ClearingDocument};
use crate::domain::repository::{
    JournalEntryFilter, JournalEntryRepository, PagedResult, Pagination,
};
use crate::domain::value_objects::{
    DocumentNumber, FiscalPeriod, JournalEntryId, Account, AccountType, MonetaryAmount, DebitCreditIndicator, CostObjects, TaxType
};
use crate::domain::rules::JournalEntryStatus;

// ============================================================================
// Database Models
// ============================================================================

#[derive(sqlx::FromRow)]
struct DbHeader {
    id: Uuid,
    company_code: String,
    fiscal_year: i32,
    document_number: String,
    document_type: String,
    document_date: NaiveDate,
    posting_date: NaiveDate,
    fiscal_period: i32,
    currency: String,
    exchange_rate: Decimal,
    local_currency: String,
    header_text: Option<String>,
    reference_document: Option<String>,
    status: String,
    document_origin: String,
    ledger: String,
    version: i64,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct DbLine {
    id: Uuid,
    header_id: Uuid,
    line_number: i32,
    gl_account: String,
    account_type: String,
    customer_number: Option<String>,
    vendor_number: Option<String>,
    amount_doc_currency: Decimal,
    debit_credit_indicator: String,
    amount_local_currency: Option<Decimal>,
    cost_center: Option<String>,
    profit_center: Option<String>,
    internal_order: Option<String>,
    wbs_element: Option<String>,
    business_area: Option<String>,
    functional_area: Option<String>,
    segment: Option<String>,
    line_text: Option<String>,
    assignment: Option<String>,
    clearing_status: String,
    tax_code: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct DbTaxLine {
    header_id: Uuid,
    line_number: i32,
    tax_code: String,
    tax_type: String,
    tax_rate: Decimal,
    tax_amount_doc: Decimal,
    debit_credit_indicator: String,
}

// ============================================================================
// Repository Implementation
// ============================================================================

pub struct PgJournalEntryRepository {
    pool: Arc<PgPool>,
}

impl PgJournalEntryRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    fn map_db_line_to_domain(&self, l: DbLine, currency: &str) -> anyhow::Result<JournalEntryLine> {
        let acc_type = match l.account_type.as_str() {
            "S" => AccountType::GeneralLedger,
            "D" => AccountType::Customer,
            "K" => AccountType::Vendor,
            _ => AccountType::GeneralLedger,
        };

        let dc = DebitCreditIndicator::from_str(&l.debit_credit_indicator)
            .ok_or_else(|| anyhow::anyhow!("Invalid D/C indicator in DB"))?;

        let amount = MonetaryAmount::new(l.amount_doc_currency, currency, dc)
            .map_err(|e| anyhow::anyhow!(e))?;

        let account = match acc_type {
            AccountType::Customer => Account::customer(&l.gl_account, l.customer_number.as_deref().unwrap_or(""))?,
            AccountType::Vendor => Account::vendor(&l.gl_account, l.vendor_number.as_deref().unwrap_or(""))?,
            _ => Account::gl_account(&l.gl_account)?,
        };

        Ok(JournalEntryLine {
            line_number: l.line_number,
            account,
            amount,
            amount_local: l.amount_local_currency,
            cost_objects: CostObjects {
                cost_center: l.cost_center,
                profit_center: l.profit_center,
                internal_order: l.internal_order,
                wbs_element: l.wbs_element,
                business_area: l.business_area,
                functional_area: l.functional_area,
                segment: l.segment,
            },
            line_text: l.line_text,
            assignment: l.assignment,
            tax_code: l.tax_code,
            clearing_status: ClearingStatus::from_str(&l.clearing_status).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl JournalEntryRepository for PgJournalEntryRepository {
    async fn save(&self, entry: &mut JournalEntry) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. 保存抬头
        let header = entry.header();
        let doc_num = entry.document_number().map(|d| d.number().to_string()).unwrap_or_default();
        
        sqlx::query(
            r#"
            INSERT INTO journal_entry_headers (
                id, company_code, fiscal_year, document_number, document_type,
                document_date, posting_date, fiscal_period, currency, exchange_rate,
                local_currency, header_text, reference_document, status, ledger,
                version, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
            ON CONFLICT (id) DO UPDATE SET
                document_number = EXCLUDED.document_number,
                status = EXCLUDED.status,
                updated_at = NOW(),
                version = journal_entry_headers.version + 1
            "#,
        )
        .bind(entry.id().as_uuid())
        .bind(&header.company_code)
        .bind(header.fiscal_period.year())
        .bind(doc_num)
        .bind(&header.document_type)
        .bind(header.document_date)
        .bind(header.posting_date)
        .bind(header.fiscal_period.period())
        .bind(&header.currency)
        .bind(header.exchange_rate)
        .bind(&header.local_currency)
        .bind(&header.header_text)
        .bind(&header.reference_document)
        .bind(entry.status().as_str())
        .bind(&header.ledger)
        .bind(entry.version() as i64)
        .bind(header.created_by)
        .bind(header.created_at)
        .bind(header.updated_at)
        .execute(&mut *tx)
        .await?;

        // 2. 保存行项目 (简单处理：先删后插，或者按 ID 更新)
        sqlx::query("DELETE FROM journal_entry_lines WHERE header_id = $1")
            .bind(entry.id().as_uuid())
            .execute(&mut *tx)
            .await?;

        for line in entry.lines() {
            let account_type = line.account.account_type().as_str();
            let dc_indicator = line.amount.dc_indicator().as_str();
            
            sqlx::query(
                r#"
                INSERT INTO journal_entry_lines (
                    id, header_id, line_number, gl_account, account_type,
                    customer_number, vendor_number, amount_doc_currency,
                    debit_credit_indicator, amount_local_currency,
                    cost_center, profit_center, internal_order, wbs_element,
                    business_area, functional_area, segment, line_text,
                    assignment, clearing_status, tax_code
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(entry.id().as_uuid())
            .bind(line.line_number)
            .bind(line.account.get_gl_account())
            .bind(account_type)
            .bind(line.account.subledger_account().filter(|_| account_type == "D"))
            .bind(line.account.subledger_account().filter(|_| account_type == "K"))
            .bind(line.amount.amount())
            .bind(dc_indicator)
            .bind(line.amount_local)
            .bind(&line.cost_objects.cost_center)
            .bind(&line.cost_objects.profit_center)
            .bind(&line.cost_objects.internal_order)
            .bind(&line.cost_objects.wbs_element)
            .bind(&line.cost_objects.business_area)
            .bind(&line.cost_objects.functional_area)
            .bind(&line.cost_objects.segment)
            .bind(&line.line_text)
            .bind(&line.assignment)
            .bind(line.clearing_status.as_str())
            .bind(&line.tax_code)
            .execute(&mut *tx)
            .await?;
        }

        // 3. 保存税务行项目
        sqlx::query("DELETE FROM journal_entry_tax WHERE header_id = $1")
            .bind(entry.id().as_uuid())
            .execute(&mut *tx)
            .await?;

        for tax in entry.tax_items() {
            sqlx::query(
                r#"
                INSERT INTO journal_entry_tax (
                    header_id, line_number, tax_code, tax_type, 
                    tax_rate, tax_amount_doc, debit_credit_indicator
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#
            )
            .bind(entry.id().as_uuid())
            .bind(tax.line_number)
            .bind(&tax.tax_code)
            .bind(tax.tax_type.as_str())
            .bind(tax.tax_rate)
            .bind(tax.tax_amount)
            .bind(tax.dc_indicator.as_str())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<Option<JournalEntry>> {
        let header = sqlx::query_as::<_, DbHeader>("SELECT * FROM journal_entry_headers WHERE id = $1")
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;

        if let Some(h) = header {
            let lines = sqlx::query_as::<_, DbLine>("SELECT * FROM journal_entry_lines WHERE header_id = $1 ORDER BY line_number")
                .bind(id)
                .fetch_all(&*self.pool)
                .await?;

            let domain_header = JournalEntryHeader {
                company_code: h.company_code.clone(),
                document_type: h.document_type,
                document_date: h.document_date,
                posting_date: h.posting_date,
                fiscal_period: FiscalPeriod::new(h.fiscal_year, h.fiscal_period)
                    .map_err(|e| anyhow::anyhow!(e))?,
                currency: h.currency.clone(),
                exchange_rate: h.exchange_rate,
                local_currency: h.local_currency,
                header_text: h.header_text,
                reference_document: h.reference_document,
                ledger: h.ledger,
                created_by: h.created_by,
                created_at: h.created_at,
                updated_at: h.updated_at,
            };

            let mut domain_lines = Vec::new();
            for l in lines {
                domain_lines.push(self.map_db_line_to_domain(l, &h.currency)?);
            }

            let tax_rows = sqlx::query_as::<_, DbTaxLine>(
                "SELECT * FROM journal_entry_tax WHERE header_id = $1 ORDER BY line_number"
            )
            .bind(id)
            .fetch_all(&*self.pool)
            .await?;

            let mut domain_tax_items = Vec::new();
            for t in tax_rows {
                domain_tax_items.push(TaxLineItem {
                    line_number: t.line_number,
                    tax_code: t.tax_code,
                    tax_type: TaxType::from_str(&t.tax_type).unwrap_or(TaxType::Input),
                    tax_rate: t.tax_rate,
                    tax_base_amount: Decimal::ZERO, // TODO
                    tax_amount: t.tax_amount_doc,
                    dc_indicator: DebitCreditIndicator::from_str(&t.debit_credit_indicator).unwrap_or(DebitCreditIndicator::Debit),
                });
            }

            let status = JournalEntryStatus::from_str(&h.status).unwrap_or(JournalEntryStatus::Draft);
            let doc_num = if h.document_number.is_empty() {
                None
            } else {
                Some(DocumentNumber::new(&h.company_code, h.fiscal_year, &h.document_number)?)
            };

            let entry = JournalEntry::reconstruct(
                JournalEntryId::from_uuid(h.id),
                doc_num,
                domain_header,
                domain_lines,
                domain_tax_items,
                status,
                h.version as u64,
            );

            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    async fn find_by_document_number(
        &self,
        doc_number: &DocumentNumber,
    ) -> anyhow::Result<Option<JournalEntry>> {
        let header = sqlx::query_as::<_, DbHeader>(
            "SELECT * FROM journal_entry_headers WHERE company_code = $1 AND fiscal_year = $2 AND document_number = $3",
        )
        .bind(doc_number.company_code())
        .bind(doc_number.fiscal_year())
        .bind(doc_number.number())
        .fetch_optional(&*self.pool)
        .await?;

        if let Some(h) = header {
            self.find_by_id(&h.id).await
        } else {
            Ok(None)
        }
    }

    async fn find_all(
        &self,
        filter: JournalEntryFilter,
        pagination: Pagination,
    ) -> anyhow::Result<PagedResult<JournalEntry>> {
        let mut query = "SELECT id FROM journal_entry_headers WHERE 1=1".to_string();
        let mut params = Vec::new();
        let mut param_idx = 1;

        if let Some(cc) = filter.company_code {
            query.push_str(&format!(" AND company_code = ${}", param_idx));
            params.push(cc);
            param_idx += 1;
        }

        if let Some(fy) = filter.fiscal_year {
            query.push_str(&format!(" AND fiscal_year = ${}", param_idx));
            params.push(fy.to_string());
            param_idx += 1;
        }

        if let Some(s) = filter.status {
            query.push_str(&format!(" AND status = ${}", param_idx));
            params.push(s.as_str().to_string());
            param_idx += 1;
        }

        // Count total
        let count_query = query.replace("SELECT id", "SELECT COUNT(*)");
        let mut sql_count = sqlx::query_as::<_, (i64,)>(&count_query);
        for p in &params {
            sql_count = sql_count.bind(p);
        }
        let total_count: (i64,) = sql_count.fetch_one(&*self.pool).await?;

        // Paging
        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));
        
        let mut sql_query = sqlx::query_as::<_, (Uuid,)>(&query);
        for p in &params {
            sql_query = sql_query.bind(p);
        }
        sql_query = sql_query.bind(pagination.page_size as i64).bind(((pagination.page - 1) * pagination.page_size) as i64);

        let ids: Vec<(Uuid,)> = sql_query.fetch_all(&*self.pool).await?;
        
        let mut items = Vec::new();
        for (id,) in ids {
            if let Some(entry) = self.find_by_id(&id).await? {
                items.push(entry);
            }
        }

        Ok(PagedResult {
            items,
            total_count: total_count.0 as u64,
            page: pagination.page,
            page_size: pagination.page_size,
        })
    }

    async fn delete(&self, id: &Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM journal_entry_headers WHERE id = $1")
            .bind(id)
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn next_document_number(
        &self,
        company_code: &str,
        fiscal_year: i32,
    ) -> anyhow::Result<String> {
        let mut tx = self.pool.begin().await?;

        // 尝试获取并更新序列号，使用 FOR UPDATE 保证并发安全
        let row: Option<(i64,)> = sqlx::query_as(
            "UPDATE document_sequences 
             SET current_value = current_value + 1 
             WHERE company_code = $1 AND fiscal_year = $2 AND document_type = 'SA' 
             RETURNING current_value"
        )
        .bind(company_code)
        .bind(fiscal_year)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((next_val,)) = row {
            tx.commit().await?;
            Ok(next_val.to_string())
        } else {
            // 如果不存在，初始化一个默认值 (1000000000 + 1)
            sqlx::query(
                "INSERT INTO document_sequences (company_code, fiscal_year, document_type, current_value) 
                 VALUES ($1, $2, 'SA', 1000000001)"
            )
            .bind(company_code)
            .bind(fiscal_year)
            .execute(&mut *tx)
            .await?;
            
            tx.commit().await?;
            Ok("1000000001".to_string())
        }
    }

    async fn find_lines_by_ids(&self, ids: &[Uuid]) -> anyhow::Result<Vec<JournalEntryLine>> {
        let mut domain_lines = Vec::new();
        for id in ids {
            // 先获取行记录
            let line: Option<DbLine> = sqlx::query_as(
                "SELECT * FROM journal_entry_lines WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(&*self.pool)
            .await?;
            
            if let Some(l) = line {
                // 获取对应的凭证头以取得币种
                let currency_row: Option<(String,)> = sqlx::query_as(
                    "SELECT currency FROM journal_entry_headers WHERE id = $1"
                )
                .bind(&l.header_id)
                .fetch_optional(&*self.pool)
                .await?;
                
                let currency = currency_row.map(|(c,)| c).unwrap_or_else(|| "CNY".to_string());
                domain_lines.push(self.map_db_line_to_domain(l, &currency)?);
            }
        }
        Ok(domain_lines)
    }

    async fn save_clearing_document(&self, clearing_doc: &ClearingDocument) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        // 1. 保存清账抬头
        sqlx::query(
            "INSERT INTO clearing_documents (id, clearing_number, company_code, fiscal_year, clearing_date, clearing_amount, currency, clearing_type, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(clearing_doc.id)
        .bind(&clearing_doc.clearing_number)
        .bind(&clearing_doc.company_code)
        .bind(clearing_doc.fiscal_year)
        .bind(clearing_doc.clearing_date)
        .bind(clearing_doc.clearing_amount)
        .bind(&clearing_doc.currency)
        .bind(&clearing_doc.clearing_type)
        .bind(clearing_doc.created_by)
        .execute(&mut *tx)
        .await?;

        // 2. 保存清账明细并更新原始行项目
        for item in &clearing_doc.items {
            sqlx::query(
                "INSERT INTO clearing_items (clearing_document_id, journal_entry_line_id, cleared_amount)
                 VALUES ($1, $2, $3)"
            )
            .bind(clearing_doc.id)
            .bind(item.journal_entry_line_id)
            .bind(item.cleared_amount)
            .execute(&mut *tx)
            .await?;

            sqlx::query(
                "UPDATE journal_entry_lines SET clearing_status = 'CLEARED' WHERE id = $1"
            )
            .bind(item.journal_entry_line_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
