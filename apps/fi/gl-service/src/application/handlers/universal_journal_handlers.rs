// 统一日记账查询处理器
use crate::domain::repositories::JournalRepository;
use crate::infrastructure::grpc::common::v1 as common_v1;
use crate::infrastructure::grpc::fi::uj::v1::*;
use anyhow::Result;
use chrono::Datelike;
use std::sync::Arc;

// 查询统一日记账
pub struct QueryUniversalJournalHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> QueryUniversalJournalHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        request: QueryUniversalJournalRequest,
    ) -> Result<QueryUniversalJournalResponse> {
        let filter = request.filter.unwrap_or_default();

        // 构建查询条件
        let company_code = filter
            .company_codes
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");

        let page = request
            .pagination
            .as_ref()
            .map(|p| p.page as u64)
            .unwrap_or(1);
        let page_size = request
            .pagination
            .as_ref()
            .map(|p| p.page_size as u64)
            .unwrap_or(100);

        // 从 GL 模块查询数据
        let entries = self
            .repository
            .search(
                company_code,
                None, // status
                page,
                page_size,
            )
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let total_count = self
            .repository
            .count(company_code, None)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // 转换为 ACDOCA 格式
        let universal_entries: Vec<UniversalJournalEntry> = entries
            .into_iter()
            .flat_map(|entry| {
                // 每个凭证的每行转换为一条 ACDOCA 记录
                entry.lines.into_iter().enumerate().map(move |(idx, line)| {
                    UniversalJournalEntry {
                        // 主键字段
                        ledger: line.ledger.clone(),
                        company_code: entry.company_code.clone(),
                        fiscal_year: entry.fiscal_year,
                        document_number: entry.document_number.clone().unwrap_or_default(),
                        document_line: (idx + 1) as i32,

                        // 凭证抬头字段
                        document_type: "SA".to_string(),
                        document_date: Some(prost_types::Timestamp {
                            seconds: entry
                                .document_date
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_utc()
                                .timestamp(),
                            nanos: 0,
                        }),
                        posting_date: Some(prost_types::Timestamp {
                            seconds: entry
                                .posting_date
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_utc()
                                .timestamp(),
                            nanos: 0,
                        }),
                        fiscal_period: entry.posting_date.month() as i32,
                        reference_document: entry.reference.clone().unwrap_or_default(),
                        header_text: "".to_string(),
                        document_currency: entry.currency.clone(),
                        exchange_rate: "1.0".to_string(),
                        logical_system: "".to_string(),
                        transaction_code: "".to_string(),

                        // 行项目字段
                        posting_key: "".to_string(),
                        debit_credit_indicator: line.debit_credit.as_char().to_string(),
                        account_type: AccountType::Gl as i32,
                        gl_account: line.account_id.clone(),
                        business_partner: "".to_string(),
                        material: "".to_string(),
                        plant: "".to_string(),
                        item_text: line.text.clone().unwrap_or_default(),
                        assignment_number: "".to_string(),

                        // 金额字段
                        amount_in_document_currency: Some(common_v1::MonetaryValue {
                            value: line.amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        amount_in_local_currency: Some(common_v1::MonetaryValue {
                            value: line.local_amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        amount_in_group_currency: line.amount_in_group_currency.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line.group_currency.clone().unwrap_or_default(),
                            }
                        }),
                        amount_in_global_currency: None,
                        amount_in_ledger_currency: line.ledger_amount.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: entry.currency.clone(),
                            }
                        }),

                        // 数量字段
                        quantity: None,

                        // 成本对象字段
                        cost_center: line.cost_center.clone().unwrap_or_default(),
                        profit_center: line.profit_center.clone().unwrap_or_default(),
                        segment: "".to_string(),
                        functional_area: "".to_string(),
                        business_area: line.business_area.clone().unwrap_or_default(),
                        controlling_area: line.controlling_area.clone().unwrap_or_default(),
                        internal_order: "".to_string(),
                        wbs_element: "".to_string(),
                        sales_order: "".to_string(),
                        sales_order_item: 0,

                        // 税务字段
                        tax_code: "".to_string(),
                        tax_jurisdiction: "".to_string(),
                        tax_amount: None,

                        // 清账字段
                        clearing_document: "".to_string(),
                        clearing_date: None,

                        // 付款字段
                        baseline_date: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.baseline_date)
                            .or_else(|| {
                                line.payment_execution
                                    .as_ref()
                                    .and_then(|pe| pe.payment_baseline_date)
                            })
                            .map(|date| prost_types::Timestamp {
                                seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                nanos: 0,
                            }),
                        due_date: line.maturity_date.map(|date| prost_types::Timestamp {
                            seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                            nanos: 0,
                        }),
                        payment_terms: "".to_string(),
                        payment_method: line
                            .payment_execution
                            .as_ref()
                            .map(|pe| pe.payment_method.clone())
                            .unwrap_or_default(),
                        payment_block: line
                            .payment_execution
                            .as_ref()
                            .and_then(|pe| pe.payment_block.clone())
                            .unwrap_or_default(),
                        house_bank: line
                            .payment_execution
                            .as_ref()
                            .and_then(|pe| pe.house_bank.clone())
                            .unwrap_or_default(),

                        // 特殊总账字段
                        special_gl_indicator: line.special_gl_indicator.to_sap_code().to_string(),

                        // 发票参考字段
                        reference_document_number: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_document_number.clone())
                            .unwrap_or_default(),
                        reference_fiscal_year: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_fiscal_year)
                            .unwrap_or(0),
                        reference_line_item: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_line_item)
                            .unwrap_or(0),
                        reference_document_type: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_document_type.clone())
                            .unwrap_or_default(),

                        // 业务交易类型字段
                        transaction_type: line.transaction_type.clone().unwrap_or_default(),
                        reference_transaction_type: line
                            .reference_transaction_type
                            .clone()
                            .unwrap_or_default(),
                        reference_key_1: "".to_string(),
                        reference_key_2: "".to_string(),
                        reference_key_3: "".to_string(),

                        // 组织维度字段
                        financial_area: line.financial_area.clone().unwrap_or_default(),
                        consolidation_unit: "".to_string(),
                        partner_company: line.trading_partner_company.clone().unwrap_or_default(),
                        trading_partner: line.trading_partner_company.clone().unwrap_or_default(),

                        // 多币种字段
                        local_currency: entry.currency.clone(),
                        group_currency: line.group_currency.clone().unwrap_or_default(),
                        global_currency: "".to_string(),
                        amount_in_object_currency: line.amount_in_object_currency.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line.object_currency.clone().unwrap_or_default(),
                            }
                        }),
                        amount_in_profit_center_currency: line
                            .amount_in_profit_center_currency
                            .as_ref()
                            .map(|amt| common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line
                                    .profit_center_currency
                                    .clone()
                                    .unwrap_or_default(),
                            }),

                        // 催款字段
                        dunning_key: line
                            .dunning_detail
                            .as_ref()
                            .and_then(|dd| dd.dunning_key.clone())
                            .unwrap_or_default(),
                        dunning_block: line
                            .dunning_detail
                            .as_ref()
                            .and_then(|dd| dd.dunning_block.clone())
                            .unwrap_or_default(),
                        last_dunning_date: line.dunning_detail.as_ref().and_then(|dd| {
                            dd.last_dunning_date.map(|date| prost_types::Timestamp {
                                seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                nanos: 0,
                            })
                        }),
                        dunning_level: line
                            .dunning_detail
                            .as_ref()
                            .map(|dd| dd.dunning_level)
                            .unwrap_or(0),

                        // 付款条件详细字段
                        discount_days_1: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.discount_days_1)
                            .unwrap_or(0),
                        discount_days_2: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.discount_days_2)
                            .unwrap_or(0),
                        net_payment_days: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.net_payment_days)
                            .unwrap_or(0),
                        discount_percent_1: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_percent_1)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "0".to_string()),
                        discount_percent_2: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_percent_2)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "0".to_string()),
                        discount_amount: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_amount)
                            .map(|amt| common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: entry.currency.clone(),
                            }),

                        // 内部交易字段
                        sending_cost_center: "".to_string(),
                        partner_profit_center: "".to_string(),
                        sending_financial_area: "".to_string(),

                        // 科目分配字段
                        account_assignment: line.account_assignment.clone().unwrap_or_default(),

                        // 本地 GAAP 字段
                        local_account: "".to_string(),
                        data_source: "".to_string(),

                        // 字段拆分字段
                        split_method: "".to_string(),
                        manual_split: false,

                        // 审计字段
                        created_by: "".to_string(),
                        created_at: None,
                        created_time: None,
                        changed_by: "".to_string(),
                        changed_at: None,

                        // 来源模块标识
                        source_module: SourceModule::Gl as i32,

                        // 扩展字段
                        extension_fields: std::collections::HashMap::new(),
                    }
                })
            })
            .collect();

        Ok(QueryUniversalJournalResponse {
            entries: universal_entries,
            pagination: Some(common_v1::PaginationResponse {
                total_items: total_count,
                total_pages: ((total_count as u64 + page_size - 1) / page_size) as i32,
                current_page: page as i32,
                page_size: page_size as i32,
            }),
        })
    }
}

