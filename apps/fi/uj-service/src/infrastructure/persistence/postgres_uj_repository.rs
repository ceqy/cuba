use crate::domain::{
    aggregates::{AccountType, SourceModule, UniversalJournalEntry},
    repositories::{
        AggregationResult, PaginationParams, PaginationResponse, RepositoryError,
        UniversalJournalFilter, UniversalJournalRepository,
    },
};
use async_trait::async_trait;
use chrono::{Datelike, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};
use std::collections::HashMap;

pub struct PostgresUniversalJournalRepository {
    pool: PgPool,
}

impl PostgresUniversalJournalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn source_module_code(source_module: SourceModule) -> &'static str {
        match source_module {
            SourceModule::GL => "GL",
            SourceModule::AP => "AP",
            SourceModule::AR => "AR",
            SourceModule::AA => "AA",
            SourceModule::MM => "MM",
            SourceModule::SD => "SD",
            SourceModule::CO => "CO",
            SourceModule::TR => "TR",
            SourceModule::Unspecified => "UNSPECIFIED",
        }
    }

    fn source_module_allowed(filter: &UniversalJournalFilter, module: &str) -> bool {
        match &filter.source_modules {
            Some(modules) => modules.iter().any(|m| m.eq_ignore_ascii_case(module)),
            None => true,
        }
    }

    fn entry_matches_filter(
        entry: &UniversalJournalEntry,
        filter: &UniversalJournalFilter,
    ) -> bool {
        if let Some(ledgers) = &filter.ledgers {
            if !ledgers.iter().any(|l| l == &entry.ledger) {
                return false;
            }
        }

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.iter().any(|c| c == &entry.company_code) {
                return false;
            }
        }

        if let Some(year_from) = filter.fiscal_year_from {
            if entry.fiscal_year < year_from {
                return false;
            }
        }
        if let Some(year_to) = filter.fiscal_year_to {
            if entry.fiscal_year > year_to {
                return false;
            }
        }

        if let Some(document_types) = &filter.document_types {
            if !document_types.iter().any(|t| t == &entry.document_type) {
                return false;
            }
        }

        if let Some(date_from) = filter.posting_date_from {
            if entry.posting_date < date_from {
                return false;
            }
        }
        if let Some(date_to) = filter.posting_date_to {
            if entry.posting_date > date_to {
                return false;
            }
        }

        if let Some(date_from) = filter.document_date_from {
            if entry.document_date < date_from {
                return false;
            }
        }
        if let Some(date_to) = filter.document_date_to {
            if entry.document_date > date_to {
                return false;
            }
        }

        if let Some(gl_accounts) = &filter.gl_accounts {
            if !gl_accounts.iter().any(|g| g == &entry.gl_account) {
                return false;
            }
        }

        if let Some(business_partners) = &filter.business_partners {
            let partner = entry.business_partner.as_deref().unwrap_or("");
            if !business_partners.iter().any(|b| b == partner) {
                return false;
            }
        }

        if let Some(cost_centers) = &filter.cost_centers {
            let cost_center = entry.cost_center.as_deref().unwrap_or("");
            if !cost_centers.iter().any(|c| c == cost_center) {
                return false;
            }
        }

        if let Some(profit_centers) = &filter.profit_centers {
            let profit_center = entry.profit_center.as_deref().unwrap_or("");
            if !profit_centers.iter().any(|p| p == profit_center) {
                return false;
            }
        }

        if let Some(segments) = &filter.segments {
            let segment = entry.segment.as_deref().unwrap_or("");
            if !segments.iter().any(|s| s == segment) {
                return false;
            }
        }

        if let Some(business_areas) = &filter.business_areas {
            let business_area = entry.business_area.as_deref().unwrap_or("");
            if !business_areas.iter().any(|b| b == business_area) {
                return false;
            }
        }

        if let Some(source_modules) = &filter.source_modules {
            let source_code = Self::source_module_code(entry.source_module);
            if !source_modules.iter().any(|s| s == source_code) {
                return false;
            }
        }

        if filter.only_open_items && entry.clearing_document.is_some() {
            return false;
        }
        if filter.only_cleared_items && entry.clearing_document.is_none() {
            return false;
        }

        if let Some(special_gls) = &filter.special_gl_indicators {
            let gl = entry.special_gl_indicator.as_deref().unwrap_or("");
            if !special_gls.iter().any(|s| s == gl) {
                return false;
            }
        }

        if let Some(search_text) = &filter.search_text {
            let search = search_text.to_lowercase();
            let haystack = [
                entry.document_number.as_str(),
                entry.header_text.as_deref().unwrap_or(""),
                entry.item_text.as_deref().unwrap_or(""),
            ]
            .join(" ")
            .to_lowercase();
            if !haystack.contains(&search) {
                return false;
            }
        }

        true
    }

    /// 构建查询 WHERE 子句
    fn build_where_clause(&self, filter: &UniversalJournalFilter) -> (String, Vec<String>) {
        let mut conditions = Vec::new();
        let mut params = Vec::new();
        let mut param_index = 1;

        // 分类账过滤
        if let Some(ledgers) = &filter.ledgers {
            if !ledgers.is_empty() {
                let placeholders: Vec<String> = ledgers
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("${}", idx)
                    })
                    .collect();
                conditions.push(format!("ledger IN ({})", placeholders.join(", ")));
                params.extend(ledgers.clone());
            }
        }

        // 公司代码过滤
        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                let placeholders: Vec<String> = company_codes
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("${}", idx)
                    })
                    .collect();
                conditions.push(format!("company_code IN ({})", placeholders.join(", ")));
                params.extend(company_codes.clone());
            }
        }

        // 会计年度过滤
        if let Some(year_from) = filter.fiscal_year_from {
            conditions.push(format!("fiscal_year >= ${}", param_index));
            params.push(year_from.to_string());
            param_index += 1;
        }
        if let Some(year_to) = filter.fiscal_year_to {
            conditions.push(format!("fiscal_year <= ${}", param_index));
            params.push(year_to.to_string());
            param_index += 1;
        }

        // 过账日期过滤
        if let Some(date_from) = filter.posting_date_from {
            conditions.push(format!("posting_date >= ${}", param_index));
            params.push(date_from.to_string());
            param_index += 1;
        }
        if let Some(date_to) = filter.posting_date_to {
            conditions.push(format!("posting_date <= ${}", param_index));
            params.push(date_to.to_string());
            param_index += 1;
        }

        // 总账科目过滤
        if let Some(gl_accounts) = &filter.gl_accounts {
            if !gl_accounts.is_empty() {
                let placeholders: Vec<String> = gl_accounts
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("${}", idx)
                    })
                    .collect();
                conditions.push(format!("gl_account IN ({})", placeholders.join(", ")));
                params.extend(gl_accounts.clone());
            }
        }

        // 来源模块过滤
        if let Some(source_modules) = &filter.source_modules {
            if !source_modules.is_empty() {
                let placeholders: Vec<String> = source_modules
                    .iter()
                    .map(|_| {
                        let idx = param_index;
                        param_index += 1;
                        format!("${}", idx)
                    })
                    .collect();
                conditions.push(format!("source_module IN ({})", placeholders.join(", ")));
                params.extend(source_modules.clone());
            }
        }

        // 清账状态过滤
        if filter.only_open_items {
            conditions.push("clearing_document IS NULL".to_string());
        }
        if filter.only_cleared_items {
            conditions.push("clearing_document IS NOT NULL".to_string());
        }

        // 全文搜索
        if let Some(search_text) = &filter.search_text {
            conditions.push(format!(
                "(header_text ILIKE ${} OR item_text ILIKE ${})",
                param_index,
                param_index + 1
            ));
            let search_pattern = format!("%{}%", search_text);
            params.push(search_pattern.clone());
            params.push(search_pattern);
            param_index += 2;
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        (where_clause, params)
    }

    /// 从数据库行映射到领域模型
    fn map_row_to_entry(
        &self,
        row: &sqlx::postgres::PgRow,
    ) -> Result<UniversalJournalEntry, RepositoryError> {
        let source_module_str: String = row.try_get("source_module")?;
        let source_module = match source_module_str.as_str() {
            "GL" => SourceModule::GL,
            "AP" => SourceModule::AP,
            "AR" => SourceModule::AR,
            "AA" => SourceModule::AA,
            "MM" => SourceModule::MM,
            "SD" => SourceModule::SD,
            "CO" => SourceModule::CO,
            "TR" => SourceModule::TR,
            _ => SourceModule::Unspecified,
        };

        let account_type_str: String = row.try_get("account_type")?;
        let account_type = match account_type_str.as_str() {
            "D" | "GL" => AccountType::GL,
            "K" | "Vendor" => AccountType::Vendor,
            "D" | "Customer" => AccountType::Customer,
            "A" | "Asset" => AccountType::Asset,
            "M" | "Material" => AccountType::Material,
            _ => AccountType::Unspecified,
        };

        let extension_fields: Option<serde_json::Value> = row.try_get("extension_fields")?;
        let extension_fields_map = extension_fields
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        Ok(UniversalJournalEntry {
            ledger: row.try_get("ledger")?,
            company_code: row.try_get("company_code")?,
            fiscal_year: row.try_get("fiscal_year")?,
            document_number: row.try_get("document_number")?,
            document_line: row.try_get("document_line")?,
            document_type: row.try_get("document_type")?,
            document_date: row.try_get("document_date")?,
            posting_date: row.try_get("posting_date")?,
            fiscal_period: row.try_get("fiscal_period")?,
            reference_document: row.try_get("reference_document")?,
            header_text: row.try_get("header_text")?,
            document_currency: row.try_get("document_currency")?,
            exchange_rate: row.try_get("exchange_rate")?,
            logical_system: row.try_get("logical_system")?,
            transaction_code: row.try_get("transaction_code")?,
            posting_key: row.try_get("posting_key")?,
            debit_credit_indicator: row.try_get("debit_credit_indicator")?,
            account_type,
            gl_account: row.try_get("gl_account")?,
            business_partner: row.try_get("business_partner")?,
            material: row.try_get("material")?,
            plant: row.try_get("plant")?,
            item_text: row.try_get("item_text")?,
            assignment_number: row.try_get("assignment_number")?,
            amount_in_document_currency: row.try_get("amount_in_document_currency")?,
            amount_in_local_currency: row.try_get("amount_in_local_currency")?,
            amount_in_group_currency: row.try_get("amount_in_group_currency")?,
            amount_in_global_currency: row.try_get("amount_in_global_currency")?,
            amount_in_ledger_currency: row.try_get("amount_in_ledger_currency")?,
            quantity: row.try_get("quantity")?,
            quantity_unit: row.try_get("quantity_unit")?,
            cost_center: row.try_get("cost_center")?,
            profit_center: row.try_get("profit_center")?,
            segment: row.try_get("segment")?,
            functional_area: row.try_get("functional_area")?,
            business_area: row.try_get("business_area")?,
            controlling_area: row.try_get("controlling_area")?,
            internal_order: row.try_get("internal_order")?,
            wbs_element: row.try_get("wbs_element")?,
            sales_order: row.try_get("sales_order")?,
            sales_order_item: row.try_get("sales_order_item")?,
            tax_code: row.try_get("tax_code")?,
            tax_jurisdiction: row.try_get("tax_jurisdiction")?,
            tax_amount: row.try_get("tax_amount")?,
            clearing_document: row.try_get("clearing_document")?,
            clearing_date: row.try_get("clearing_date")?,
            baseline_date: row.try_get("baseline_date")?,
            due_date: row.try_get("due_date")?,
            payment_terms: row.try_get("payment_terms")?,
            payment_method: row.try_get("payment_method")?,
            payment_block: row.try_get("payment_block")?,
            house_bank: row.try_get("house_bank")?,
            special_gl_indicator: row.try_get("special_gl_indicator")?,
            reference_document_number: row.try_get("reference_document_number")?,
            reference_fiscal_year: row.try_get("reference_fiscal_year")?,
            reference_line_item: row.try_get("reference_line_item")?,
            reference_document_type: row.try_get("reference_document_type")?,
            transaction_type: row.try_get("transaction_type")?,
            reference_transaction_type: row.try_get("reference_transaction_type")?,
            reference_key_1: row.try_get("reference_key_1")?,
            reference_key_2: row.try_get("reference_key_2")?,
            reference_key_3: row.try_get("reference_key_3")?,
            financial_area: row.try_get("financial_area")?,
            consolidation_unit: row.try_get("consolidation_unit")?,
            partner_company: row.try_get("partner_company")?,
            trading_partner: row.try_get("trading_partner")?,
            local_currency: row.try_get("local_currency")?,
            group_currency: row.try_get("group_currency")?,
            global_currency: row.try_get("global_currency")?,
            amount_in_object_currency: row.try_get("amount_in_object_currency")?,
            amount_in_profit_center_currency: row.try_get("amount_in_profit_center_currency")?,
            dunning_key: row.try_get("dunning_key")?,
            dunning_block: row.try_get("dunning_block")?,
            last_dunning_date: row.try_get("last_dunning_date")?,
            dunning_level: row.try_get("dunning_level")?,
            discount_days_1: row.try_get("discount_days_1")?,
            discount_days_2: row.try_get("discount_days_2")?,
            net_payment_days: row.try_get("net_payment_days")?,
            discount_percent_1: row.try_get("discount_percent_1")?,
            discount_percent_2: row.try_get("discount_percent_2")?,
            discount_amount: row.try_get("discount_amount")?,
            sending_cost_center: row.try_get("sending_cost_center")?,
            partner_profit_center: row.try_get("partner_profit_center")?,
            sending_financial_area: row.try_get("sending_financial_area")?,
            account_assignment: row.try_get("account_assignment")?,
            local_account: row.try_get("local_account")?,
            data_source: row.try_get("data_source")?,
            split_method: row.try_get("split_method")?,
            manual_split: row.try_get("manual_split")?,
            created_by: row.try_get("created_by")?,
            created_at: row.try_get("created_at")?,
            changed_by: row.try_get("changed_by")?,
            changed_at: row.try_get("changed_at")?,
            source_module,
            extension_fields: extension_fields_map,
        })
    }

    async fn fetch_ap_open_items(
        &self,
        filter: &UniversalJournalFilter,
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        if !Self::source_module_allowed(filter, "AP") {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                o.document_number,
                o.company_code,
                o.fiscal_year,
                o.line_item_number,
                o.posting_date,
                o.due_date,
                o.baseline_date,
                o.currency,
                o.original_amount,
                o.open_amount,
                o.clearing_document,
                o.clearing_date,
                o.reference_document,
                o.item_text,
                o.payment_block,
                o.ledger,
                o.special_gl_indicator,
                o.payment_method,
                o.payment_terms,
                o.dunning_block,
                o.dunning_level,
                o.transaction_type,
                o.reference_transaction_type,
                o.created_at,
                i.document_date,
                i.document_type,
                s.supplier_id AS business_partner,
                s.reconciliation_account AS gl_account
            FROM open_items o
            LEFT JOIN invoices i
                ON i.document_number = o.document_number
                AND i.company_code = o.company_code
                AND i.fiscal_year = o.fiscal_year
            LEFT JOIN suppliers s ON o.supplier_id = s.id
            "#,
        );

        let mut has_conditions = false;
        let mut push_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !has_conditions {
                builder.push(" WHERE ");
                has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("o.company_code IN (");
                let mut separated = query_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(ledgers) = &filter.ledgers {
            if !ledgers.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("o.ledger IN (");
                let mut separated = query_builder.separated(", ");
                for ledger in ledgers {
                    separated.push_bind(ledger);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(year_from) = filter.fiscal_year_from {
            push_condition(&mut query_builder);
            query_builder.push("o.fiscal_year >= ");
            query_builder.push_bind(year_from);
        }

        if let Some(year_to) = filter.fiscal_year_to {
            push_condition(&mut query_builder);
            query_builder.push("o.fiscal_year <= ");
            query_builder.push_bind(year_to);
        }

        if let Some(date_from) = filter.posting_date_from {
            push_condition(&mut query_builder);
            query_builder.push("o.posting_date >= ");
            query_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.posting_date_to {
            push_condition(&mut query_builder);
            query_builder.push("o.posting_date <= ");
            query_builder.push_bind(date_to);
        }

        if filter.only_open_items {
            push_condition(&mut query_builder);
            query_builder.push("o.clearing_document IS NULL");
        } else if filter.only_cleared_items {
            push_condition(&mut query_builder);
            query_builder.push("o.clearing_document IS NOT NULL");
        }

        let rows = query_builder.build().fetch_all(&self.pool).await?;
        let entries = rows
            .into_iter()
            .map(|row| -> Result<UniversalJournalEntry, RepositoryError> {
                let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
                let posting_date: chrono::NaiveDate = row.try_get("posting_date")?;
                let document_date: Option<chrono::NaiveDate> = row.try_get("document_date")?;
                let document_type: Option<String> = row.try_get("document_type")?;
                let ledger: Option<String> = row.try_get("ledger")?;
                let business_partner: Option<String> = row.try_get("business_partner")?;
                let gl_account: Option<String> = row.try_get("gl_account")?;

                Ok(UniversalJournalEntry {
                    ledger: ledger.unwrap_or_else(|| "0L".to_string()),
                    company_code: row.try_get("company_code")?,
                    fiscal_year: row.try_get("fiscal_year")?,
                    document_number: row.try_get("document_number")?,
                    document_line: row.try_get("line_item_number")?,
                    document_type: document_type.unwrap_or_else(|| "KR".to_string()),
                    document_date: document_date.unwrap_or(posting_date),
                    posting_date,
                    fiscal_period: posting_date.month() as i32,
                    reference_document: row.try_get("reference_document")?,
                    header_text: None,
                    document_currency: row.try_get("currency")?,
                    exchange_rate: None,
                    logical_system: None,
                    transaction_code: None,
                    posting_key: "31".to_string(),
                    debit_credit_indicator: "H".to_string(),
                    account_type: AccountType::Vendor,
                    gl_account: gl_account.unwrap_or_else(|| "000000".to_string()),
                    business_partner,
                    material: None,
                    plant: None,
                    item_text: row.try_get("item_text")?,
                    assignment_number: None,
                    amount_in_document_currency: row.try_get("open_amount")?,
                    amount_in_local_currency: row.try_get("open_amount")?,
                    amount_in_group_currency: None,
                    amount_in_global_currency: None,
                    amount_in_ledger_currency: None,
                    quantity: None,
                    quantity_unit: None,
                    cost_center: None,
                    profit_center: None,
                    segment: None,
                    functional_area: None,
                    business_area: None,
                    controlling_area: None,
                    internal_order: None,
                    wbs_element: None,
                    sales_order: None,
                    sales_order_item: None,
                    tax_code: None,
                    tax_jurisdiction: None,
                    tax_amount: None,
                    clearing_document: row.try_get("clearing_document")?,
                    clearing_date: row.try_get("clearing_date")?,
                    baseline_date: row.try_get("baseline_date")?,
                    due_date: row.try_get("due_date")?,
                    payment_terms: row.try_get("payment_terms")?,
                    payment_method: row.try_get("payment_method")?,
                    payment_block: row.try_get("payment_block")?,
                    house_bank: None,
                    special_gl_indicator: row.try_get("special_gl_indicator")?,
                    reference_document_number: None,
                    reference_fiscal_year: None,
                    reference_line_item: None,
                    reference_document_type: None,
                    transaction_type: row.try_get("transaction_type")?,
                    reference_transaction_type: row.try_get("reference_transaction_type")?,
                    reference_key_1: None,
                    reference_key_2: None,
                    reference_key_3: None,
                    financial_area: None,
                    consolidation_unit: None,
                    partner_company: None,
                    trading_partner: None,
                    local_currency: row.try_get("currency")?,
                    group_currency: None,
                    global_currency: None,
                    amount_in_object_currency: None,
                    amount_in_profit_center_currency: None,
                    dunning_key: None,
                    dunning_block: row.try_get("dunning_block")?,
                    last_dunning_date: None,
                    dunning_level: row.try_get("dunning_level")?,
                    discount_days_1: None,
                    discount_days_2: None,
                    net_payment_days: None,
                    discount_percent_1: None,
                    discount_percent_2: None,
                    discount_amount: None,
                    sending_cost_center: None,
                    partner_profit_center: None,
                    sending_financial_area: None,
                    account_assignment: None,
                    local_account: None,
                    data_source: None,
                    split_method: None,
                    manual_split: false,
                    created_by: "AP".to_string(),
                    created_at: created_at.naive_utc(),
                    changed_by: None,
                    changed_at: None,
                    source_module: SourceModule::AP,
                    extension_fields: HashMap::new(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    async fn fetch_ar_open_items(
        &self,
        filter: &UniversalJournalFilter,
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        if !Self::source_module_allowed(filter, "AR") {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                o.document_number,
                o.company_code,
                o.fiscal_year,
                o.line_item_number,
                o.doc_type,
                o.posting_date,
                o.document_date,
                o.due_date,
                o.baseline_date,
                o.currency,
                o.original_amount,
                o.open_amount,
                o.clearing_document,
                o.clearing_date,
                o.reference_document,
                o.item_text,
                o.payment_block,
                o.ledger,
                o.special_gl_indicator,
                o.payment_method,
                o.payment_terms,
                o.dunning_block,
                o.dunning_level,
                o.transaction_type,
                o.reference_transaction_type,
                o.created_at,
                c.customer_id AS business_partner,
                c.reconciliation_account AS gl_account
            FROM open_items o
            LEFT JOIN customers c ON o.customer_id = c.customer_id
            "#,
        );

        let mut has_conditions = false;
        let mut push_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !has_conditions {
                builder.push(" WHERE ");
                has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("o.company_code IN (");
                let mut separated = query_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(ledgers) = &filter.ledgers {
            if !ledgers.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("o.ledger IN (");
                let mut separated = query_builder.separated(", ");
                for ledger in ledgers {
                    separated.push_bind(ledger);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(year_from) = filter.fiscal_year_from {
            push_condition(&mut query_builder);
            query_builder.push("o.fiscal_year >= ");
            query_builder.push_bind(year_from);
        }

        if let Some(year_to) = filter.fiscal_year_to {
            push_condition(&mut query_builder);
            query_builder.push("o.fiscal_year <= ");
            query_builder.push_bind(year_to);
        }

        if let Some(date_from) = filter.posting_date_from {
            push_condition(&mut query_builder);
            query_builder.push("o.posting_date >= ");
            query_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.posting_date_to {
            push_condition(&mut query_builder);
            query_builder.push("o.posting_date <= ");
            query_builder.push_bind(date_to);
        }

        if filter.only_open_items {
            push_condition(&mut query_builder);
            query_builder.push("o.clearing_document IS NULL");
        } else if filter.only_cleared_items {
            push_condition(&mut query_builder);
            query_builder.push("o.clearing_document IS NOT NULL");
        }

        let rows = query_builder.build().fetch_all(&self.pool).await?;
        let entries = rows
            .into_iter()
            .map(|row| -> Result<UniversalJournalEntry, RepositoryError> {
                let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
                let posting_date: chrono::NaiveDate = row.try_get("posting_date")?;
                let document_date: chrono::NaiveDate = row.try_get("document_date")?;
                let ledger: Option<String> = row.try_get("ledger")?;
                let business_partner: Option<String> = row.try_get("business_partner")?;
                let gl_account: Option<String> = row.try_get("gl_account")?;

                Ok(UniversalJournalEntry {
                    ledger: ledger.unwrap_or_else(|| "0L".to_string()),
                    company_code: row.try_get("company_code")?,
                    fiscal_year: row.try_get("fiscal_year")?,
                    document_number: row.try_get("document_number")?,
                    document_line: row.try_get("line_item_number")?,
                    document_type: row.try_get::<String, _>("doc_type")?,
                    document_date,
                    posting_date,
                    fiscal_period: posting_date.month() as i32,
                    reference_document: row.try_get("reference_document")?,
                    header_text: None,
                    document_currency: row.try_get("currency")?,
                    exchange_rate: None,
                    logical_system: None,
                    transaction_code: None,
                    posting_key: "01".to_string(),
                    debit_credit_indicator: "S".to_string(),
                    account_type: AccountType::Customer,
                    gl_account: gl_account.unwrap_or_else(|| "000000".to_string()),
                    business_partner,
                    material: None,
                    plant: None,
                    item_text: row.try_get("item_text")?,
                    assignment_number: None,
                    amount_in_document_currency: row.try_get("open_amount")?,
                    amount_in_local_currency: row.try_get("open_amount")?,
                    amount_in_group_currency: None,
                    amount_in_global_currency: None,
                    amount_in_ledger_currency: None,
                    quantity: None,
                    quantity_unit: None,
                    cost_center: None,
                    profit_center: None,
                    segment: None,
                    functional_area: None,
                    business_area: None,
                    controlling_area: None,
                    internal_order: None,
                    wbs_element: None,
                    sales_order: None,
                    sales_order_item: None,
                    tax_code: None,
                    tax_jurisdiction: None,
                    tax_amount: None,
                    clearing_document: row.try_get("clearing_document")?,
                    clearing_date: row.try_get("clearing_date")?,
                    baseline_date: row.try_get("baseline_date")?,
                    due_date: row.try_get("due_date")?,
                    payment_terms: row.try_get("payment_terms")?,
                    payment_method: row.try_get("payment_method")?,
                    payment_block: row.try_get("payment_block")?,
                    house_bank: None,
                    special_gl_indicator: row.try_get("special_gl_indicator")?,
                    reference_document_number: None,
                    reference_fiscal_year: None,
                    reference_line_item: None,
                    reference_document_type: None,
                    transaction_type: row.try_get("transaction_type")?,
                    reference_transaction_type: row.try_get("reference_transaction_type")?,
                    reference_key_1: None,
                    reference_key_2: None,
                    reference_key_3: None,
                    financial_area: None,
                    consolidation_unit: None,
                    partner_company: None,
                    trading_partner: None,
                    local_currency: row.try_get("currency")?,
                    group_currency: None,
                    global_currency: None,
                    amount_in_object_currency: None,
                    amount_in_profit_center_currency: None,
                    dunning_key: None,
                    dunning_block: row.try_get("dunning_block")?,
                    last_dunning_date: None,
                    dunning_level: row.try_get("dunning_level")?,
                    discount_days_1: None,
                    discount_days_2: None,
                    net_payment_days: None,
                    discount_percent_1: None,
                    discount_percent_2: None,
                    discount_amount: None,
                    sending_cost_center: None,
                    partner_profit_center: None,
                    sending_financial_area: None,
                    account_assignment: None,
                    local_account: None,
                    data_source: None,
                    split_method: None,
                    manual_split: false,
                    created_by: "AR".to_string(),
                    created_at: created_at.naive_utc(),
                    changed_by: None,
                    changed_at: None,
                    source_module: SourceModule::AR,
                    extension_fields: HashMap::new(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    async fn fetch_co_allocations(
        &self,
        filter: &UniversalJournalFilter,
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        if !Self::source_module_allowed(filter, "CO") {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                r.run_id,
                r.controlling_area,
                r.fiscal_year,
                r.fiscal_period,
                r.allocation_type,
                r.created_at,
                s.sender_id,
                s.sender_object,
                s.sent_amount,
                s.currency,
                s.cost_center,
                s.profit_center,
                s.segment,
                s.internal_order,
                s.wbs_element
            FROM allocation_senders s
            JOIN allocation_runs r ON r.run_id = s.run_id
            "#,
        );

        let mut has_conditions = false;
        let mut push_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !has_conditions {
                builder.push(" WHERE ");
                has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("r.controlling_area IN (");
                let mut separated = query_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(year_from) = filter.fiscal_year_from {
            push_condition(&mut query_builder);
            query_builder.push("r.fiscal_year >= ");
            query_builder.push_bind(year_from);
        }

        if let Some(year_to) = filter.fiscal_year_to {
            push_condition(&mut query_builder);
            query_builder.push("r.fiscal_year <= ");
            query_builder.push_bind(year_to);
        }

        let sender_rows = query_builder.build().fetch_all(&self.pool).await?;

        let mut entries = Vec::new();
        for row in sender_rows {
            let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let posting_date = created_at.date_naive();
            let amount: Decimal = row.try_get("sent_amount")?;
            let currency: String = row.try_get("currency")?;

            let sender_object: Option<String> = row.try_get("sender_object")?;

            entries.push(UniversalJournalEntry {
                ledger: "0L".to_string(),
                company_code: row.try_get("controlling_area")?,
                fiscal_year: row.try_get("fiscal_year")?,
                document_number: row.try_get::<uuid::Uuid, _>("sender_id")?.to_string(),
                document_line: 1,
                document_type: "CO".to_string(),
                document_date: posting_date,
                posting_date,
                fiscal_period: row.try_get("fiscal_period")?,
                reference_document: None,
                header_text: None,
                document_currency: currency.clone(),
                exchange_rate: None,
                logical_system: None,
                transaction_code: None,
                posting_key: "50".to_string(),
                debit_credit_indicator: "H".to_string(),
                account_type: AccountType::GL,
                gl_account: "COALLOC".to_string(),
                business_partner: None,
                material: None,
                plant: None,
                item_text: sender_object,
                assignment_number: None,
                amount_in_document_currency: amount,
                amount_in_local_currency: amount,
                amount_in_group_currency: None,
                amount_in_global_currency: None,
                amount_in_ledger_currency: None,
                quantity: None,
                quantity_unit: None,
                cost_center: row.try_get("cost_center")?,
                profit_center: row.try_get("profit_center")?,
                segment: row.try_get("segment")?,
                functional_area: None,
                business_area: None,
                controlling_area: Some(row.try_get("controlling_area")?),
                internal_order: row.try_get("internal_order")?,
                wbs_element: row.try_get("wbs_element")?,
                sales_order: None,
                sales_order_item: None,
                tax_code: None,
                tax_jurisdiction: None,
                tax_amount: None,
                clearing_document: None,
                clearing_date: None,
                baseline_date: None,
                due_date: None,
                payment_terms: None,
                payment_method: None,
                payment_block: None,
                house_bank: None,
                special_gl_indicator: None,
                reference_document_number: None,
                reference_fiscal_year: None,
                reference_line_item: None,
                reference_document_type: None,
                transaction_type: Some(row.try_get("allocation_type")?),
                reference_transaction_type: None,
                reference_key_1: None,
                reference_key_2: None,
                reference_key_3: None,
                financial_area: None,
                consolidation_unit: None,
                partner_company: None,
                trading_partner: None,
                local_currency: currency,
                group_currency: None,
                global_currency: None,
                amount_in_object_currency: None,
                amount_in_profit_center_currency: None,
                dunning_key: None,
                dunning_block: None,
                last_dunning_date: None,
                dunning_level: None,
                discount_days_1: None,
                discount_days_2: None,
                net_payment_days: None,
                discount_percent_1: None,
                discount_percent_2: None,
                discount_amount: None,
                sending_cost_center: None,
                partner_profit_center: None,
                sending_financial_area: None,
                account_assignment: None,
                local_account: None,
                data_source: None,
                split_method: None,
                manual_split: false,
                created_by: "CO".to_string(),
                created_at: created_at.naive_utc(),
                changed_by: None,
                changed_at: None,
                source_module: SourceModule::CO,
                extension_fields: HashMap::new(),
            });
        }

        let mut receiver_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                r.run_id,
                r.controlling_area,
                r.fiscal_year,
                r.fiscal_period,
                r.allocation_type,
                r.created_at,
                rcv.receiver_id,
                rcv.receiver_object,
                rcv.received_amount,
                rcv.currency,
                rcv.cost_center,
                rcv.profit_center,
                rcv.segment,
                rcv.internal_order,
                rcv.wbs_element
            FROM allocation_receivers rcv
            JOIN allocation_runs r ON r.run_id = rcv.run_id
            "#,
        );

        let mut receiver_has_conditions = false;
        let mut push_receiver_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !receiver_has_conditions {
                builder.push(" WHERE ");
                receiver_has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_receiver_condition(&mut receiver_builder);
                receiver_builder.push("r.controlling_area IN (");
                let mut separated = receiver_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(year_from) = filter.fiscal_year_from {
            push_receiver_condition(&mut receiver_builder);
            receiver_builder.push("r.fiscal_year >= ");
            receiver_builder.push_bind(year_from);
        }

        if let Some(year_to) = filter.fiscal_year_to {
            push_receiver_condition(&mut receiver_builder);
            receiver_builder.push("r.fiscal_year <= ");
            receiver_builder.push_bind(year_to);
        }

        let receiver_rows = receiver_builder.build().fetch_all(&self.pool).await?;
        for row in receiver_rows {
            let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let posting_date = created_at.date_naive();
            let amount: Decimal = row.try_get("received_amount")?;
            let currency: String = row.try_get("currency")?;

            let receiver_object: Option<String> = row.try_get("receiver_object")?;

            entries.push(UniversalJournalEntry {
                ledger: "0L".to_string(),
                company_code: row.try_get("controlling_area")?,
                fiscal_year: row.try_get("fiscal_year")?,
                document_number: row.try_get::<uuid::Uuid, _>("receiver_id")?.to_string(),
                document_line: 1,
                document_type: "CO".to_string(),
                document_date: posting_date,
                posting_date,
                fiscal_period: row.try_get("fiscal_period")?,
                reference_document: None,
                header_text: None,
                document_currency: currency.clone(),
                exchange_rate: None,
                logical_system: None,
                transaction_code: None,
                posting_key: "40".to_string(),
                debit_credit_indicator: "S".to_string(),
                account_type: AccountType::GL,
                gl_account: "COALLOC".to_string(),
                business_partner: None,
                material: None,
                plant: None,
                item_text: receiver_object,
                assignment_number: None,
                amount_in_document_currency: amount,
                amount_in_local_currency: amount,
                amount_in_group_currency: None,
                amount_in_global_currency: None,
                amount_in_ledger_currency: None,
                quantity: None,
                quantity_unit: None,
                cost_center: row.try_get("cost_center")?,
                profit_center: row.try_get("profit_center")?,
                segment: row.try_get("segment")?,
                functional_area: None,
                business_area: None,
                controlling_area: Some(row.try_get("controlling_area")?),
                internal_order: row.try_get("internal_order")?,
                wbs_element: row.try_get("wbs_element")?,
                sales_order: None,
                sales_order_item: None,
                tax_code: None,
                tax_jurisdiction: None,
                tax_amount: None,
                clearing_document: None,
                clearing_date: None,
                baseline_date: None,
                due_date: None,
                payment_terms: None,
                payment_method: None,
                payment_block: None,
                house_bank: None,
                special_gl_indicator: None,
                reference_document_number: None,
                reference_fiscal_year: None,
                reference_line_item: None,
                reference_document_type: None,
                transaction_type: Some(row.try_get("allocation_type")?),
                reference_transaction_type: None,
                reference_key_1: None,
                reference_key_2: None,
                reference_key_3: None,
                financial_area: None,
                consolidation_unit: None,
                partner_company: None,
                trading_partner: None,
                local_currency: currency,
                group_currency: None,
                global_currency: None,
                amount_in_object_currency: None,
                amount_in_profit_center_currency: None,
                dunning_key: None,
                dunning_block: None,
                last_dunning_date: None,
                dunning_level: None,
                discount_days_1: None,
                discount_days_2: None,
                net_payment_days: None,
                discount_percent_1: None,
                discount_percent_2: None,
                discount_amount: None,
                sending_cost_center: None,
                partner_profit_center: None,
                sending_financial_area: None,
                account_assignment: None,
                local_account: None,
                data_source: None,
                split_method: None,
                manual_split: false,
                created_by: "CO".to_string(),
                created_at: created_at.naive_utc(),
                changed_by: None,
                changed_at: None,
                source_module: SourceModule::CO,
                extension_fields: HashMap::new(),
            });
        }

        Ok(entries)
    }

    async fn fetch_tr_entries(
        &self,
        filter: &UniversalJournalFilter,
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        if !Self::source_module_allowed(filter, "TR") {
            return Ok(Vec::new());
        }

        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                st.transaction_id,
                st.value_date,
                st.amount,
                st.currency,
                st.memo,
                st.partner_name,
                st.transaction_type,
                bs.statement_id,
                bs.company_code,
                bs.house_bank,
                bs.bank_account,
                bs.created_at
            FROM statement_transactions st
            JOIN bank_statements bs ON bs.statement_id = st.statement_id
            "#,
        );

        let mut has_conditions = false;
        let mut push_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !has_conditions {
                builder.push(" WHERE ");
                has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_condition(&mut query_builder);
                query_builder.push("bs.company_code IN (");
                let mut separated = query_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(date_from) = filter.posting_date_from {
            push_condition(&mut query_builder);
            query_builder.push("st.value_date >= ");
            query_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.posting_date_to {
            push_condition(&mut query_builder);
            query_builder.push("st.value_date <= ");
            query_builder.push_bind(date_to);
        }

        let rows = query_builder.build().fetch_all(&self.pool).await?;
        let mut entries = Vec::new();
        for row in rows {
            let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let value_date: chrono::NaiveDate = row.try_get("value_date")?;
            let amount: Decimal = row.try_get("amount")?;
            let currency: String = row.try_get("currency")?;

            entries.push(UniversalJournalEntry {
                ledger: "0L".to_string(),
                company_code: row.try_get("company_code")?,
                fiscal_year: value_date.year(),
                document_number: row.try_get::<uuid::Uuid, _>("transaction_id")?.to_string(),
                document_line: 1,
                document_type: "TR".to_string(),
                document_date: value_date,
                posting_date: value_date,
                fiscal_period: value_date.month() as i32,
                reference_document: None,
                header_text: None,
                document_currency: currency.clone(),
                exchange_rate: None,
                logical_system: None,
                transaction_code: None,
                posting_key: "40".to_string(),
                debit_credit_indicator: "S".to_string(),
                account_type: AccountType::GL,
                gl_account: "TRBANK".to_string(),
                business_partner: row.try_get("partner_name")?,
                material: None,
                plant: None,
                item_text: row.try_get("memo")?,
                assignment_number: None,
                amount_in_document_currency: amount,
                amount_in_local_currency: amount,
                amount_in_group_currency: None,
                amount_in_global_currency: None,
                amount_in_ledger_currency: None,
                quantity: None,
                quantity_unit: None,
                cost_center: None,
                profit_center: None,
                segment: None,
                functional_area: None,
                business_area: None,
                controlling_area: None,
                internal_order: None,
                wbs_element: None,
                sales_order: None,
                sales_order_item: None,
                tax_code: None,
                tax_jurisdiction: None,
                tax_amount: None,
                clearing_document: None,
                clearing_date: None,
                baseline_date: None,
                due_date: None,
                payment_terms: None,
                payment_method: None,
                payment_block: None,
                house_bank: row.try_get("house_bank")?,
                special_gl_indicator: None,
                reference_document_number: None,
                reference_fiscal_year: None,
                reference_line_item: None,
                reference_document_type: None,
                transaction_type: row.try_get("transaction_type")?,
                reference_transaction_type: None,
                reference_key_1: None,
                reference_key_2: None,
                reference_key_3: None,
                financial_area: None,
                consolidation_unit: None,
                partner_company: None,
                trading_partner: None,
                local_currency: currency,
                group_currency: None,
                global_currency: None,
                amount_in_object_currency: None,
                amount_in_profit_center_currency: None,
                dunning_key: None,
                dunning_block: None,
                last_dunning_date: None,
                dunning_level: None,
                discount_days_1: None,
                discount_days_2: None,
                net_payment_days: None,
                discount_percent_1: None,
                discount_percent_2: None,
                discount_amount: None,
                sending_cost_center: None,
                partner_profit_center: None,
                sending_financial_area: None,
                account_assignment: None,
                local_account: None,
                data_source: None,
                split_method: None,
                manual_split: false,
                created_by: "TR".to_string(),
                created_at: created_at.naive_utc(),
                changed_by: None,
                changed_at: None,
                source_module: SourceModule::TR,
                extension_fields: HashMap::new(),
            });
        }

        let mut payment_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                pd.doc_id,
                pd.document_number,
                pd.fiscal_year,
                pd.amount,
                pd.currency,
                pd.payee_name,
                pd.payment_method,
                pd.house_bank,
                pd.bank_account,
                pd.transaction_type,
                pr.company_codes,
                pr.posting_date,
                pr.created_at
            FROM payment_documents pd
            JOIN payment_runs pr ON pr.run_id = pd.run_id
            "#,
        );

        let mut payment_has_conditions = false;
        let mut push_payment_condition = |builder: &mut QueryBuilder<Postgres>| {
            if !payment_has_conditions {
                builder.push(" WHERE ");
                payment_has_conditions = true;
            } else {
                builder.push(" AND ");
            }
        };

        if let Some(company_codes) = &filter.company_codes {
            if !company_codes.is_empty() {
                push_payment_condition(&mut payment_builder);
                payment_builder.push("split_part(pr.company_codes, ',', 1) IN (");
                let mut separated = payment_builder.separated(", ");
                for code in company_codes {
                    separated.push_bind(code);
                }
                separated.push_unseparated(")");
            }
        }

        if let Some(date_from) = filter.posting_date_from {
            push_payment_condition(&mut payment_builder);
            payment_builder.push("COALESCE(pr.posting_date, pr.created_at::date) >= ");
            payment_builder.push_bind(date_from);
        }

        if let Some(date_to) = filter.posting_date_to {
            push_payment_condition(&mut payment_builder);
            payment_builder.push("COALESCE(pr.posting_date, pr.created_at::date) <= ");
            payment_builder.push_bind(date_to);
        }

        let payment_rows = payment_builder.build().fetch_all(&self.pool).await?;
        for row in payment_rows {
            let created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let posting_date: Option<chrono::NaiveDate> = row.try_get("posting_date")?;
            let posting_date = posting_date.unwrap_or_else(|| created_at.date_naive());
            let amount: Decimal = row.try_get("amount")?;
            let currency: String = row.try_get("currency")?;
            let company_code = row
                .try_get::<Option<String>, _>("company_codes")?
                .and_then(|codes| codes.split(',').next().map(|s| s.trim().to_string()))
                .unwrap_or_default();

            entries.push(UniversalJournalEntry {
                ledger: "0L".to_string(),
                company_code,
                fiscal_year: row
                    .try_get::<Option<i32>, _>("fiscal_year")?
                    .unwrap_or(posting_date.year()),
                document_number: row.try_get("document_number")?,
                document_line: 1,
                document_type: "TR".to_string(),
                document_date: posting_date,
                posting_date,
                fiscal_period: posting_date.month() as i32,
                reference_document: None,
                header_text: None,
                document_currency: currency.clone(),
                exchange_rate: None,
                logical_system: None,
                transaction_code: None,
                posting_key: "50".to_string(),
                debit_credit_indicator: "H".to_string(),
                account_type: AccountType::GL,
                gl_account: "TRPAY".to_string(),
                business_partner: row.try_get("payee_name")?,
                material: None,
                plant: None,
                item_text: None,
                assignment_number: None,
                amount_in_document_currency: amount,
                amount_in_local_currency: amount,
                amount_in_group_currency: None,
                amount_in_global_currency: None,
                amount_in_ledger_currency: None,
                quantity: None,
                quantity_unit: None,
                cost_center: None,
                profit_center: None,
                segment: None,
                functional_area: None,
                business_area: None,
                controlling_area: None,
                internal_order: None,
                wbs_element: None,
                sales_order: None,
                sales_order_item: None,
                tax_code: None,
                tax_jurisdiction: None,
                tax_amount: None,
                clearing_document: None,
                clearing_date: None,
                baseline_date: None,
                due_date: None,
                payment_terms: None,
                payment_method: row.try_get("payment_method")?,
                payment_block: None,
                house_bank: row.try_get("house_bank")?,
                special_gl_indicator: None,
                reference_document_number: None,
                reference_fiscal_year: None,
                reference_line_item: None,
                reference_document_type: None,
                transaction_type: row.try_get("transaction_type")?,
                reference_transaction_type: None,
                reference_key_1: None,
                reference_key_2: None,
                reference_key_3: None,
                financial_area: None,
                consolidation_unit: None,
                partner_company: None,
                trading_partner: None,
                local_currency: currency,
                group_currency: None,
                global_currency: None,
                amount_in_object_currency: None,
                amount_in_profit_center_currency: None,
                dunning_key: None,
                dunning_block: None,
                last_dunning_date: None,
                dunning_level: None,
                discount_days_1: None,
                discount_days_2: None,
                net_payment_days: None,
                discount_percent_1: None,
                discount_percent_2: None,
                discount_amount: None,
                sending_cost_center: None,
                partner_profit_center: None,
                sending_financial_area: None,
                account_assignment: None,
                local_account: None,
                data_source: None,
                split_method: None,
                manual_split: false,
                created_by: "TR".to_string(),
                created_at: created_at.naive_utc(),
                changed_by: None,
                changed_at: None,
                source_module: SourceModule::TR,
                extension_fields: HashMap::new(),
            });
        }

        Ok(entries)
    }

    async fn fetch_module_entries(
        &self,
        filter: &UniversalJournalFilter,
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        let mut entries = Vec::new();

        entries.extend(self.fetch_ap_open_items(filter).await?);
        entries.extend(self.fetch_ar_open_items(filter).await?);
        entries.extend(self.fetch_co_allocations(filter).await?);
        entries.extend(self.fetch_tr_entries(filter).await?);

        Ok(entries)
    }
}

#[async_trait]
impl UniversalJournalRepository for PostgresUniversalJournalRepository {
    async fn query(
        &self,
        filter: &UniversalJournalFilter,
        pagination: &PaginationParams,
        order_by: &[String],
    ) -> Result<(Vec<UniversalJournalEntry>, PaginationResponse), RepositoryError> {
        let mut entries = self.stream(filter, order_by).await?;
        let total_count = entries.len() as i64;

        let start = pagination.offset() as usize;
        let end = (start + pagination.limit() as usize).min(entries.len());
        let paged_entries = if start < entries.len() {
            entries[start..end].to_vec()
        } else {
            Vec::new()
        };

        let pagination_response =
            PaginationResponse::new(total_count, pagination.page, pagination.page_size);

        Ok((paged_entries, pagination_response))
    }

    async fn stream(
        &self,
        filter: &UniversalJournalFilter,
        order_by: &[String],
    ) -> Result<Vec<UniversalJournalEntry>, RepositoryError> {
        let (where_clause, _params) = self.build_where_clause(filter);

        let order_clause = if order_by.is_empty() {
            "ORDER BY posting_date DESC, document_number, document_line".to_string()
        } else {
            format!("ORDER BY {}", order_by.join(", "))
        };

        let query = format!(
            "SELECT * FROM universal_journal_entries {} {}",
            where_clause, order_clause
        );

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;
        let mut entries: Vec<UniversalJournalEntry> = rows
            .iter()
            .map(|row| self.map_row_to_entry(row))
            .collect::<Result<Vec<_>, _>>()?;

        let module_entries = self.fetch_module_entries(filter).await?;
        entries.extend(
            module_entries
                .into_iter()
                .filter(|entry| Self::entry_matches_filter(entry, filter)),
        );

        entries.sort_by(|a, b| {
            b.posting_date
                .cmp(&a.posting_date)
                .then_with(|| a.document_number.cmp(&b.document_number))
                .then_with(|| a.document_line.cmp(&b.document_line))
        });

        Ok(entries)
    }

    async fn stream_batched(
        &self,
        filter: &UniversalJournalFilter,
        order_by: &[String],
        params: &crate::domain::streaming::StreamingParams,
    ) -> Result<std::pin::Pin<Box<dyn futures::Stream<Item = Result<Vec<UniversalJournalEntry>, RepositoryError>> + Send>>, RepositoryError> {
        let entries = self.stream(filter, order_by).await?;
        let batch_size = params.batch_size.max(1);
        
        let chunks: Vec<Vec<UniversalJournalEntry>> = entries
            .chunks(batch_size)
            .map(|chunk| chunk.to_vec())
            .collect();
            
        Ok(Box::pin(futures::stream::iter(chunks.into_iter().map(Ok))))
    }

    async fn get_by_key(
        &self,
        ledger: &str,
        company_code: &str,
        fiscal_year: i32,
        document_number: &str,
        document_line: i32,
    ) -> Result<Option<UniversalJournalEntry>, RepositoryError> {
        let query = r#"
            SELECT * FROM universal_journal_entries
            WHERE ledger = $1
              AND company_code = $2
              AND fiscal_year = $3
              AND document_number = $4
              AND document_line = $5
        "#;

        let row = sqlx::query(query)
            .bind(ledger)
            .bind(company_code)
            .bind(fiscal_year)
            .bind(document_number)
            .bind(document_line)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(self.map_row_to_entry(&r)?)),
            None => {
                let filter = UniversalJournalFilter {
                    ledgers: Some(vec![ledger.to_string()]),
                    company_codes: Some(vec![company_code.to_string()]),
                    fiscal_year_from: Some(fiscal_year),
                    fiscal_year_to: Some(fiscal_year),
                    ..Default::default()
                };

                let entries = self.fetch_module_entries(&filter).await?;
                let entry = entries.into_iter().find(|entry| {
                    entry.ledger == ledger
                        && entry.company_code == company_code
                        && entry.fiscal_year == fiscal_year
                        && entry.document_number == document_number
                        && entry.document_line == document_line
                });

                Ok(entry)
            },
        }
    }

    async fn aggregate(
        &self,
        filter: &UniversalJournalFilter,
        dimensions: &[String],
        measure: &str,
        measure_field: &str,
    ) -> Result<Vec<AggregationResult>, RepositoryError> {
        let entries = self.stream(filter, &[]).await?;
        let mut aggregation_map: HashMap<String, (Decimal, i64)> = HashMap::new();

        for entry in entries {
            let dimension_key = if dimensions.iter().any(|d| d == "gl_account") {
                entry.gl_account.clone()
            } else if dimensions.iter().any(|d| d == "cost_center") {
                entry.cost_center.clone().unwrap_or_default()
            } else if dimensions.iter().any(|d| d == "profit_center") {
                entry.profit_center.clone().unwrap_or_default()
            } else if dimensions.iter().any(|d| d == "company_code") {
                entry.company_code.clone()
            } else if dimensions.iter().any(|d| d == "document_type") {
                entry.document_type.clone()
            } else if dimensions.iter().any(|d| d == "source_module") {
                Self::source_module_code(entry.source_module).to_string()
            } else {
                "ALL".to_string()
            };

            let amount = match measure_field {
                "amount_in_local_currency" => entry.amount_in_local_currency,
                _ => entry.amount_in_document_currency,
            };

            let entry_agg = aggregation_map
                .entry(dimension_key)
                .or_insert((Decimal::ZERO, 0));
            entry_agg.0 += amount;
            entry_agg.1 += 1;
        }

        let results = aggregation_map
            .into_iter()
            .map(|(key, (sum, count))| AggregationResult {
                dimension_values: {
                    let mut map = HashMap::new();
                    map.insert("dimension".to_string(), key);
                    map
                },
                measure_value: sum.to_string(),
                record_count: count,
            })
            .collect();

        Ok(results)
    }

    async fn save(&self, entry: &UniversalJournalEntry) -> Result<(), RepositoryError> {
        let query = r#"
            INSERT INTO universal_journal_entries (
                ledger, company_code, fiscal_year, document_number, document_line,
                document_type, document_date, posting_date, fiscal_period,
                document_currency, posting_key, debit_credit_indicator,
                account_type, gl_account, amount_in_document_currency,
                amount_in_local_currency, local_currency, source_module,
                created_by, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            ON CONFLICT (ledger, company_code, fiscal_year, document_number, document_line)
            DO UPDATE SET
                document_type = EXCLUDED.document_type,
                document_date = EXCLUDED.document_date,
                posting_date = EXCLUDED.posting_date,
                amount_in_document_currency = EXCLUDED.amount_in_document_currency,
                amount_in_local_currency = EXCLUDED.amount_in_local_currency,
                changed_by = EXCLUDED.created_by,
                changed_at = CURRENT_TIMESTAMP
        "#;

        let source_module_str = match entry.source_module {
            SourceModule::GL => "GL",
            SourceModule::AP => "AP",
            SourceModule::AR => "AR",
            SourceModule::AA => "AA",
            SourceModule::MM => "MM",
            SourceModule::SD => "SD",
            SourceModule::CO => "CO",
            SourceModule::TR => "TR",
            _ => "UNSPECIFIED",
        };

        let account_type_str = match entry.account_type {
            AccountType::GL => "GL",
            AccountType::Vendor => "Vendor",
            AccountType::Customer => "Customer",
            AccountType::Asset => "Asset",
            AccountType::Material => "Material",
            _ => "UNSPECIFIED",
        };

        sqlx::query(query)
            .bind(&entry.ledger)
            .bind(&entry.company_code)
            .bind(entry.fiscal_year)
            .bind(&entry.document_number)
            .bind(entry.document_line)
            .bind(&entry.document_type)
            .bind(entry.document_date)
            .bind(entry.posting_date)
            .bind(entry.fiscal_period)
            .bind(&entry.document_currency)
            .bind(&entry.posting_key)
            .bind(&entry.debit_credit_indicator)
            .bind(account_type_str)
            .bind(&entry.gl_account)
            .bind(entry.amount_in_document_currency)
            .bind(entry.amount_in_local_currency)
            .bind(&entry.local_currency)
            .bind(source_module_str)
            .bind(&entry.created_by)
            .bind(entry.created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn batch_save(&self, entries: &[UniversalJournalEntry]) -> Result<(), RepositoryError> {
        let tx = self.pool.begin().await?;

        for entry in entries {
            self.save(entry).await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use sqlx::PgPool;

    #[test]
    fn build_where_clause_includes_source_module_filter() {
        let pool = PgPool::connect_lazy("postgres://postgres:postgres@localhost:5432/uj_test")
            .expect("connect_lazy should not fail");
        let repo = PostgresUniversalJournalRepository::new(pool);
        let filter = UniversalJournalFilter {
            source_modules: Some(vec!["AP".to_string(), "AR".to_string()]),
            ..Default::default()
        };

        let (clause, params) = repo.build_where_clause(&filter);
        assert!(clause.contains("source_module IN"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn entry_filter_honors_source_module_and_company_code() {
        let posting_date = NaiveDate::from_ymd_opt(2026, 1, 19).unwrap();
        let entry = UniversalJournalEntry::new(
            "0L".to_string(),
            "1000".to_string(),
            2026,
            "AP-TEST".to_string(),
            1,
            "KR".to_string(),
            posting_date,
            posting_date,
            1,
            "CNY".to_string(),
            "CNY".to_string(),
            "31".to_string(),
            "H".to_string(),
            AccountType::Vendor,
            "160000".to_string(),
            Decimal::new(10000, 2),
            Decimal::new(10000, 2),
            SourceModule::AP,
            "tester".to_string(),
        );

        let filter = UniversalJournalFilter {
            company_codes: Some(vec!["1000".to_string()]),
            source_modules: Some(vec!["AP".to_string()]),
            ..Default::default()
        };
        assert!(PostgresUniversalJournalRepository::entry_matches_filter(
            &entry, &filter
        ));

        let mismatch = UniversalJournalFilter {
            company_codes: Some(vec!["2000".to_string()]),
            ..Default::default()
        };
        assert!(!PostgresUniversalJournalRepository::entry_matches_filter(
            &entry, &mismatch
        ));
    }
}
