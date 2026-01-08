//! Unit tests for GL Service Domain Layer

#[cfg(test)]
mod journal_entry_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    use crate::domain::entities::{JournalEntry, JournalEntryLine, TaxService, TaxLineItem};
    use crate::domain::value_objects::{
        Account, FiscalPeriod, MonetaryAmount, DebitCreditIndicator, TaxType,
    };

    // ========================================================================
    // Test: 创建凭证草稿
    // ========================================================================
    #[test]
    fn test_create_draft_journal_entry() {
        let entry = JournalEntry::create_draft(
            "1000".to_string(),
            "SA".to_string(),
            NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
            FiscalPeriod::new(2026, 1).unwrap(),
            "CNY".to_string(),
            Uuid::new_v4(),
        );

        assert_eq!(entry.header().company_code, "1000");
        assert_eq!(entry.header().document_type, "SA");
        assert_eq!(entry.header().currency, "CNY");
        assert!(entry.lines().is_empty());
    }

    // ========================================================================
    // Test: 添加行项目
    // ========================================================================
    #[test]
    fn test_add_line_items() {
        let mut entry = create_test_entry();

        let account = Account::gl_account("1001000").unwrap();
        let amount = MonetaryAmount::new(dec!(1000.00), "CNY", DebitCreditIndicator::Debit).unwrap();
        let line = JournalEntryLine::new(1, account, amount);

        let result = entry.add_line(line);
        assert!(result.is_ok());
        assert_eq!(entry.lines().len(), 1);
    }

    // ========================================================================
    // Test: 借贷平衡校验 - 平衡
    // ========================================================================
    #[test]
    fn test_balance_validation_balanced() {
        let mut entry = create_test_entry();

        // 借方 1000
        let debit_account = Account::gl_account("1001000").unwrap();
        let debit_amount = MonetaryAmount::new(dec!(1000.00), "CNY", DebitCreditIndicator::Debit).unwrap();
        entry.add_line(JournalEntryLine::new(1, debit_account, debit_amount)).unwrap();

        // 贷方 1000
        let credit_account = Account::gl_account("2001000").unwrap();
        let credit_amount = MonetaryAmount::new(dec!(1000.00), "CNY", DebitCreditIndicator::Credit).unwrap();
        entry.add_line(JournalEntryLine::new(2, credit_account, credit_amount)).unwrap();

        assert!(entry.is_balanced());
    }

    // ========================================================================
    // Test: 借贷平衡校验 - 不平衡
    // ========================================================================
    #[test]
    fn test_balance_validation_unbalanced() {
        let mut entry = create_test_entry();

        // 只有借方
        let debit_account = Account::gl_account("1001000").unwrap();
        let debit_amount = MonetaryAmount::new(dec!(1000.00), "CNY", DebitCreditIndicator::Debit).unwrap();
        entry.add_line(JournalEntryLine::new(1, debit_account, debit_amount)).unwrap();

        assert!(!entry.is_balanced());
    }

    // ========================================================================
    // Test: 税务计算
    // ========================================================================
    struct MockTaxService;

    impl TaxService for MockTaxService {
        fn get_tax_info(&self, tax_code: &str) -> Option<(TaxType, Decimal)> {
            match tax_code {
                "V1" => Some((TaxType::Input, dec!(0.13))),
                "S1" => Some((TaxType::Output, dec!(0.13))),
                _ => None,
            }
        }
    }

    #[test]
    fn test_tax_calculation() {
        let mut entry = create_test_entry();

        // 添加带税码的行项目
        let account = Account::gl_account("1001000").unwrap();
        let amount = MonetaryAmount::new(dec!(1000.00), "CNY", DebitCreditIndicator::Debit).unwrap();
        let line = JournalEntryLine::new(1, account, amount).with_tax_code("V1");
        entry.add_line(line).unwrap();

        // 计算税务
        let tax_service = MockTaxService;
        entry.calculate_taxes(&tax_service).unwrap();

        assert_eq!(entry.tax_items().len(), 1);
        assert_eq!(entry.tax_items()[0].tax_amount, dec!(130.00)); // 1000 * 13%
    }

    // ========================================================================
    // Helper: 创建测试凭证
    // ========================================================================
    fn create_test_entry() -> JournalEntry {
        JournalEntry::create_draft(
            "1000".to_string(),
            "SA".to_string(),
            NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 8).unwrap(),
            FiscalPeriod::new(2026, 1).unwrap(),
            "CNY".to_string(),
            Uuid::new_v4(),
        )
    }
}