// 流式查询统一日记账
pub struct StreamUniversalJournalHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository + 'static> StreamUniversalJournalHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// 流式查询 - 分批获取数据并转换为流
    /// 每批 100 条，避免内存溢出
    pub fn stream(
        &self,
        request: QueryUniversalJournalRequest,
    ) -> impl tokio_stream::Stream<Item = Result<UniversalJournalEntry, tonic::Status>> + Send + 'static
    {
        let repository = Arc::clone(&self.repository);

        async_stream::try_stream! {
            let filter = request.filter.unwrap_or_default();
            let company_code = filter
                .company_codes
                .first()
                .cloned()
                .unwrap_or_default();

            const BATCH_SIZE: u64 = 100;
            let mut page: u64 = 1;

            loop {
                // 分批查询
                let entries = repository
                    .search(&company_code, None, page, BATCH_SIZE)
                    .await
                    .map_err(|e| tonic::Status::internal(e.to_string()))?;

                if entries.is_empty() {
                    break;
                }

                // 转换为 ACDOCA 格式并逐条发送
                for entry in entries {
                    for (idx, line) in entry.lines.into_iter().enumerate() {
                        yield UniversalJournalEntry {
                            // 主键字段
                            ledger: line.ledger.clone(),
                            company_code: entry.company_code.clone(),
                            fiscal_year: entry.fiscal_year,
                            document_number: entry.document_number.clone().unwrap_or_default(),
                            document_line: (idx + 1) as i32,

                            // 凭证抬头字段
                            document_type: "SA".to_string(),
                            document_date: Some(prost_types::Timestamp {
                                seconds: entry
                                    .document_date
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_utc()
                                    .timestamp(),
                                nanos: 0,
                            }),
                            posting_date: Some(prost_types::Timestamp {
                                seconds: entry
                                    .posting_date
                                    .and_hms_opt(0, 0, 0)
                                    .unwrap()
                                    .and_utc()
                                    .timestamp(),
                                nanos: 0,
                            }),
                            fiscal_period: entry.posting_date.month() as i32,
                            reference_document: entry.reference.clone().unwrap_or_default(),
                            header_text: "".to_string(),
                            document_currency: entry.currency.clone(),
                            exchange_rate: "1.0".to_string(),
                            logical_system: "".to_string(),
                            transaction_code: "".to_string(),

                            // 行项目字段
                            posting_key: "".to_string(),
                            debit_credit_indicator: line.debit_credit.as_char().to_string(),
                            account_type: AccountType::Gl as i32,
                            gl_account: line.account_id.clone(),
                            business_partner: line.business_partner.clone().unwrap_or_default(),
                            material: "".to_string(),
                            plant: "".to_string(),
                            item_text: line.text.clone().unwrap_or_default(),
                            assignment_number: "".to_string(),

                            // 金额字段
                            amount_in_document_currency: Some(common_v1::MonetaryValue {
                                value: line.amount.to_string(),
                                currency_code: entry.currency.clone(),
                            }),
                            amount_in_local_currency: Some(common_v1::MonetaryValue {
                                value: line.local_amount.to_string(),
                                currency_code: entry.currency.clone(),
                            }),
                            amount_in_group_currency: line.amount_in_group_currency.as_ref().map(|amt| {
                                common_v1::MonetaryValue {
                                    value: amt.to_string(),
                                    currency_code: line.group_currency.clone().unwrap_or_default(),
                                }
                            }),
                            amount_in_global_currency: None,
                            amount_in_ledger_currency: line.ledger_amount.as_ref().map(|amt| {
                                common_v1::MonetaryValue {
                                    value: amt.to_string(),
                                    currency_code: entry.currency.clone(),
                                }
                            }),

                            // 数量字段
                            quantity: None,

                            // 成本对象字段
                            cost_center: line.cost_center.clone().unwrap_or_default(),
                            profit_center: line.profit_center.clone().unwrap_or_default(),
                            segment: "".to_string(),
                            functional_area: "".to_string(),
                            business_area: line.business_area.clone().unwrap_or_default(),
                            controlling_area: line.controlling_area.clone().unwrap_or_default(),
                            internal_order: "".to_string(),
                            wbs_element: "".to_string(),
                            sales_order: "".to_string(),
                            sales_order_item: 0,

                            // 税务字段
                            tax_code: "".to_string(),
                            tax_jurisdiction: "".to_string(),
                            tax_amount: None,

                            // 清账字段
                            clearing_document: "".to_string(),
                            clearing_date: None,

                            // 付款字段 - 从 payment_execution/payment_terms_detail 获取
                            baseline_date: line
                                .payment_terms_detail
                                .as_ref()
                                .and_then(|ptd| ptd.baseline_date)
                                .or_else(|| {
                                    line.payment_execution
                                        .as_ref()
                                        .and_then(|pe| pe.payment_baseline_date)
                                })
                                .map(|date| prost_types::Timestamp {
                                    seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                    nanos: 0,
                                }),
                            due_date: line.maturity_date.map(|date| prost_types::Timestamp {
                                seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                nanos: 0,
                            }),
                            payment_terms: "".to_string(),
                            payment_method: line.payment_execution.as_ref()
                                .map(|pe| pe.payment_method.clone())
                                .unwrap_or_default(),
                            payment_block: line.payment_execution.as_ref()
                                .and_then(|pe| pe.payment_block.clone())
                                .unwrap_or_default(),
                            house_bank: line.payment_execution.as_ref()
                                .and_then(|pe| pe.house_bank.clone())
                                .unwrap_or_default(),

                            // 特殊总账字段
                            special_gl_indicator: line.special_gl_indicator.to_sap_code().to_string(),

                            // 发票参考字段
                            reference_document_number: line
                                .invoice_reference
                                .as_ref()
                                .and_then(|ir| ir.reference_document_number.clone())
                                .unwrap_or_default(),
                            reference_fiscal_year: line
                                .invoice_reference
                                .as_ref()
                                .and_then(|ir| ir.reference_fiscal_year)
                                .unwrap_or(0),
                            reference_line_item: line
                                .invoice_reference
                                .as_ref()
                                .and_then(|ir| ir.reference_line_item)
                                .unwrap_or(0),
                            reference_document_type: line
                                .invoice_reference
                                .as_ref()
                                .and_then(|ir| ir.reference_document_type.clone())
                                .unwrap_or_default(),

                            // 业务交易类型字段
                            transaction_type: line.transaction_type.clone().unwrap_or_default(),
                            reference_transaction_type: line.reference_transaction_type.clone().unwrap_or_default(),
                            reference_key_1: "".to_string(),
                            reference_key_2: "".to_string(),
                            reference_key_3: "".to_string(),

                            // 组织维度字段
                            financial_area: line.financial_area.clone().unwrap_or_default(),
                            consolidation_unit: "".to_string(),
                            partner_company: "".to_string(),
                            trading_partner: line.trading_partner_company.clone().unwrap_or_default(),

                            // 多币种字段
                            local_currency: entry.currency.clone(),
                            group_currency: line.group_currency.clone().unwrap_or_default(),
                            global_currency: "".to_string(),
                            amount_in_object_currency: line.amount_in_object_currency.as_ref().map(|amt| {
                                common_v1::MonetaryValue {
                                    value: amt.to_string(),
                                    currency_code: line.object_currency.clone().unwrap_or_default(),
                                }
                            }),
                            amount_in_profit_center_currency: line.amount_in_profit_center_currency.as_ref().map(|amt| {
                                common_v1::MonetaryValue {
                                    value: amt.to_string(),
                                    currency_code: line.profit_center_currency.clone().unwrap_or_default(),
                                }
                            }),

                            // 催款字段
                            dunning_key: line
                                .dunning_detail
                                .as_ref()
                                .and_then(|dd| dd.dunning_key.clone())
                                .unwrap_or_default(),
                            dunning_block: line
                                .dunning_detail
                                .as_ref()
                                .and_then(|dd| dd.dunning_block.clone())
                                .unwrap_or_default(),
                            last_dunning_date: line.dunning_detail.as_ref().and_then(|dd| {
                                dd.last_dunning_date.map(|date| prost_types::Timestamp {
                                    seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                    nanos: 0,
                                })
                            }),
                            dunning_level: line
                                .dunning_detail
                                .as_ref()
                                .map(|dd| dd.dunning_level)
                                .unwrap_or(0),

                            // 付款条件详细字段 - 从 payment_terms_detail 获取
                            discount_days_1: line.payment_terms_detail.as_ref()
                                .map(|ptd| ptd.discount_days_1)
                                .unwrap_or(0),
                            discount_days_2: line.payment_terms_detail.as_ref()
                                .map(|ptd| ptd.discount_days_2)
                                .unwrap_or(0),
                            net_payment_days: line.payment_terms_detail.as_ref()
                                .map(|ptd| ptd.net_payment_days)
                                .unwrap_or(0),
                            discount_percent_1: line.payment_terms_detail.as_ref()
                                .and_then(|ptd| ptd.discount_percent_1)
                                .map(|d| d.to_string())
                                .unwrap_or_else(|| "0".to_string()),
                            discount_percent_2: line.payment_terms_detail.as_ref()
                                .and_then(|ptd| ptd.discount_percent_2)
                                .map(|d| d.to_string())
                                .unwrap_or_else(|| "0".to_string()),
                            discount_amount: line.payment_terms_detail.as_ref()
                                .and_then(|ptd| ptd.discount_amount)
                                .map(|amt| common_v1::MonetaryValue {
                                    value: amt.to_string(),
                                    currency_code: entry.currency.clone(),
                                }),

                            // 内部交易字段
                            sending_cost_center: "".to_string(),
                            partner_profit_center: "".to_string(),
                            sending_financial_area: "".to_string(),

                            // 科目分配字段
                            account_assignment: line.account_assignment.clone().unwrap_or_default(),

                            // 本地 GAAP 字段
                            local_account: "".to_string(),
                            data_source: "".to_string(),

                            // 字段拆分字段
                            split_method: "".to_string(),
                            manual_split: false,

                            // 审计字段
                            created_by: "".to_string(),
                            created_at: Some(prost_types::Timestamp {
                                seconds: entry.created_at.timestamp(),
                                nanos: 0,
                            }),
                            created_time: None,
                            changed_by: "".to_string(),
                            changed_at: None,

                            // 来源模块标识
                            source_module: SourceModule::Gl as i32,

                            // 扩展字段
                            extension_fields: std::collections::HashMap::new(),
                        };
                    }
                }

                page += 1;
            }
        }
    }
}

