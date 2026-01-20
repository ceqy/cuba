use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BankStatement {
    pub statement_id: Uuid,
    pub company_code: String,
    pub statement_format: String,
    pub status: String,
    pub house_bank: Option<String>,
    pub bank_account: Option<String>,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub transactions: Vec<StatementTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatementTransaction {
    pub transaction_id: Uuid,
    pub statement_id: Uuid,
    pub value_date: NaiveDate,
    pub amount: Decimal,
    pub currency: String,
    pub memo: Option<String>,
    pub partner_name: Option<String>,
    pub transaction_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentRun {
    pub run_id: Uuid,
    pub run_number: String,
    pub company_codes: Option<String>,
    pub posting_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    #[sqlx(skip)]
    pub documents: Vec<PaymentDocument>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentDocument {
    pub doc_id: Uuid,
    pub run_id: Uuid,
    pub document_number: String,
    pub fiscal_year: Option<i32>,
    pub amount: Decimal,
    pub currency: String,
    pub payee_name: Option<String>,
    pub payment_method: Option<String>,
    pub house_bank: Option<String>,
    pub bank_account: Option<String>,
    pub transaction_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_statement_supports_bank_fields() {
        let stmt = BankStatement {
            statement_id: Uuid::new_v4(),
            company_code: "1000".to_string(),
            statement_format: "MT940".to_string(),
            status: "PROCESSED".to_string(),
            house_bank: Some("HB01".to_string()),
            bank_account: Some("123456".to_string()),
            created_at: Utc::now(),
            transactions: Vec::new(),
        };

        assert_eq!(stmt.house_bank.as_deref(), Some("HB01"));
        assert_eq!(stmt.bank_account.as_deref(), Some("123456"));
    }

    #[test]
    fn payment_document_supports_payment_fields() {
        let doc = PaymentDocument {
            doc_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            document_number: "1500001".to_string(),
            fiscal_year: Some(2026),
            amount: Decimal::new(5000, 2),
            currency: "CNY".to_string(),
            payee_name: Some("Vendor ABC".to_string()),
            payment_method: Some("T".to_string()),
            house_bank: Some("HB01".to_string()),
            bank_account: Some("123456".to_string()),
            transaction_type: Some("TR".to_string()),
        };

        assert_eq!(doc.payment_method.as_deref(), Some("T"));
        assert_eq!(doc.transaction_type.as_deref(), Some("TR"));
    }
}
