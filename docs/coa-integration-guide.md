# COA 服务集成指南

## 概述

本文档说明如何将 COA（Chart of Accounts）服务集成到 GL/AP/AR 等财务服务中，实现科目验证功能。

## 架构设计

```
┌─────────────┐         ┌─────────────┐
│  GL Service │────────>│ COA Service │
│  (50052)    │  gRPC   │  (50060)    │
└─────────────┘         └─────────────┘
       │
       │ validates
       │ accounts
       ▼
┌─────────────┐
│  Database   │
└─────────────┘
```

## 集成步骤

### 1. 在 GL Service 中添加 COA 客户端

#### 1.1 更新 build.rs

在 `apps/fi/gl-service/build.rs` 中添加 COA proto 编译：

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    // Compile GL service protos
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("gl_descriptor.bin"))
        .compile_protos(
            &["../../../protos/fi/gl/gl.proto", "../../../protos/common/common.proto"],
            &["../../../protos", "../../../third_party"],
        )?;

    // Compile COA service protos (client only)
    tonic_prost_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(
            &["../../../protos/fi/coa/coa.proto"],
            &["../../../protos", "../../../third_party"],
        )?;

    Ok(())
}
```

#### 1.2 创建 COA 客户端

创建 `apps/fi/gl-service/src/infrastructure/clients/coa_client.rs`：

```rust
use tonic::transport::Channel;
use chrono::NaiveDate;

pub mod coa_proto {
    tonic::include_proto!("fi.coa.v1");
}

use coa_proto::chart_of_accounts_service_client::ChartOfAccountsServiceClient;

#[derive(Clone)]
pub struct CoaClient {
    client: ChartOfAccountsServiceClient<Channel>,
}

impl CoaClient {
    pub async fn connect(endpoint: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = ChartOfAccountsServiceClient::connect(endpoint.to_string()).await?;
        Ok(Self { client })
    }

    pub async fn validate_account(
        &mut self,
        chart_code: &str,
        account_code: &str,
        company_code: Option<&str>,
        posting_date: NaiveDate,
    ) -> Result<AccountValidationResult, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation...
    }
}
```

#### 1.3 创建领域服务

创建 `apps/fi/gl-service/src/domain/services/account_validation.rs`：

```rust
use std::sync::Arc;
use crate::infrastructure::clients::{CoaClient, AccountValidationResult};

pub struct AccountValidationService {
    coa_client: Arc<tokio::sync::Mutex<CoaClient>>,
    chart_code: String,
}

impl AccountValidationService {
    pub async fn validate_journal_entry_accounts(
        &self,
        account_codes: Vec<String>,
        company_code: &str,
        posting_date: NaiveDate,
    ) -> Result<Vec<AccountValidationResult>, Box<dyn std::error::Error + Send + Sync>> {
        // Batch validate all accounts
        let results = self.batch_validate_accounts(account_codes, Some(company_code)).await?;

        // Check for invalid accounts
        let invalid_accounts: Vec<_> = results.iter().filter(|r| !r.is_valid).collect();

        if !invalid_accounts.is_empty() {
            let error_msg = invalid_accounts
                .iter()
                .filter_map(|r| r.get_error_message())
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!("科目验证失败: {}", error_msg).into());
        }

        Ok(results)
    }
}
```

#### 1.4 更新 Handler

在 `apps/fi/gl-service/src/application/handlers.rs` 中集成验证：

```rust
pub struct CreateJournalEntryHandler<R> {
    repository: Arc<R>,
    account_validation: Option<Arc<AccountValidationService>>,
}

impl<R: JournalRepository> CreateJournalEntryHandler<R> {
    pub fn with_account_validation(mut self, validation: Arc<AccountValidationService>) -> Self {
        self.account_validation = Some(validation);
        self
    }

    pub async fn handle(&self, cmd: CreateJournalEntryCommand) -> Result<JournalEntry, Box<dyn std::error::Error + Send + Sync>> {
        // Validate accounts if COA service is available
        if let Some(validator) = &self.account_validation {
            let account_codes: Vec<String> = cmd.lines.iter()
                .map(|l| l.account_id.clone())
                .collect();

            validator.validate_journal_entry_accounts(
                account_codes,
                &cmd.company_code,
                cmd.posting_date.naive_local().date(),
            ).await?;
        }

        // Continue with journal entry creation...
    }
}
```

#### 1.5 更新 main.rs

在 `apps/fi/gl-service/src/main.rs` 中初始化 COA 客户端：

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize COA client (optional - gracefully degrade if unavailable)
    let coa_endpoint = std::env::var("COA_SERVICE_URL")
        .unwrap_or_else(|_| "http://coa-service.finance.svc.cluster.local:50060".to_string());

    let account_validation = match CoaClient::connect(&coa_endpoint).await {
        Ok(coa_client) => {
            info!("Connected to COA service at {}", coa_endpoint);
            let chart_code = std::env::var("CHART_OF_ACCOUNTS")
                .unwrap_or_else(|_| "CN01".to_string());
            Some(Arc::new(AccountValidationService::new(coa_client, chart_code)))
        }
        Err(e) => {
            tracing::warn!("Failed to connect to COA service: {}. Account validation will be skipped.", e);
            None
        }
    };

    // Application Handlers
    let mut create_handler = CreateJournalEntryHandler::new(journal_repo.clone());
    if let Some(validator) = account_validation {
        create_handler = create_handler.with_account_validation(validator);
    }
    let create_handler = Arc::new(create_handler);

    // Continue with service setup...
}
```