// 获取单条统一日记账记录
pub struct GetUniversalJournalEntryHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> GetUniversalJournalEntryHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        request: GetUniversalJournalEntryRequest,
    ) -> Result<Option<UniversalJournalEntry>> {
        // 根据凭证号查询
        let entries = self
            .repository
            .search(&request.company_code, None, 1, 1000)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // 查找匹配的凭证和行
        for entry in entries {
            if entry.fiscal_year == request.fiscal_year
                && entry.document_number.as_ref() == Some(&request.document_number)
            {
                // 找到对应行
                if let Some(line) = entry.lines.get((request.document_line - 1) as usize) {
                    return Ok(Some(UniversalJournalEntry {
                        ledger: request.ledger.clone(),
                        company_code: request.company_code.clone(),
                        fiscal_year: request.fiscal_year,
                        document_number: request.document_number.clone(),
                        document_line: request.document_line,

                        document_type: "SA".to_string(),
                        document_date: Some(prost_types::Timestamp {
                            seconds: entry
                                .document_date
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_utc()
                                .timestamp(),
                            nanos: 0,
                        }),
                        posting_date: Some(prost_types::Timestamp {
                            seconds: entry
                                .posting_date
                                .and_hms_opt(0, 0, 0)
                                .unwrap()
                                .and_utc()
                                .timestamp(),
                            nanos: 0,
                        }),
                        fiscal_period: entry.posting_date.month() as i32,
                        reference_document: entry.reference.clone().unwrap_or_default(),
                        header_text: "".to_string(),
                        document_currency: entry.currency.clone(),
                        exchange_rate: "1.0".to_string(),
                        logical_system: "".to_string(),
                        transaction_code: "".to_string(),

                        posting_key: "".to_string(),
                        debit_credit_indicator: line.debit_credit.as_char().to_string(),
                        account_type: AccountType::Gl as i32,
                        gl_account: line.account_id.clone(),
                        business_partner: "".to_string(),
                        material: "".to_string(),
                        plant: "".to_string(),
                        item_text: line.text.clone().unwrap_or_default(),
                        assignment_number: "".to_string(),

                        amount_in_document_currency: Some(common_v1::MonetaryValue {
                            value: line.amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        amount_in_local_currency: Some(common_v1::MonetaryValue {
                            value: line.local_amount.to_string(),
                            currency_code: entry.currency.clone(),
                        }),
                        amount_in_group_currency: line.amount_in_group_currency.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line.group_currency.clone().unwrap_or_default(),
                            }
                        }),
                        amount_in_global_currency: None,
                        amount_in_ledger_currency: line.ledger_amount.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: entry.currency.clone(),
                            }
                        }),

                        quantity: None,

                        cost_center: line.cost_center.clone().unwrap_or_default(),
                        profit_center: line.profit_center.clone().unwrap_or_default(),
                        segment: "".to_string(),
                        functional_area: "".to_string(),
                        business_area: line.business_area.clone().unwrap_or_default(),
                        controlling_area: line.controlling_area.clone().unwrap_or_default(),
                        internal_order: "".to_string(),
                        wbs_element: "".to_string(),
                        sales_order: "".to_string(),
                        sales_order_item: 0,

                        tax_code: "".to_string(),
                        tax_jurisdiction: "".to_string(),
                        tax_amount: None,

                        clearing_document: "".to_string(),
                        clearing_date: None,

                        baseline_date: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.baseline_date)
                            .or_else(|| {
                                line.payment_execution
                                    .as_ref()
                                    .and_then(|pe| pe.payment_baseline_date)
                            })
                            .map(|date| prost_types::Timestamp {
                                seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                nanos: 0,
                            }),
                        due_date: line.maturity_date.map(|date| prost_types::Timestamp {
                            seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                            nanos: 0,
                        }),
                        payment_terms: "".to_string(),
                        payment_method: line
                            .payment_execution
                            .as_ref()
                            .map(|pe| pe.payment_method.clone())
                            .unwrap_or_default(),
                        payment_block: line
                            .payment_execution
                            .as_ref()
                            .and_then(|pe| pe.payment_block.clone())
                            .unwrap_or_default(),
                        house_bank: line
                            .payment_execution
                            .as_ref()
                            .and_then(|pe| pe.house_bank.clone())
                            .unwrap_or_default(),

                        special_gl_indicator: line.special_gl_indicator.to_sap_code().to_string(),

                        reference_document_number: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_document_number.clone())
                            .unwrap_or_default(),
                        reference_fiscal_year: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_fiscal_year)
                            .unwrap_or(0),
                        reference_line_item: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_line_item)
                            .unwrap_or(0),
                        reference_document_type: line
                            .invoice_reference
                            .as_ref()
                            .and_then(|ir| ir.reference_document_type.clone())
                            .unwrap_or_default(),

                        transaction_type: line.transaction_type.clone().unwrap_or_default(),
                        reference_transaction_type: line.reference_transaction_type.clone().unwrap_or_default(),
                        reference_key_1: "".to_string(),
                        reference_key_2: "".to_string(),
                        reference_key_3: "".to_string(),

                        financial_area: line.financial_area.clone().unwrap_or_default(),
                        consolidation_unit: "".to_string(),
                        partner_company: line.trading_partner_company.clone().unwrap_or_default(),
                        trading_partner: line.trading_partner_company.clone().unwrap_or_default(),

                        local_currency: entry.currency.clone(),
                        group_currency: line.group_currency.clone().unwrap_or_default(),
                        global_currency: "".to_string(),
                        amount_in_object_currency: line.amount_in_object_currency.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line.object_currency.clone().unwrap_or_default(),
                            }
                        }),
                        amount_in_profit_center_currency: line.amount_in_profit_center_currency.as_ref().map(|amt| {
                            common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: line.profit_center_currency.clone().unwrap_or_default(),
                            }
                        }),

                        dunning_key: line
                            .dunning_detail
                            .as_ref()
                            .and_then(|dd| dd.dunning_key.clone())
                            .unwrap_or_default(),
                        dunning_block: line
                            .dunning_detail
                            .as_ref()
                            .and_then(|dd| dd.dunning_block.clone())
                            .unwrap_or_default(),
                        last_dunning_date: line.dunning_detail.as_ref().and_then(|dd| {
                            dd.last_dunning_date.map(|date| prost_types::Timestamp {
                                seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
                                nanos: 0,
                            })
                        }),
                        dunning_level: line
                            .dunning_detail
                            .as_ref()
                            .map(|dd| dd.dunning_level)
                            .unwrap_or(0),

                        discount_days_1: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.discount_days_1)
                            .unwrap_or(0),
                        discount_days_2: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.discount_days_2)
                            .unwrap_or(0),
                        net_payment_days: line
                            .payment_terms_detail
                            .as_ref()
                            .map(|ptd| ptd.net_payment_days)
                            .unwrap_or(0),
                        discount_percent_1: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_percent_1)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "0".to_string()),
                        discount_percent_2: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_percent_2)
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "0".to_string()),
                        discount_amount: line
                            .payment_terms_detail
                            .as_ref()
                            .and_then(|ptd| ptd.discount_amount)
                            .map(|amt| common_v1::MonetaryValue {
                                value: amt.to_string(),
                                currency_code: entry.currency.clone(),
                            }),

                        sending_cost_center: "".to_string(),
                        partner_profit_center: "".to_string(),
                        sending_financial_area: "".to_string(),

                        account_assignment: line.account_assignment.clone().unwrap_or_default(),

                        local_account: "".to_string(),
                        data_source: "".to_string(),

                        split_method: "".to_string(),
                        manual_split: false,

                        created_by: "".to_string(),
                        created_at: None,
                        created_time: None,
                        changed_by: "".to_string(),
                        changed_at: None,

                        source_module: SourceModule::Gl as i32,

                        extension_fields: std::collections::HashMap::new(),
                    }));
                }
            }
        }

        Ok(None)
    }
}

