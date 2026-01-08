//! Proto to Domain Mapper
//!
//! 将 Proto 消息转换为领域对象

use chrono::{NaiveDate, DateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;
use prost_types::Timestamp;

use crate::application::{CreateJournalEntryCommand, CreateLineItemCommand, UpdateJournalEntryCommand};
use crate::proto::finance::gl::*;
use crate::proto::common::SystemDocumentReference;
use crate::domain::entities::{JournalEntry, JournalEntryHeader as DomainHeader, JournalEntryLine as DomainLine};
use crate::domain::value_objects::{AccountType, DebitCreditIndicator};

/// 将 CreateJournalEntryRequest 转换为 CreateJournalEntryCommand
pub fn map_create_request(
    request: CreateJournalEntryRequest,
    created_by: Uuid,
) -> anyhow::Result<CreateJournalEntryCommand> {
    let header = request.header.ok_or_else(|| anyhow::anyhow!("Header is required"))?;

    // 解析日期
    let document_date = parse_date(header.document_date.as_ref())?;
    let posting_date = parse_date(header.posting_date.as_ref())?;

    // 转换行项目
    let lines: Vec<CreateLineItemCommand> = request
        .line_items
        .into_iter()
        .map(|line| {
            let dc_indicator = if line.debit_credit_indicator == "H" {
                "H".to_string()
            } else {
                "S".to_string()
            };

            // 解析金额 - 使用 amount_doc 字段 (原: WRBTR)
            let amount = Decimal::from_str(&line.amount_doc).unwrap_or_default();

            CreateLineItemCommand {
                gl_account: line.gl_account,
                amount,
                debit_credit: dc_indicator,
                cost_center: if line.cost_center.is_empty() { None } else { Some(line.cost_center) },
                profit_center: if line.profit_center.is_empty() { None } else { Some(line.profit_center) },
                line_text: None,
                tax_code: if line.tax_code.is_empty() { None } else { Some(line.tax_code) },
            }
        })
        .collect();

    Ok(CreateJournalEntryCommand {
        company_code: header.company_code,
        document_type: header.document_type,
        document_date,
        posting_date,
        fiscal_year: header.fiscal_year,
        fiscal_period: header.fiscal_period,
        currency: header.currency,
        header_text: if header.header_text.is_empty() { None } else { Some(header.header_text) },
        lines,
        created_by,
        exchange_rate: Decimal::from_str(&header.exchange_rate).ok(),
    })
}

/// 将 UpdateJournalEntryRequest 转换为 UpdateJournalEntryCommand
pub fn map_update_request(
    request: UpdateJournalEntryRequest,
) -> anyhow::Result<UpdateJournalEntryCommand> {
    let id = Uuid::parse_str(&request.journal_entry_id)?;
    
    let header_text = request.header.as_ref().and_then(|h| {
        if h.header_text.is_empty() { None } else { Some(h.header_text.clone()) }
    });
    
    let lines = if request.line_items.is_empty() {
        None
    } else {
        let items = request.line_items.into_iter().map(|line| {
            let dc_indicator = if line.debit_credit_indicator == "H" { "H".to_string() } else { "S".to_string() };
            let amount = Decimal::from_str(&line.amount_doc).unwrap_or_default();
            CreateLineItemCommand {
                gl_account: line.gl_account,
                amount,
                debit_credit: dc_indicator,
                cost_center: if line.cost_center.is_empty() { None } else { Some(line.cost_center) },
                profit_center: if line.profit_center.is_empty() { None } else { Some(line.profit_center) },
                line_text: None,
                tax_code: if line.tax_code.is_empty() { None } else { Some(line.tax_code) },
            }
        }).collect();
        Some(items)
    };

    Ok(UpdateJournalEntryCommand {
        id,
        document_date: None,
        posting_date: None,
        header_text,
        lines,
    })
}

/// 将 ParkJournalEntryRequest 转换为 CreateJournalEntryCommand
pub fn map_park_request(
    request: ParkJournalEntryRequest,
) -> anyhow::Result<CreateJournalEntryCommand> {
    let header = request.header.ok_or_else(|| anyhow::anyhow!("Missing header in ParkJournalEntryRequest"))?;
    
    let line_items = request.line_items.into_iter().map(|line| {
        let dc_indicator = if line.debit_credit_indicator == "H" { "H".to_string() } else { "S".to_string() };
        let amount = Decimal::from_str(&line.amount_doc).unwrap_or_default();
        CreateLineItemCommand {
            gl_account: line.gl_account,
            amount,
            debit_credit: dc_indicator,
            cost_center: if line.cost_center.is_empty() { None } else { Some(line.cost_center) },
            profit_center: if line.profit_center.is_empty() { None } else { Some(line.profit_center) },
            line_text: None,
            tax_code: if line.tax_code.is_empty() { None } else { Some(line.tax_code) },
        }
    }).collect();

    Ok(CreateJournalEntryCommand {
        company_code: header.company_code,
        document_type: header.document_type,
        document_date: chrono::NaiveDateTime::from_timestamp_opt(
            header.document_date.as_ref().map(|ts| ts.seconds).unwrap_or(0),
            header.document_date.as_ref().map(|ts| ts.nanos).unwrap_or(0) as u32
        ).map(|dt| dt.date()).unwrap_or_else(|| Utc::now().naive_utc().date()),
        posting_date: chrono::NaiveDateTime::from_timestamp_opt(
            header.posting_date.as_ref().map(|ts| ts.seconds).unwrap_or(0),
            header.posting_date.as_ref().map(|ts| ts.nanos).unwrap_or(0) as u32
        ).map(|dt| dt.date()).unwrap_or_else(|| Utc::now().naive_utc().date()),
        currency: header.currency,
        header_text: if header.header_text.is_empty() { None } else { Some(header.header_text) },
        lines: line_items,
        created_by: Uuid::nil(), // TODO: Get from context
        fiscal_year: header.fiscal_year,
        fiscal_period: header.fiscal_period,
        exchange_rate: Decimal::from_str(&header.exchange_rate).ok(),
    })
}

/// 将 JournalEntry 转换为 ParkJournalEntryResponse
pub fn map_to_park_response(entry: &JournalEntry) -> ParkJournalEntryResponse {
    ParkJournalEntryResponse {
        success: true,
        parked_document_reference: Some(SystemDocumentReference {
            company_code: entry.header().company_code.clone(),
            fiscal_year: entry.header().fiscal_period.year(),
            document_number: entry.document_number().map(|d| d.number().to_string()).unwrap_or_default(),
            document_type: entry.header().document_type.clone(),
            document_category: String::new(),
        }),
        messages: Vec::new(),
    }
}

/// 将 JournalEntry 转换为 JournalEntryResponse
pub fn map_to_response(entry: &JournalEntry) -> JournalEntryResponse {
    JournalEntryResponse {
        success: true,
        document_reference: Some(crate::proto::common::SystemDocumentReference {
            document_number: entry.document_number().map(|d| d.number().to_string()).unwrap_or_else(|| entry.id().to_string()),
            fiscal_year: entry.header().fiscal_period.year(),
            company_code: entry.header().company_code.clone(),
            document_type: entry.header().document_type.clone(),
            document_category: "G".to_string(),
        }),
        messages: vec![],
    }
}

/// 将 JournalEntry 转换为 JournalEntryDetail
pub fn map_to_detail(entry: &JournalEntry) -> JournalEntryDetail {
    JournalEntryDetail {
        document_reference: Some(crate::proto::common::SystemDocumentReference {
            document_number: entry.document_number().map(|d| d.number().to_string()).unwrap_or_else(|| entry.id().to_string()),
            fiscal_year: entry.header().fiscal_period.year(),
            company_code: entry.header().company_code.clone(),
            document_type: entry.header().document_type.clone(),
            document_category: "G".to_string(),
        }),
        header: Some(map_header_to_proto(entry.header())),
        line_items: entry.lines().iter().map(map_line_to_proto).collect(),
        tax_items: entry.tax_items().iter().map(map_tax_line_to_proto).collect(),
        one_time_accounts: vec![],
        attachments: vec![],
        approval_history: vec![],
        audit_data: Some(crate::proto::common::AuditData {
            created_by: entry.header().created_by.to_string(),
            created_at: Some(to_timestamp(entry.header().created_at)),
            changed_by: entry.header().created_by.to_string(), // Placeholder
            changed_at: Some(to_timestamp(entry.header().updated_at)),
        }),
    }
}

/// 将领域列表转换为摘要列表
pub fn map_items_to_summary(entries: Vec<JournalEntry>) -> Vec<JournalEntrySummary> {
    entries.into_iter().map(|entry| {
        JournalEntrySummary {
            document_reference: Some(crate::proto::common::SystemDocumentReference {
                document_number: entry.document_number().map(|d| d.number().to_string()).unwrap_or_else(|| entry.id().to_string()),
                fiscal_year: entry.header().fiscal_period.year(),
                company_code: entry.header().company_code.clone(),
                document_type: entry.header().document_type.clone(),
                document_category: "G".to_string(),
            }),
            document_date: Some(to_timestamp_from_date(entry.header().document_date)),
            posting_date: Some(to_timestamp_from_date(entry.header().posting_date)),
            header_text: entry.header().header_text.clone().unwrap_or_default(),
            created_by: entry.header().created_by.to_string(),
            status: entry.status() as i32,
            currency: entry.header().currency.clone(),
            total_amount: entry.total_debit().to_string(),
            fiscal_period: entry.header().fiscal_period.period(),
            reference_document: entry.header().reference_document.clone().unwrap_or_default(),
        }
    }).collect()
}

fn map_header_to_proto(h: &DomainHeader) -> JournalEntryHeader {
    JournalEntryHeader {
        company_code: h.company_code.clone(),
        document_type: h.document_type.clone(),
        document_date: Some(to_timestamp_from_date(h.document_date)),
        posting_date: Some(to_timestamp_from_date(h.posting_date)),
        fiscal_period: h.fiscal_period.period(),
        fiscal_year: h.fiscal_period.year(),
        header_text: h.header_text.clone().unwrap_or_default(),
        reference_document: h.reference_document.clone().unwrap_or_default(),
        currency: h.currency.clone(),
        exchange_rate: h.exchange_rate.to_string(),
        local_currency: h.local_currency.clone(),
        created_by: h.created_by.to_string(),
        ledger: h.ledger.clone(),
        ..Default::default()
    }
}

fn map_line_to_proto(l: &DomainLine) -> JournalEntryLineItem {
    JournalEntryLineItem {
        line_item_number: l.line_number,
        debit_credit_indicator: l.amount.dc_indicator().as_str().to_string(),
        gl_account: l.account.get_gl_account().to_string(),
        customer_number: l.account.subledger_account().filter(|_| matches!(l.account.account_type(), AccountType::Customer)).map(|s| s.to_string()).unwrap_or_default(),
        vendor_number: l.account.subledger_account().filter(|_| matches!(l.account.account_type(), AccountType::Vendor)).map(|s| s.to_string()).unwrap_or_default(),
        account_type: l.account.account_type().as_str().to_string(),
        amount_doc: l.amount.amount().to_string(),
        amount_local: l.amount_local.map(|v| v.to_string()).unwrap_or_default(),
        cost_center: l.cost_objects.cost_center.clone().unwrap_or_default(),
        profit_center: l.cost_objects.profit_center.clone().unwrap_or_default(),
        business_area: l.cost_objects.business_area.clone().unwrap_or_default(),
        segment: l.cost_objects.segment.clone().unwrap_or_default(),
        functional_area: l.cost_objects.functional_area.clone().unwrap_or_default(),
        internal_order: l.cost_objects.internal_order.clone().unwrap_or_default(),
        wbs_element: l.cost_objects.wbs_element.clone().unwrap_or_default(),
        tax_code: l.tax_code.clone().unwrap_or_default(),
        ..Default::default()
    }
}

fn map_tax_line_to_proto(t: &crate::domain::entities::TaxLineItem) -> TaxLineItem {
    TaxLineItem {
        line_item_number: t.line_number,
        tax_code: t.tax_code.clone(),
        tax_rate: t.tax_rate.to_string(),
        tax_base_amount_doc: t.tax_base_amount.to_string(),
        tax_amount_doc: t.tax_amount.to_string(),
        tax_type: t.tax_type.as_str().to_string(),
        debit_credit_indicator: t.dc_indicator.as_str().to_string(),
        ..Default::default()
    }
}

/// 解析 Protobuf Timestamp 为 NaiveDate
fn parse_date(ts: Option<&Timestamp>) -> anyhow::Result<NaiveDate> {
    match ts {
        Some(ts) => {
            let dt = DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                .ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;
            Ok(dt.date_naive())
        }
        None => Ok(Utc::now().date_naive()),
    }
}

fn to_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn to_timestamp_from_date(date: NaiveDate) -> Timestamp {
    let dt = date.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap();
    to_timestamp(dt)
}
