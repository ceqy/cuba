// 集成测试：付款条件详细信息持久化
use chrono::NaiveDate;
use gl_service::domain::aggregates::journal_entry::{
    DebitCredit, JournalEntry, LineItem, PaymentTermsDetail,
};
use gl_service::domain::repositories::JournalRepository;
use gl_service::infrastructure::persistence::postgres_journal_repository::PostgresJournalRepository;
use rust_decimal_macros::dec;

/// 测试付款条件详细信息的持久化和查询
#[tokio::test]
#[ignore] // 需要数据库连接，使用 cargo test -- --ignored 运行
async fn test_payment_terms_detail_persistence() {
    // 设置数据库连接
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_test".to_string());
    let config = cuba_database::DatabaseConfig {
        url: database_url,
        ..Default::default()
    };
    let pool = cuba_database::init_pool(&config).await.unwrap();
    let repo = PostgresJournalRepository::new(pool);

    // 创建带付款条件的凭证
    let baseline = NaiveDate::from_ymd_opt(2026, 1, 19).unwrap();
    let payment_terms = PaymentTermsDetail {
        baseline_date: Some(baseline),
        discount_days_1: 10,
        discount_days_2: 20,
        net_payment_days: 30,
        discount_percent_1: Some(dec!(3.0)),
        discount_percent_2: Some(dec!(2.0)),
        discount_amount: None,
    };

    let lines = vec![
        LineItem::new(
            1,
            "5000".to_string(), // 费用科目
            DebitCredit::Debit,
            dec!(100000.00),
            dec!(100000.00),
        ),
        LineItem::new(
            2,
            "2100".to_string(), // 应付账款
            DebitCredit::Credit,
            dec!(100000.00),
            dec!(100000.00),
        )
        .with_payment_terms(payment_terms.clone()),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        "CNY".to_string(),
        Some("采购发票 - 带付款条件".to_string()),
        lines,
        None,
    )
    .unwrap();

    let entry_id = entry.id;

    // 保存到数据库
    repo.save(&entry).await.unwrap();

    // 从数据库读取
    let loaded_entry = repo.find_by_id(&entry_id).await.unwrap().unwrap();

    // 验证付款条件详细信息
    assert_eq!(loaded_entry.lines.len(), 2);
    let line_with_terms = &loaded_entry.lines[1];
    assert!(line_with_terms.payment_terms_detail.is_some());

    let loaded_terms = line_with_terms.payment_terms_detail.as_ref().unwrap();
    assert_eq!(loaded_terms.baseline_date, Some(baseline));
    assert_eq!(loaded_terms.discount_days_1, 10);
    assert_eq!(loaded_terms.discount_days_2, 20);
    assert_eq!(loaded_terms.net_payment_days, 30);
    assert_eq!(loaded_terms.discount_percent_1, Some(dec!(3.0)));
    assert_eq!(loaded_terms.discount_percent_2, Some(dec!(2.0)));

    // 验证折扣计算功能
    let discount_date_1 = loaded_terms.calculate_discount_date_1().unwrap();
    assert_eq!(
        discount_date_1,
        NaiveDate::from_ymd_opt(2026, 1, 29).unwrap()
    );

    let net_due_date = loaded_terms.calculate_net_due_date().unwrap();
    assert_eq!(net_due_date, NaiveDate::from_ymd_opt(2026, 2, 18).unwrap());

    // 测试现金折扣计算
    let invoice_amount = dec!(100000.00);
    let payment_date_early = NaiveDate::from_ymd_opt(2026, 1, 25).unwrap(); // 在第一个折扣期内
    let discount = loaded_terms.calculate_discount_amount(invoice_amount, payment_date_early);
    assert_eq!(discount, dec!(3000.00)); // 100000 * 3% = 3000

    println!("✅ 付款条件详细信息持久化测试通过");
}

/// 测试无付款条件的凭证（NULL 值处理）
#[tokio::test]
#[ignore]
async fn test_null_payment_terms_detail() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_test".to_string());
    let config = cuba_database::DatabaseConfig {
        url: database_url,
        ..Default::default()
    };
    let pool = cuba_database::init_pool(&config).await.unwrap();
    let repo = PostgresJournalRepository::new(pool);

    // 创建不带付款条件的凭证
    let lines = vec![
        LineItem::new(
            1,
            "1100".to_string(),
            DebitCredit::Debit,
            dec!(50000.00),
            dec!(50000.00),
        ),
        LineItem::new(
            2,
            "4000".to_string(),
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
        ),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        "CNY".to_string(),
        Some("普通凭证 - 无付款条件".to_string()),
        lines,
        None,
    )
    .unwrap();

    let entry_id = entry.id;

    // 保存到数据库
    repo.save(&entry).await.unwrap();

    // 从数据库读取
    let loaded_entry = repo.find_by_id(&entry_id).await.unwrap().unwrap();

    // 验证付款条件为 None
    for line in &loaded_entry.lines {
        assert!(line.payment_terms_detail.is_none());
    }

    println!("✅ NULL 付款条件处理测试通过");
}

