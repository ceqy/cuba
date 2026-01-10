# Shared Infrastructure Libraries Usage Guide

此文档介绍如何在微服务中集成和使用 `cuba-*` 核心库。

## 1. 引入依赖 (Cargo.toml)

在各个微服务（如 `gl-service`）中，引入必要的库：

```toml
[dependencies]
# ... 其他依赖
tonic = "0.12" # 确保版本兼容

# 核心库
cuba-core = { path = "../../../libs/cuba-core" }
cuba-database = { path = "../../../libs/cuba-database" }
cuba-errors = { path = "../../../libs/cuba-errors" }
# cuba-messaging = { path = "../../../libs/cuba-messaging" } # 需要消息时引入
```

---

## 2. 数据库集成 (cuba-database)

### 初始化连接池 (main.rs)
使用 `PostgresDb` 替代直接使用 sqlx，它预置了最佳实践配置（超时、连接数）。

```rust
use cuba_database::PostgresDb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 初始化数据库
    let db_url = std::env::var("DATABASE_URL")?;
    let db = PostgresDb::new(&db_url).await?; // 自动处理连接池配置

    // 2. 注入到 Service 层
    let my_service = MyService::new(db);
    
    // ... 启动 gRPC Server
}
```

### 在 Repository 中使用
Repository 应该持有 `PostgresDb` 实例。

```rust
use cuba_database::PostgresDb;
use cuba_errors::ServiceError;

pub struct PostgresJournalEntryRepository {
    db: PostgresDb,
}

impl PostgresJournalEntryRepository {
    pub async fn find_by_id(&self, id: &str) -> Result<Option<JournalEntry>, ServiceError> {
        // 直接访问内部 pool
        let row = sqlx::query_as!(
            JournalEntryModel,
            "SELECT * FROM journal_entries WHERE id = $1",
            id
        )
        .fetch_optional(self.db.pool()) // 使用 .pool()
        .await
        .map_err(ServiceError::DatabaseError)?; // 自动映射 sqlx 错误

        Ok(row.map(|m| m.into_domain()))
    }
}
```

### 使用事务 (UnitOfWork)
`cuba-database` 提供了简单的事务抽象。

```rust
use cuba_database::UnitOfWork;

// 开启事务
let mut tx = db.begin().await?;

sqlx::query("INSERT INTO ...").execute(&mut *tx).await?;
sqlx::query("UPDATE ...").execute(&mut *tx).await?;

tx.commit().await?; // 提交事务
```

---

## 3. 错误处理 (cuba-errors)

这是最强大的功能之一。您不再需要手动将数据库错误转换为 gRPC 状态码。

### 定义 Service 方法
只需返回 `Result<T, ServiceError>`，`?` 运算符会自动转换错误，gRPC 层会自动映射到正确的 Status Code。

```rust
use cuba_errors::ServiceError;
use tonic::{Request, Response, Status};

// gRPC Handler
async fn create_journal_entry(
    &self,
    request: Request<CreateJournalEntryRequest>,
) -> Result<Response<CreateJournalEntryResponse>, Status> { // 注意这里返回 generic tonic::Status
    
    // 1. 调用业务逻辑 (Domain Layer)
    // 如果 domain_logic 返回 ServiceError::InvalidInput, 
    // 这里用 ? 传播，use cuba_errors::ServiceError 的 From<ServiceError> for Status 实现会自动转换
    let result = self.domain_logic.create(request.into_inner()).await?; 
    
    Ok(Response::new(result))
}

// Domain Layer
async fn domain_logic(&self, cmd: CreateCommand) -> Result<Id, ServiceError> {
    // 校验失败 -> 自动映射为 gRPC INVALID_ARGUMENT
    if cmd.amount < 0 {
        return Err(ServiceError::InvalidInput("Amount must be positive".into()));
    }

    // 数据库唯一键冲突 -> 自动映射为 gRPC ALREADY_EXISTS
    self.repo.save(&entity).await?; 

    Ok(entity.id)
}
```

---

## 4. 核心类型使用 (cuba-core)

统一全系统的类型定义，避免每个服务重复定义。

```rust
use cuba_core::{
    RequestContext, 
    MonetaryValue, 
    PageRequest,
    Decimal
};

// 1. 请求上下文 (通常从 gRPC metadata 提取)
let ctx = RequestContext {
    request_id: "req-123".into(),
    tenant_id: "T001".into(),
    user_id: "U999".into(),
    trace_id: "trace-abc".into(),
    ..Default::default()
};

// 2. 金额处理 (使用 rust_decimal)
let price = MonetaryValue::new(Decimal::new(10050, 2), "USD"); // 100.50 USD

// 3. 分页请求
let page_req = PageRequest {
    page: 1,
    page_size: 20,
    ..Default::default()
};
```
