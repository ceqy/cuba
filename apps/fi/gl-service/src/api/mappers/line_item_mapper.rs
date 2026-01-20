//! LineItem 映射模块
//! 统一处理 JournalEntryLineItem 的 Proto ↔ DTO ↔ Domain 转换

use crate::application::commands::{DunningDetailDTO, InvoiceReferenceDTO, LineItemDTO};
use crate::domain::aggregates::journal_entry::LineItem;
use crate::infrastructure::grpc::fi::gl::v1::{
    DunningDetail as ProtoDunningDetail, InvoiceReference as ProtoInvoiceReference,
    JournalEntryLineItem as ProtoLineItem,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use tonic::Status;

use super::{
    payment_execution_from_proto, payment_execution_to_proto, payment_terms_from_proto,
    payment_terms_to_proto, proto_to_naive_date_opt, str_to_option,
};

/// LineItem: Proto → DTO
/// 用于 create_journal_entry 和 batch_create_journal_entries
pub fn line_item_from_proto(proto: &ProtoLineItem) -> Result<LineItemDTO, Status> {
    // 验证并解析借贷方向
    let debit_credit = match proto.debit_credit_indicator.to_uppercase().as_str() {
        "S" | "DEBIT" => "D",
        "H" | "CREDIT" => "C",
        _ => {
            return Err(Status::invalid_argument(format!(
                "Invalid debit_credit: {}",
                proto.debit_credit_indicator
            )));
        },
    };

    // 解析金额
    let amount = proto
        .amount_in_document_currency
        .as_ref()
        .ok_or_else(|| Status::invalid_argument("amount_in_document_currency is required"))?;
    let amount = Decimal::from_str(&amount.value)
        .map_err(|e| Status::invalid_argument(format!("Invalid amount: {}", e)))?;

    // 解析分类账金额
    let ledger_amount = proto
        .amount_in_ledger_currency
        .as_ref()
        .and_then(|amt| Decimal::from_str(&amt.value).ok());

    // 验证特殊总账标识
    let special_gl_indicator = str_to_option(&proto.special_gl_indicator);

    // 解析业务伙伴类型
    let business_partner_type = match proto.account_type {
        1 => Some("CUSTOMER".to_string()),
        2 => Some("VENDOR".to_string()),
        _ => None,
    };

    // 解析付款执行信息
    let payment_execution = proto
        .payment_execution
        .clone()
        .map(payment_execution_from_proto)
        .transpose()?;

    // 解析付款条件详细信息
    let payment_terms_detail = proto
        .payment_terms_detail
        .clone()
        .map(payment_terms_from_proto)
        .transpose()?;

    let invoice_reference = proto
        .invoice_reference
        .as_ref()
        .map(|ir| InvoiceReferenceDTO {
            reference_document_number: str_to_option(&ir.reference_document_number),
            reference_fiscal_year: if ir.reference_fiscal_year == 0 {
                None
            } else {
                Some(ir.reference_fiscal_year)
            },
            reference_line_item: if ir.reference_line_item == 0 {
                None
            } else {
                Some(ir.reference_line_item)
            },
            reference_document_type: str_to_option(&ir.reference_document_type),
            reference_company_code: str_to_option(&ir.reference_company_code),
        });

    let dunning_detail = proto.dunning_detail.as_ref().map(|dd| DunningDetailDTO {
        dunning_key: str_to_option(&dd.dunning_key),
        dunning_block: str_to_option(&dd.dunning_block),
        last_dunning_date: proto_to_naive_date_opt(dd.last_dunning_date.as_ref()),
        dunning_date: proto_to_naive_date_opt(dd.dunning_date.as_ref()),
        dunning_level: dd.dunning_level,
        dunning_area: str_to_option(&dd.dunning_area),
        grace_period_days: dd.grace_period_days,
        dunning_charges: dd
            .dunning_charges
            .as_ref()
            .and_then(|amt| Decimal::from_str(&amt.value).ok()),
        dunning_clerk: str_to_option(&dd.dunning_clerk),
    });

    let amount_in_object_currency = proto
        .amount_in_object_currency
        .as_ref()
        .and_then(|amt| Decimal::from_str(&amt.value).ok());
    let object_currency = proto
        .amount_in_object_currency
        .as_ref()
        .map(|amt| amt.currency_code.clone());

    let amount_in_profit_center_currency = proto
        .amount_in_profit_center_currency
        .as_ref()
        .and_then(|amt| Decimal::from_str(&amt.value).ok());
    let profit_center_currency = proto
        .amount_in_profit_center_currency
        .as_ref()
        .map(|amt| amt.currency_code.clone());

    let amount_in_group_currency = proto
        .amount_in_group_currency
        .as_ref()
        .and_then(|amt| Decimal::from_str(&amt.value).ok());
    let group_currency = proto
        .amount_in_group_currency
        .as_ref()
        .map(|amt| amt.currency_code.clone());

    Ok(LineItemDTO {
        account_id: proto.gl_account.clone(),
        debit_credit: debit_credit.to_string(),
        amount,
        cost_center: str_to_option(&proto.cost_center),
        profit_center: str_to_option(&proto.profit_center),
        text: str_to_option(&proto.text),
        special_gl_indicator,
        ledger: str_to_option(&proto.ledger),
        ledger_type: if proto.ledger_type == 0 {
            None
        } else {
            Some(proto.ledger_type)
        },
        ledger_amount,
        financial_area: str_to_option(&proto.financial_area),
        business_area: str_to_option(&proto.business_area),
        controlling_area: str_to_option(&proto.controlling_area),
        account_assignment: str_to_option(&proto.account_assignment),
        business_partner: str_to_option(&proto.business_partner),
        business_partner_type,
        maturity_date: proto_to_naive_date_opt(
            proto
                .payment_terms_detail
                .as_ref()
                .and_then(|pt| pt.baseline_date.as_ref())
                .or(proto
                    .payment_execution
                    .as_ref()
                    .and_then(|pe| pe.payment_baseline_date.as_ref())),
        ),
        payment_execution,
        payment_terms_detail,
        invoice_reference,
        dunning_detail,
        transaction_type: str_to_option(&proto.transaction_type),
        reference_transaction_type: str_to_option(&proto.reference_transaction_type),
        trading_partner_company: str_to_option(&proto.trading_partner_company),
        amount_in_object_currency,
        object_currency,
        amount_in_profit_center_currency,
        profit_center_currency,
        amount_in_group_currency,
        group_currency,
    })
}

/// LineItem: Domain → Proto
/// 用于 get_journal_entry 和 query_journal_entries
pub fn line_item_to_proto(domain: &LineItem, currency_code: &str) -> ProtoLineItem {
    let invoice_reference = domain
        .invoice_reference
        .as_ref()
        .map(|ir| ProtoInvoiceReference {
            reference_document_number: ir.reference_document_number.clone().unwrap_or_default(),
            reference_fiscal_year: ir.reference_fiscal_year.unwrap_or(0),
            reference_line_item: ir.reference_line_item.unwrap_or(0),
            reference_document_type: ir.reference_document_type.clone().unwrap_or_default(),
            reference_company_code: ir.reference_company_code.clone().unwrap_or_default(),
        });

    let dunning_detail = domain.dunning_detail.as_ref().map(|dd| ProtoDunningDetail {
        dunning_key: dd.dunning_key.clone().unwrap_or_default(),
        dunning_block: dd.dunning_block.clone().unwrap_or_default(),
        last_dunning_date: dd.last_dunning_date.map(|date| prost_types::Timestamp {
            seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
            nanos: 0,
        }),
        dunning_date: dd.dunning_date.map(|date| prost_types::Timestamp {
            seconds: date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp(),
            nanos: 0,
        }),
        dunning_level: dd.dunning_level,
        dunning_area: dd.dunning_area.clone().unwrap_or_default(),
        grace_period_days: dd.grace_period_days,
        dunning_charges: dd.dunning_charges.map(|amt| {
            crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: currency_code.to_string(),
            }
        }),
        dunning_clerk: dd.dunning_clerk.clone().unwrap_or_default(),
    });

    ProtoLineItem {
        line_item_number: domain.line_number,
        posting_key: String::new(),
        debit_credit_indicator: domain.debit_credit.as_char().to_string(),
        account_type: 0, // GL account
        gl_account: domain.account_id.clone(),
        business_partner: domain.business_partner.clone().unwrap_or_default(),
        text: domain.text.clone().unwrap_or_default(),
        assignment_number: String::new(),

        // 金额字段
        amount_in_document_currency: Some(crate::infrastructure::grpc::common::v1::MonetaryValue {
            value: domain.amount.to_string(),
            currency_code: currency_code.to_string(),
        }),
        amount_in_local_currency: Some(crate::infrastructure::grpc::common::v1::MonetaryValue {
            value: domain.local_amount.to_string(),
            currency_code: currency_code.to_string(),
        }),
        amount_in_group_currency: domain.amount_in_group_currency.as_ref().map(|amt| {
            crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: domain
                    .group_currency
                    .clone()
                    .unwrap_or_else(|| currency_code.to_string()),
            }
        }),
        amount_in_ledger_currency: domain.ledger_amount.as_ref().map(|amt| {
            crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: currency_code.to_string(),
            }
        }),

        // 数量字段
        quantity: None,

        // 成本对象字段
        cost_center: domain.cost_center.clone().unwrap_or_default(),
        profit_center: domain.profit_center.clone().unwrap_or_default(),
        segment: String::new(),
        business_area: domain.business_area.clone().unwrap_or_default(),
        controlling_area: domain.controlling_area.clone().unwrap_or_default(),
        internal_order: String::new(),
        wbs_element: String::new(),

        // 税务字段
        tax_code: String::new(),
        tax_jurisdiction: String::new(),

        // 清账字段
        clearing_document: String::new(),
        clearing_date: None,

        // 特殊总账字段
        special_gl_indicator: domain.special_gl_indicator.to_sap_code().to_string(),

        // 业务交易类型字段
        transaction_type: domain.transaction_type.clone().unwrap_or_default(),
        reference_transaction_type: domain
            .reference_transaction_type
            .clone()
            .unwrap_or_default(),
        trading_partner_company: domain.trading_partner_company.clone().unwrap_or_default(),

        // 组织维度字段
        financial_area: domain.financial_area.clone().unwrap_or_default(),

        // 分类账字段
        ledger: domain.ledger.clone(),
        ledger_type: domain.ledger_type as i32,

        // 多币种字段
        amount_in_object_currency: domain.amount_in_object_currency.as_ref().map(|amt| {
            crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: domain.object_currency.clone().unwrap_or_default(),
            }
        }),
        amount_in_profit_center_currency: domain.amount_in_profit_center_currency.as_ref().map(
            |amt| crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: domain.profit_center_currency.clone().unwrap_or_default(),
            },
        ),

        // 科目分配字段
        account_assignment: domain.account_assignment.clone().unwrap_or_default(),

        // 付款执行和付款条件详细信息
        payment_execution: domain
            .payment_execution
            .as_ref()
            .map(payment_execution_to_proto),
        payment_terms_detail: domain
            .payment_terms_detail
            .as_ref()
            .map(|ptd| payment_terms_to_proto(ptd, currency_code)),

        // 发票参考
        invoice_reference,
        dunning_detail,

        // 内部交易详细信息
        internal_trading_detail: None,
        local_gaap_detail: None,
        field_split_detail: None,
    }
}