### 2. 环境变量配置

在 K8s 部署中添加以下环境变量：

```yaml
env:
  - name: COA_SERVICE_URL
    value: "http://coa-service.finance.svc.cluster.local:50060"
  - name: CHART_OF_ACCOUNTS
    value: "CN01"  # 中国会计准则
```

### 3. 验证流程

```
1. GL Service 接收创建凭证请求
   ↓
2. 提取所有科目代码
   ↓
3. 调用 COA Service 批量验证科目
   ↓
4. COA Service 检查：
   - 科目是否存在
   - 科目是否激活
   - 科目是否可过账
   - 科目有效期
   ↓
5. 如果验证失败，返回错误
   ↓
6. 如果验证成功，继续创建凭证
```

## 错误处理

### 优雅降级

如果 COA 服务不可用，GL Service 会：
1. 记录警告日志
2. 跳过科目验证
3. 继续正常处理凭证

这确保了系统的高可用性。

### 验证错误示例

```json
{
  "error": "科目验证失败: 科目 9999999999 不存在; 科目 1001000000 未激活"
}
```

## 性能优化

### 批量验证

使用批量验证 API 减少网络往返：

```rust
// 不推荐：逐个验证
for account_code in account_codes {
    validate_account(account_code).await?;
}

// 推荐：批量验证
batch_validate_accounts(account_codes).await?;
```

### 连接池

COA 客户端使用 tonic 的连接池，自动管理连接复用。

## 测试

### 单元测试

```rust
#[tokio::test]
async fn test_validate_account() {
    let mut client = CoaClient::connect("http://localhost:50060").await.unwrap();

    let result = client.validate_account(
        "CN01",
        "1001000000",
        Some("1000"),
        chrono::Utc::now().naive_utc().date(),
    ).await.unwrap();

    assert!(result.is_valid);
}
```

### 集成测试

```bash
# 启动 COA 服务
cargo run -p coa-service

# 启动 GL 服务
export COA_SERVICE_URL=http://localhost:50060
cargo run -p gl-service

# 测试创建凭证
grpcurl -plaintext -d '{
  "header": {
    "company_code": "1000",
    "fiscal_year": 2026,
    "posting_date": "2026-01-16T00:00:00Z",
    "document_date": "2026-01-16T00:00:00Z",
    "currency": "CNY"
  },
  "line_items": [
    {
      "gl_account": "1001000000",
      "debit_credit_indicator": "D",
      "amount": {"value": "1000", "currency_code": "CNY"}
    },
    {
      "gl_account": "2202000000",
      "debit_credit_indicator": "C",
      "amount": {"value": "1000", "currency_code": "CNY"}
    }
  ]
}' localhost:50052 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

## 监控

### 日志

```
INFO  Connected to COA service at http://coa-service:50060
INFO  Validating 2 accounts via COA service
INFO  All accounts validated successfully
```

### 指标

建议监控以下指标：
- COA 服务连接成功率
- 科目验证请求数
- 科目验证失败率
- 验证响应时间

## 故障排查

### COA 服务连接失败

```
WARN  Failed to connect to COA service: Connection refused. Account validation will be skipped.
```

**解决方案：**
1. 检查 COA 服务是否运行
2. 检查网络连接
3. 验证 COA_SERVICE_URL 配置

### 科目验证失败

```
ERROR Account validation failed: 科目 1001000000 不存在
```

**解决方案：**
1. 检查科目代码是否正确
2. 确认科目已在 COA 服务中创建
3. 验证科目表代码（CHART_OF_ACCOUNTS）

## AP/AR 服务集成

AP 和 AR 服务的集成步骤与 GL Service 相同：

1. 复制 COA 客户端代码
2. 在发票过账前验证科目
3. 配置环境变量

示例代码结构相同，只需调整命名空间。

## 总结

COA 服务集成提供了：
- ✅ 集中式科目管理
- ✅ 实时科目验证
- ✅ 优雅降级机制
- ✅ 批量验证优化
- ✅ 完整的错误处理

这确保了财务数据的准确性和系统的可靠性。
