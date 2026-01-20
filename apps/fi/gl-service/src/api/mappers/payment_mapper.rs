//! 付款相关字段映射
//! 统一处理 PaymentExecution 和 PaymentTermsDetail 的转换

use crate::application::commands::{PaymentExecutionDTO, PaymentTermsDetailDTO};
use crate::domain::aggregates::journal_entry::{PaymentExecutionDetail, PaymentTermsDetail};
use crate::infrastructure::grpc::fi::gl::v1::{
    PaymentExecutionDetail as ProtoPaymentExecution, PaymentTermsDetail as ProtoPaymentTerms,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use tonic::Status;

use super::{naive_date_to_proto_opt, proto_to_naive_date_opt, str_to_option};

/// PaymentExecution: Proto → DTO
pub fn payment_execution_from_proto(
    proto: ProtoPaymentExecution,
) -> Result<PaymentExecutionDTO, Status> {
    Ok(PaymentExecutionDTO {
        payment_method: proto.payment_method,
        house_bank: str_to_option(&proto.house_bank),
        partner_bank_type: str_to_option(&proto.partner_bank_type),
        payment_block: str_to_option(&proto.payment_block),
        payment_baseline_date: proto_to_naive_date_opt(proto.payment_baseline_date.as_ref()),
        payment_reference: str_to_option(&proto.payment_reference),
        payment_priority: if proto.payment_priority == 0 {
            None
        } else {
            Some(proto.payment_priority)
        },
    })
}

/// PaymentExecution: Domain → Proto
pub fn payment_execution_to_proto(domain: &PaymentExecutionDetail) -> ProtoPaymentExecution {
    ProtoPaymentExecution {
        payment_method: domain.payment_method.clone(),
        house_bank: domain.house_bank.clone().unwrap_or_default(),
        partner_bank_type: domain.partner_bank_type.clone().unwrap_or_default(),
        payment_block: domain.payment_block.clone().unwrap_or_default(),
        payment_baseline_date: naive_date_to_proto_opt(domain.payment_baseline_date),
        payment_reference: domain.payment_reference.clone().unwrap_or_default(),
        payment_priority: domain.payment_priority.unwrap_or(0),
    }
}

/// PaymentTermsDetail: Proto → DTO
pub fn payment_terms_from_proto(proto: ProtoPaymentTerms) -> Result<PaymentTermsDetailDTO, Status> {
    // 解析折扣百分比
    let discount_percent_1 = if proto.discount_percent_1.is_empty() {
        None
    } else {
        Decimal::from_str(&proto.discount_percent_1)
            .map_err(|e| Status::invalid_argument(format!("Invalid discount_percent_1: {}", e)))?
            .into()
    };

    let discount_percent_2 = if proto.discount_percent_2.is_empty() {
        None
    } else {
        Decimal::from_str(&proto.discount_percent_2)
            .map_err(|e| Status::invalid_argument(format!("Invalid discount_percent_2: {}", e)))?
            .into()
    };

    // 解析折扣金额
    let discount_amount = proto
        .discount_amount
        .and_then(|amt| Decimal::from_str(&amt.value).ok());

    Ok(PaymentTermsDetailDTO {
        baseline_date: proto_to_naive_date_opt(proto.baseline_date.as_ref()),
        discount_days_1: proto.discount_days_1,
        discount_days_2: proto.discount_days_2,
        net_payment_days: proto.net_payment_days,
        discount_percent_1,
        discount_percent_2,
        discount_amount,
    })
}

/// PaymentTermsDetail: Domain → Proto
pub fn payment_terms_to_proto(
    domain: &PaymentTermsDetail,
    currency_code: &str,
) -> ProtoPaymentTerms {
    ProtoPaymentTerms {
        baseline_date: naive_date_to_proto_opt(domain.baseline_date),
        discount_days_1: domain.discount_days_1,
        discount_days_2: domain.discount_days_2,
        net_payment_days: domain.net_payment_days,
        discount_percent_1: domain
            .discount_percent_1
            .map(|d| d.to_string())
            .unwrap_or_default(),
        discount_percent_2: domain
            .discount_percent_2
            .map(|d| d.to_string())
            .unwrap_or_default(),
        discount_amount: domain.discount_amount.map(|amt| {
            crate::infrastructure::grpc::common::v1::MonetaryValue {
                value: amt.to_string(),
                currency_code: currency_code.to_string(),
            }
        }),
    }
}
