//! AR/AP Service - Unit Tests
//!
//! 领域层和应用层单元测试

#[cfg(test)]
mod domain_tests {
    use crate::domain::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    
    // ========================================================================
    // Value Objects Tests
    // ========================================================================
    
    #[test]
    fn test_money_creation() {
        let money = Money::new(dec!(1000.50), Currency::CNY);
        assert_eq!(money.amount, dec!(1000.50));
        assert_eq!(money.currency.code(), "CNY");
    }
    
    #[test]
    fn test_money_zero() {
        let zero = Money::zero(Currency::USD);
        assert!(zero.is_zero());
        assert_eq!(zero.currency.code(), "USD");
    }
    
    #[test]
    fn test_money_positive_negative() {
        let positive = Money::new(dec!(100), Currency::CNY);
        let negative = Money::new(dec!(-100), Currency::CNY);
        
        assert!(positive.is_positive());
        assert!(negative.is_negative());
        assert!(!positive.is_negative());
        assert!(!negative.is_positive());
    }
    
    #[test]
    fn test_money_abs() {
        let negative = Money::new(dec!(-500), Currency::EUR);
        let result = negative.abs();
        assert_eq!(result.amount, dec!(500));
    }
    
    #[test]
    fn test_currency_from_code() {
        assert!(matches!(Currency::from_code("CNY"), Currency::CNY));
        assert!(matches!(Currency::from_code("USD"), Currency::USD));
        assert!(matches!(Currency::from_code("EUR"), Currency::EUR));
        assert!(matches!(Currency::from_code("JPY"), Currency::JPY));
        assert!(matches!(Currency::from_code("GBP"), Currency::GBP));
        assert!(matches!(Currency::from_code("XYZ"), Currency::Other(_)));
    }
    
    #[test]
    fn test_document_reference() {
        let doc_ref = DocumentReference::new(
            "1000".to_string(),
            "1900000001".to_string(),
            2024
        );
        
        assert_eq!(doc_ref.company_code, "1000");
        assert_eq!(doc_ref.document_number, "1900000001");
        assert_eq!(doc_ref.fiscal_year, 2024);
        assert_eq!(doc_ref.to_string(), "1000-2024-1900000001");
    }
    