/// 测试单级折扣付款条件
#[tokio::test]
#[ignore]
async fn test_single_discount_payment_terms() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_test".to_string());
    let config = cuba_database::DatabaseConfig {
        url: database_url,
        ..Default::default()
    };
    let pool = cuba_database::init_pool(&config).await.unwrap();
    let repo = PostgresJournalRepository::new(pool);

    // 创建单级折扣付款条件（2/10 net 30）
    let baseline = NaiveDate::from_ymd_opt(2026, 1, 19).unwrap();
    let payment_terms = PaymentTermsDetail::with_single_discount(Some(baseline), 10, dec!(2.0), 30);

    let lines = vec![
        LineItem::new(
            1,
            "5000".to_string(),
            DebitCredit::Debit,
            dec!(50000.00),
            dec!(50000.00),
        ),
        LineItem::new(
            2,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(50000.00),
            dec!(50000.00),
        )
        .with_payment_terms(payment_terms),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        "CNY".to_string(),
        Some("单级折扣付款条件".to_string()),
        lines,
        None,
    )
    .unwrap();

    let entry_id = entry.id;

    // 保存并读取
    repo.save(&entry).await.unwrap();
    let loaded_entry = repo.find_by_id(&entry_id).await.unwrap().unwrap();

    // 验证单级折扣
    let loaded_terms = loaded_entry.lines[1].payment_terms_detail.as_ref().unwrap();
    assert_eq!(loaded_terms.discount_days_1, 10);
    assert_eq!(loaded_terms.discount_days_2, 0); // 无第二级折扣
    assert_eq!(loaded_terms.discount_percent_1, Some(dec!(2.0)));
    assert!(loaded_terms.discount_percent_2.is_none());

    // 验证付款条件描述
    assert_eq!(loaded_terms.get_terms_description(), "2/10 net 30");

    println!("✅ 单级折扣付款条件测试通过");
}

/// 测试付款条件更新
#[tokio::test]
#[ignore]
async fn test_update_payment_terms_detail() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_test".to_string());
    let config = cuba_database::DatabaseConfig {
        url: database_url,
        ..Default::default()
    };
    let pool = cuba_database::init_pool(&config).await.unwrap();
    let repo = PostgresJournalRepository::new(pool);

    // 创建初始凭证（无付款条件）
    let lines = vec![
        LineItem::new(
            1,
            "5000".to_string(),
            DebitCredit::Debit,
            dec!(80000.00),
            dec!(80000.00),
        ),
        LineItem::new(
            2,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(80000.00),
            dec!(80000.00),
        ),
    ];

    let mut entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
        "CNY".to_string(),
        Some("更新付款条件测试".to_string()),
        lines,
        None,
    )
    .unwrap();

    let entry_id = entry.id;

    // 保存初始版本
    repo.save(&entry).await.unwrap();

    // 更新：添加付款条件
    let baseline = NaiveDate::from_ymd_opt(2026, 1, 19).unwrap();
    let payment_terms = PaymentTermsDetail::with_single_discount(Some(baseline), 15, dec!(2.5), 45);

    entry.lines[1].payment_terms_detail = Some(payment_terms);

    // 保存更新后的版本
    repo.save(&entry).await.unwrap();

    // 读取并验证
    let loaded_entry = repo.find_by_id(&entry_id).await.unwrap().unwrap();
    let loaded_terms = loaded_entry.lines[1].payment_terms_detail.as_ref().unwrap();

    assert_eq!(loaded_terms.discount_days_1, 15);
    assert_eq!(loaded_terms.net_payment_days, 45);
    assert_eq!(loaded_terms.discount_percent_1, Some(dec!(2.5)));

    println!("✅ 付款条件更新测试通过");
}

/// 测试 JSONB 查询功能（通过 baseline_date 查询）
#[tokio::test]
#[ignore]
async fn test_query_by_baseline_date() {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/cuba_test".to_string());
    let config = cuba_database::DatabaseConfig {
        url: database_url,
        ..Default::default()
    };
    let pool = cuba_database::init_pool(&config).await.unwrap();

    // 创建测试数据
    let baseline = NaiveDate::from_ymd_opt(2026, 1, 20).unwrap();
    let payment_terms = PaymentTermsDetail::with_single_discount(Some(baseline), 10, dec!(2.0), 30);

    let repo = PostgresJournalRepository::new(pool.clone());
    let lines = vec![
        LineItem::new(
            1,
            "5000".to_string(),
            DebitCredit::Debit,
            dec!(60000.00),
            dec!(60000.00),
        ),
        LineItem::new(
            2,
            "2100".to_string(),
            DebitCredit::Credit,
            dec!(60000.00),
            dec!(60000.00),
        )
        .with_payment_terms(payment_terms),
    ];

    let entry = JournalEntry::new(
        "1000".to_string(),
        2026,
        NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        NaiveDate::from_ymd_opt(2026, 1, 20).unwrap(),
        "CNY".to_string(),
        Some("JSONB 查询测试".to_string()),
        lines,
        None,
    )
    .unwrap();

    repo.save(&entry).await.unwrap();

    // 使用原生 SQL 查询（测试 JSONB 索引）
    let result = sqlx::query(
        r#"
        SELECT COUNT(*) as count
        FROM journal_entry_lines
        WHERE payment_terms_detail->>'baseline_date' = $1
        "#,
    )
    .bind(baseline.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();

    let count: i64 = result.get("count");
    assert!(count >= 1, "应该能查询到至少一条记录");

    println!("✅ JSONB 查询功能测试通过");
}
