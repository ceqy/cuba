// 集成测试：特殊总账标识功能
use gl_service::domain::aggregates::journal_entry::{JournalEntry, LineItem, DebitCredit, SpecialGlType};
use chrono::NaiveDate;
use rust_decimal_macros::dec;

#[test]
fn test_down_payment_scenario() {
    // 场景：供应商预付款
    let lines = vec![
        LineItem::with_special_gl(
            1,
            "1100".to_string(),  // 银行存款
            DebitCredit::Debit,
            dec!(50000.00),
            dec!(50000.00),
            SpecialGlType::DownPayment,
        ).with_text("预付款给供应商ABC".to_string()),
        LineItem::with_special_gl(
            2,
            "2100".to_string(),  // 应付账款
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
            SpecialGlType::DownPayment,
        ).with_text("预付款给供应商ABC".to_string()),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("预付款凭证".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证
    assert!(entry.has_special_gl_items());
    assert_eq!(entry.get_down_payment_items().len(), 2);
    assert_eq!(entry.calculate_down_payment_amount(), dec!(100000.00));

    // 验证特殊总账类型
    for line in &entry.lines {
        assert_eq!(line.special_gl_indicator, SpecialGlType::DownPayment);
        assert_eq!(line.special_gl_indicator.to_sap_code(), "F");
    }
}

#[test]
fn test_advance_payment_scenario() {
    // 场景：客户预收款
    let lines = vec![
        LineItem::with_special_gl(
            1,
            "1100".to_string(),  // 银行存款
            DebitCredit::Debit,
            dec!(30000.00),
            dec!(30000.00),
            SpecialGlType::AdvancePayment,
        ).with_text("预收款从客户XYZ".to_string()),
        LineItem::with_special_gl(
            2,
            "2200".to_string(),  // 应收账款
            DebitCredit::Credit,
            dec!(30000.00),
            dec!(30000.00),
            SpecialGlType::AdvancePayment,
        ).with_text("预收款从客户XYZ".to_string()),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("预收款凭证".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证
    assert!(entry.has_special_gl_items());
    assert_eq!(entry.get_advance_payment_items().len(), 2);
    assert_eq!(entry.calculate_advance_payment_amount(), dec!(60000.00));

    // 验证特殊总账类型
    for line in &entry.lines {
        assert_eq!(line.special_gl_indicator, SpecialGlType::AdvancePayment);
        assert_eq!(line.special_gl_indicator.to_sap_code(), "V");
    }
}

#[test]
fn test_bill_of_exchange_scenario() {
    // 场景：应收票据
    let lines = vec![
        LineItem::with_special_gl(
            1,
            "1120".to_string(),  // 应收票据
            DebitCredit::Debit,
            dec!(100000.00),
            dec!(100000.00),
            SpecialGlType::BillOfExchange,
        ).with_text("应收票据 - 客户XYZ".to_string()),
        LineItem::new(
            2,
            "4000".to_string(),  // 销售收入
            DebitCredit::Credit,
            dec!(100000.00),
            dec!(100000.00),
        ).with_text("销售收入".to_string()),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("应收票据凭证".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证
    assert!(entry.has_special_gl_items());
    assert_eq!(entry.get_bill_related_items().len(), 1);
    assert_eq!(entry.calculate_bill_amount(), dec!(100000.00));

    // 验证特殊总账类型
    assert_eq!(entry.lines[0].special_gl_indicator, SpecialGlType::BillOfExchange);
    assert_eq!(entry.lines[0].special_gl_indicator.to_sap_code(), "A");
    assert_eq!(entry.lines[1].special_gl_indicator, SpecialGlType::Normal);
}

#[test]
fn test_mixed_special_gl_scenario() {
    // 场景：混合凭证（包含多种特殊总账类型）
    let lines = vec![
        LineItem::with_special_gl(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        ),
        LineItem::with_special_gl(
            2,
            "1120".to_string(),
            DebitCredit::Debit,
            dec!(20000.00),
            dec!(20000.00),
            SpecialGlType::BillOfExchange,
        ),
        LineItem::new(
            3,
            "4000".to_string(),
            DebitCredit::Credit,
            dec!(30000.00),
            dec!(30000.00),
        ),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("混合凭证".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证
    assert!(entry.has_special_gl_items());
    assert!(entry.is_mixed_entry());
    assert_eq!(entry.get_special_gl_types().len(), 2);

    let summary = entry.get_special_gl_summary();
    assert_eq!(summary.len(), 3);

    // 验证分组
    let grouped = entry.group_by_special_gl_type();
    assert_eq!(grouped.len(), 3);
    assert!(grouped.contains_key(&SpecialGlType::DownPayment));
    assert!(grouped.contains_key(&SpecialGlType::BillOfExchange));
    assert!(grouped.contains_key(&SpecialGlType::Normal));
}

#[test]
fn test_clearing_scenario() {
    // 场景：预付款清账
    // 1. 创建预付款凭证
    let down_payment_lines = vec![
        LineItem::with_special_gl(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        ),
        LineItem::with_special_gl(
            2,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        ),
    ];

    let down_payment_entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("预付款".to_string()),
        down_payment_lines,
        None,
    ).unwrap();

    // 2. 创建发票凭证（清账预付款）
    let invoice_lines = vec![
        LineItem::new(
            1,
            "5000".to_string(),  // 费用
            DebitCredit::Debit,
            dec!(15000.00),
            dec!(15000.00),
        ),
        LineItem::with_special_gl(
            2,
            "2100".to_string(),  // 应付账款（清账预付款）
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        ),
        LineItem::new(
            3,
            "2100".to_string(),  // 应付账款（剩余部分）
            DebitCredit::Credit,
            dec!(25000.00),
            dec!(25000.00),
        ),
    ];

    let invoice_entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        "CNY".to_string(),
        Some("发票清账预付款".to_string()),
        invoice_lines,
        None,
    ).unwrap();

    // 验证预付款凭证
    assert!(down_payment_entry.has_special_gl_items());
    assert_eq!(down_payment_entry.calculate_down_payment_amount(), dec!(20000.00));

    // 验证发票凭证包含预付款清账
    assert!(invoice_entry.has_special_gl_items());
    assert_eq!(invoice_entry.get_down_payment_items().len(), 1);
}

#[test]
fn test_special_gl_with_parallel_accounting() {
    // 场景：特殊总账 + 并行会计
    let lines = vec![
        LineItem::builder()
            .line_number(1)
            .account_id("1100".to_string())
            .debit_credit(DebitCredit::Debit)
            .amount(dec!(10000.00))
            .local_amount(dec!(10000.00))
            .special_gl_indicator(SpecialGlType::DownPayment)
            .ledger("0L".to_string())
            .ledger_type(gl_service::domain::aggregates::journal_entry::LedgerType::Leading)
            .build()
            .unwrap(),
        LineItem::builder()
            .line_number(2)
            .account_id("2100".to_string())
            .debit_credit(DebitCredit::Credit)
            .amount(dec!(10000.00))
            .local_amount(dec!(10000.00))
            .special_gl_indicator(SpecialGlType::DownPayment)
            .ledger("0L".to_string())
            .ledger_type(gl_service::domain::aggregates::journal_entry::LedgerType::Leading)
            .build()
            .unwrap(),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("预付款 + 并行会计".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证
    assert!(entry.has_special_gl_items());
    assert_eq!(entry.calculate_down_payment_amount(), dec!(20000.00));

    // 验证并行会计
    for line in &entry.lines {
        assert_eq!(line.ledger, "0L");
        assert_eq!(line.ledger_type, gl_service::domain::aggregates::journal_entry::LedgerType::Leading);
    }
}

#[test]
fn test_special_gl_validation() {
    // 测试特殊总账业务规则验证
    let lines = vec![
        LineItem::with_special_gl(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(10000.00),
            dec!(10000.00),
            SpecialGlType::DownPayment,
        ),
        LineItem::new(
            2,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(10000.00),
            dec!(10000.00),
        ),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap(),
        "CNY".to_string(),
        Some("预付款凭证".to_string()),
        lines,
        None,
    ).unwrap();

    // 验证特殊总账规则
    let validation_result = entry.validate_special_gl_rules();
    assert!(validation_result.is_ok());
}