// 聚合查询统一日记账
pub struct AggregateUniversalJournalHandler<R> {
    repository: Arc<R>,
}

impl<R: JournalRepository> AggregateUniversalJournalHandler<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    pub async fn handle(
        &self,
        request: AggregateUniversalJournalRequest,
    ) -> Result<AggregateUniversalJournalResponse> {
        use rust_decimal::Decimal;
        use std::collections::HashMap;

        let filter = request.filter.unwrap_or_default();

        let company_code = filter
            .company_codes
            .first()
            .map(|s| s.as_str())
            .unwrap_or("");

        // 查询数据
        let entries = self
            .repository
            .search(
                company_code,
                None,
                1,
                10000, // 聚合查询需要更多数据
            )
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        // 根据维度聚合
        let mut aggregation_map: HashMap<String, (Decimal, i64)> = HashMap::new();

        for entry in entries {
            for line in entry.lines {
                // 构建维度键
                let dimension_key = if request
                    .dimensions
                    .contains(&(AggregationDimension::GlAccount as i32))
                {
                    line.account_id.clone()
                } else if request
                    .dimensions
                    .contains(&(AggregationDimension::CostCenter as i32))
                {
                    line.cost_center.clone().unwrap_or_default()
                } else if request
                    .dimensions
                    .contains(&(AggregationDimension::ProfitCenter as i32))
                {
                    line.profit_center.clone().unwrap_or_default()
                } else if request
                    .dimensions
                    .contains(&(AggregationDimension::CompanyCode as i32))
                {
                    entry.company_code.clone()
                } else {
                    "ALL".to_string()
                };

                // 聚合金额
                let amount = match request.measure_field.as_str() {
                    "amount_in_local_currency" => line.local_amount,
                    _ => line.amount,
                };

                let entry = aggregation_map
                    .entry(dimension_key)
                    .or_insert((Decimal::ZERO, 0));
                entry.0 += amount;
                entry.1 += 1;
            }
        }

        // 转换为响应格式
        let results: Vec<aggregate_universal_journal_response::AggregationResult> = aggregation_map
            .into_iter()
            .map(|(key, (sum, count))| {
                let mut dimension_values = HashMap::new();
                dimension_values.insert("dimension".to_string(), key);

                aggregate_universal_journal_response::AggregationResult {
                    dimension_values,
                    measure_value: sum.to_string(),
                    record_count: count,
                }
            })
            .collect();

        Ok(AggregateUniversalJournalResponse { results })
    }
}