    #[test]
    fn test_payment_terms_due_date() {
        let terms = PaymentTerms {
            code: "Z001".to_string(),
            description: Some("Net 30".to_string()),
            net_days: 30,
            discount_days_1: Some(10),
            discount_percent_1: Some(dec!(2.0)),
            discount_days_2: None,
            discount_percent_2: None,
        };
        
        let base_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let due_date = terms.calculate_due_date(base_date);
        
        assert_eq!(due_date, chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }
    
    #[test]
    fn test_payment_terms_discount() {
        let terms = PaymentTerms {
            code: "Z001".to_string(),
            description: None,
            net_days: 30,
            discount_days_1: Some(10),
            discount_percent_1: Some(dec!(2.0)),
            discount_days_2: Some(20),
            discount_percent_2: Some(dec!(1.0)),
        };
        
        let invoice_date = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        
        // Within 10 days - 2% discount
        let payment_date_1 = chrono::NaiveDate::from_ymd_opt(2024, 1, 5).unwrap();
        let discount_1 = terms.calculate_discount(dec!(1000), payment_date_1, invoice_date);
        assert_eq!(discount_1, dec!(20));
        
        // Within 20 days - 1% discount
        let payment_date_2 = chrono::NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        let discount_2 = terms.calculate_discount(dec!(1000), payment_date_2, invoice_date);
        assert_eq!(discount_2, dec!(10));
        
        // After 20 days - no discount
        let payment_date_3 = chrono::NaiveDate::from_ymd_opt(2024, 1, 25).unwrap();
        let discount_3 = terms.calculate_discount(dec!(1000), payment_date_3, invoice_date);
        assert_eq!(discount_3, dec!(0));
    }
    
    // ========================================================================
    // Enum Conversion Tests
    // ========================================================================
    
    #[test]
    fn test_clearing_type_conversion() {
        assert_eq!(ClearingType::Full.as_str(), "FULL");
        assert_eq!(ClearingType::Partial.as_str(), "PARTIAL");
        assert_eq!(ClearingType::Automatic.as_str(), "AUTOMATIC");
        assert_eq!(ClearingType::Net.as_str(), "NET");
        
        assert!(matches!(ClearingType::from_str("FULL"), Some(ClearingType::Full)));
        assert!(matches!(ClearingType::from_str("INVALID"), None));
    }
    
    #[test]
    fn test_payment_method_conversion() {
        assert_eq!(PaymentMethod::Wire.as_str(), "WIRE");
        assert_eq!(PaymentMethod::Check.as_str(), "CHECK");
        assert_eq!(PaymentMethod::ACH.as_str(), "ACH");
        assert_eq!(PaymentMethod::Card.as_str(), "CARD");
        
        assert!(matches!(PaymentMethod::from_str("WIRE"), Some(PaymentMethod::Wire)));
        assert!(matches!(PaymentMethod::from_str("INVALID"), None));
    }
    
    #[test]
    fn test_credit_check_result_conversion() {
        assert_eq!(CreditCheckResult::Pass.as_str(), "PASS");
        assert_eq!(CreditCheckResult::Fail.as_str(), "FAIL");
        assert_eq!(CreditCheckResult::Warning.as_str(), "WARNING");
        
        assert!(matches!(CreditCheckResult::from_str("PASS"), Some(CreditCheckResult::Pass)));
    }
    
    #[test]
    fn test_payment_proposal_status_conversion() {
        assert_eq!(PaymentProposalStatus::Draft.as_str(), "DRAFT");
        assert_eq!(PaymentProposalStatus::Approved.as_str(), "APPROVED");
        assert_eq!(PaymentProposalStatus::Executed.as_str(), "EXECUTED");
        assert_eq!(PaymentProposalStatus::Cancelled.as_str(), "CANCELLED");
    }
    
    // ========================================================================
    // Entity Tests
    // ========================================================================
    
    #[test]
    fn test_open_item_is_overdue() {
        let today = chrono::Utc::now().date_naive();
        let yesterday = today - chrono::Duration::days(1);
        let tomorrow = today + chrono::Duration::days(1);
        
        let overdue_item = OpenItem {
            id: uuid::Uuid::new_v4(),
            company_code: "1000".to_string(),
            document_number: "TEST001".to_string(),
            fiscal_year: 2024,
            line_item: 1,
            account_type: AccountType::Customer,
            partner_id: "C001".to_string(),
            posting_date: yesterday,
            due_date: Some(yesterday),
            amount: dec!(1000),
            currency: "CNY".to_string(),
            open_amount: dec!(1000),
            clearing_date: None,
            clearing_doc: None,
            created_at: chrono::Utc::now(),
        };
        
        let not_overdue_item = OpenItem {
            due_date: Some(tomorrow),
            ..overdue_item.clone()
        };
        
        let no_due_date_item = OpenItem {
            due_date: None,
            ..overdue_item.clone()
        };
        
        assert!(overdue_item.is_overdue());
        assert!(!not_overdue_item.is_overdue());
        assert!(!no_due_date_item.is_overdue());
    }
    
    #[test]
    fn test_business_partner_display_name() {
        let org_partner = BusinessPartner {
            id: uuid::Uuid::new_v4(),
            partner_id: "BP001".to_string(),
            partner_type: PartnerType::Organization,
            name_org1: Some("Test Company Ltd".to_string()),
            name_last: None,
            name_first: None,
            search_term: None,
            country: Some("CN".to_string()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        
        let person_partner = BusinessPartner {
            partner_type: PartnerType::Person,
            name_org1: None,
            name_last: Some("Wang".to_string()),
            name_first: Some("Wei".to_string()),
            ..org_partner.clone()
        };
        
        assert_eq!(org_partner.display_name(), "Test Company Ltd");
        assert_eq!(person_partner.display_name(), "Wang Wei");
    }
    
    // ========================================================================
    // Advance Payment Tests
    // ========================================================================
    
    #[test]
    fn test_advance_payment_can_apply() {
        let advance = AdvancePayment {
            id: uuid::Uuid::new_v4(),
            advance_id: "ADV001".to_string(),
            company_code: "1000".to_string(),
            partner_id: "C001".to_string(),
            account_type: AccountType::Customer,
            posting_date: chrono::Utc::now().date_naive(),
            amount: Money::new(dec!(10000), Currency::CNY),
            remaining_amount: Money::new(dec!(5000), Currency::CNY),
            gl_account: Some("113100".to_string()),
            reference: None,
            status: AdvancePaymentStatus::Active,
            created_at: chrono::Utc::now(),
        };
        
        assert!(advance.can_apply(dec!(5000)));
        assert!(advance.can_apply(dec!(1000)));
        assert!(!advance.can_apply(dec!(6000))); // Exceeds remaining
    }
    
    #[test]
    fn test_advance_payment_apply() {
        let mut advance = AdvancePayment {
            id: uuid::Uuid::new_v4(),
            advance_id: "ADV002".to_string(),
            company_code: "1000".to_string(),
            partner_id: "C001".to_string(),
            account_type: AccountType::Customer,
            posting_date: chrono::Utc::now().date_naive(),
            amount: Money::new(dec!(10000), Currency::CNY),
            remaining_amount: Money::new(dec!(10000), Currency::CNY),
            gl_account: None,
            reference: None,
            status: AdvancePaymentStatus::Active,
            created_at: chrono::Utc::now(),
        };
        
        // Apply partial amount
        let result = advance.apply(dec!(3000));
        assert!(result.is_ok());
        assert_eq!(advance.remaining_amount.amount, dec!(7000));
        assert!(matches!(advance.status, AdvancePaymentStatus::Active));
        
        // Apply remaining amount
        let result = advance.apply(dec!(7000));
        assert!(result.is_ok());
        assert_eq!(advance.remaining_amount.amount, dec!(0));
        assert!(matches!(advance.status, AdvancePaymentStatus::FullyApplied));
    }
    
    // ========================================================================
    // Aging Bucket Tests
    // ========================================================================
    
    #[test]
    fn test_aging_bucket_totals() {
        let aging = AgingBucket {
            current: dec!(1000),
            days_1_30: dec!(2000),
            days_31_60: dec!(1500),
            days_61_90: dec!(500),
            over_90_days: dec!(100),
        };
        
        assert_eq!(aging.total(), dec!(5100));
        assert_eq!(aging.overdue_total(), dec!(4100));
    }
}
