# Universal Journal (ACDOCA) Service

## 概述 (Overview)

Universal Journal Service 是一个完整的 SAP S/4HANA Universal Journal (ACDOCA) 统一视图服务实现，提供跨模块（GL/AP/AR/AA/MM/SD/CO/TR）的财务数据查询和分析功能。

## 功能特性 (Features)

### 核心功能
- ✅ **统一查询接口** - 支持分页查询、过滤、排序
- ✅ **流式查询接口** - 支持大数据量场景的流式查询
- ✅ **单条记录查询** - 通过主键快速查询单条记录
- ✅ **聚合查询接口** - 支持多维度聚合分析

### ACDOCA 完整字段映射
- ✅ 主键字段（分类账、公司代码、会计年度、凭证号、行号）
- ✅ 凭证抬头字段（凭证类型、日期、货币、汇率等）
- ✅ 行项目字段（过账码、借贷标识、科目、业务伙伴等）
- ✅ 金额字段（凭证货币、本位币、集团货币、全球货币等）
- ✅ 成本对象字段（成本中心、利润中心、段、功能范围等）
- ✅ 税务字段（税码、税收辖区、税额）
- ✅ 清账字段（清账凭证、清账日期）
- ✅ 付款字段（基准日期、到期日、付款条件、付款方式等）
- ✅ 特殊总账字段（特殊总账标识）
- ✅ 发票参考字段（参考凭证号、参考年度等）
- ✅ 业务交易类型字段（交易类型、参考交易类型等）
- ✅ 组织维度字段（财务范围、合并单位、伙伴公司等）
- ✅ 多币种字段（本位币、集团货币、全球货币等）
- ✅ 催款字段（催款码、催款冻结、催款日期等）
- ✅ 付款条件详细字段（折扣天数、折扣百分比等）
- ✅ 内部交易字段（发送成本中心、伙伴利润中心等）
- ✅ 科目分配字段（科目分配）
- ✅ 本地 GAAP 字段（本地科目、数据来源）
- ✅ 字段拆分字段（拆分方法、手工拆分标识）
- ✅ 审计字段（创建人、创建时间、修改人、修改时间）
- ✅ 来源模块标识（GL/AP/AR/AA/MM/SD/CO/TR）

### 查询过滤功能
- 分类账过滤
- 公司代码过滤
- 会计年度范围过滤
- 凭证类型过滤
- 过账日期范围过滤
- 凭证日期范围过滤
- 总账科目过滤
- 账户类型过滤
- 业务伙伴过滤
- 成本对象过滤（成本中心、利润中心、段、业务范围）
- 来源模块过滤
- 清账状态过滤（未清项/已清项）
- 特殊总账过滤
- 全文搜索（凭证文本、行项目文本）

## 技术架构 (Architecture)

### 技术栈
- **语言**: Rust
- **框架**: Tonic (gRPC)
- **数据库**: PostgreSQL
- **ORM**: SQLx
- **异步运行时**: Tokio

### 项目结构
```
uj-service/
├── src/
│   ├── api/                    # gRPC 服务接口层
│   │   └── grpc_server.rs      # gRPC 服务实现
│   ├── domain/                 # 领域层
│   │   ├── aggregates/         # 聚合根
│   │   │   └── universal_journal_entry.rs  # Universal Journal 实体
│   │   └── repositories.rs     # 仓储接口定义
│   ├── infrastructure/         # 基础设施层
│   │   ├── grpc/               # gRPC 相关
│   │   │   └── proto.rs        # Proto 生成代码
│   │   └── persistence/        # 数据持久化
│   │       └── postgres_uj_repository.rs  # PostgreSQL 仓储实现
│   ├── lib.rs                  # 库入口
│   └── main.rs                 # 服务入口
├── migrations/                 # 数据库迁移脚本
│   └── 20260119000000_create_universal_journal.sql
├── build.rs                    # 构建脚本
├── Cargo.toml                  # 依赖配置
└── README.md                   # 本文档
```

## 数据库设计 (Database Design)

### 主表: universal_journal_entries

主键字段：
- `ledger` - 分类账（0L-主账，1L/2L-非主账）
- `company_code` - 公司代码
- `fiscal_year` - 会计年度
- `document_number` - 凭证号
- `document_line` - 凭证行号

### 索引优化
- 过账日期索引（最常用查询）
- 凭证日期索引
- 公司代码 + 会计年度索引
- 总账科目索引
- 业务伙伴索引
- 成本中心索引
- 利润中心索引
- 段索引
- 业务范围索引
- 清账状态索引（未清项查询）
- 来源模块索引
- 凭证类型索引
- 特殊总账标识索引
- 复合索引（公司代码 + 会计年度 + 过账日期）
- 复合索引（总账科目 + 过账日期）
- 全文搜索索引（GIN 索引）

## API 接口 (API Endpoints)

### 1. 查询 Universal Journal（分页）
```protobuf
rpc QueryUniversalJournal(QueryUniversalJournalRequest) returns (QueryUniversalJournalResponse)
```

**HTTP 端点**: `POST /api/v1/finance/universal-journal:query`

**请求示例**:
```json
{
  "filter": {
    "company_codes": ["1000"],
    "fiscal_year_from": 2026,
    "fiscal_year_to": 2026,
    "posting_date_from": "2026-01-01T00:00:00Z",
    "posting_date_to": "2026-12-31T23:59:59Z",
    "only_open_items": true
  },
  "pagination": {
    "page": 1,
    "page_size": 50
  },
  "order_by": ["posting_date DESC", "document_number"]
}
```

### 2. 流式查询 Universal Journal
```protobuf
rpc StreamUniversalJournal(QueryUniversalJournalRequest) returns (stream UniversalJournalEntry)
```

**HTTP 端点**: `POST /api/v1/finance/universal-journal:stream`

用于大数据量查询场景，返回流式响应。

### 3. 获取单条记录
```protobuf
rpc GetUniversalJournalEntry(GetUniversalJournalEntryRequest) returns (UniversalJournalEntry)
```

**HTTP 端点**: `GET /api/v1/finance/universal-journal/entries/{ledger}/{company_code}/{fiscal_year}/{document_number}/{document_line}`

### 4. 聚合查询
```protobuf
rpc AggregateUniversalJournal(AggregateUniversalJournalRequest) returns (AggregateUniversalJournalResponse)
```

**HTTP 端点**: `POST /api/v1/finance/universal-journal:aggregate`

**请求示例**:
```json
{
  "filter": {
    "company_codes": ["1000"],
    "fiscal_year_from": 2026,
    "fiscal_year_to": 2026
  },
  "dimensions": ["AGGREGATION_DIMENSION_GL_ACCOUNT", "AGGREGATION_DIMENSION_COST_CENTER"],
  "measure": "AGGREGATION_MEASURE_SUM",
  "measure_field": "amount_in_local_currency"
}
```

## 部署指南 (Deployment)

### 环境变量
```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/erp
```

### 运行数据库迁移
```bash
cd apps/fi/uj-service
sqlx migrate run
```

### 启动服务
```bash
cargo run --package uj-service
```

服务将在 `0.0.0.0:50055` 端口启动。

### Docker 部署
```bash
# 构建镜像
docker build -t uj-service:latest .

# 运行容器
docker run -d \
  -p 50055:50055 \
  -e DATABASE_URL=postgres://postgres:postgres@db:5432/erp \
  uj-service:latest
```

## 开发指南 (Development)

### 编译项目
```bash
cargo build --package uj-service
```

### 运行测试
```bash
cargo test --package uj-service
```

### 代码检查
```bash
cargo check --package uj-service
cargo clippy --package uj-service
```

## 性能优化 (Performance)

### 数据库优化
1. **索引优化** - 为常用查询字段创建索引
2. **分区表** - 按会计年度分区（未来实现）
3. **物化视图** - 为常用聚合查询创建物化视图（未来实现）

### 查询优化
1. **分页查询** - 使用 LIMIT/OFFSET 进行分页
2. **流式查询** - 大数据量场景使用流式查询
3. **字段选择** - 支持指定返回字段（未来实现）

### 缓存策略
1. **查询结果缓存** - 使用 Redis 缓存常用查询结果（未来实现）
2. **聚合结果缓存** - 缓存聚合查询结果（未来实现）

## 数据同步 (Data Synchronization)

Universal Journal 数据来源于各个财务模块：

- **GL Service** - 总账凭证
- **AP Service** - 应付账款凭证
- **AR Service** - 应收账款凭证
- **AA Service** - 固定资产凭证
- **MM Service** - 物料管理凭证
- **SD Service** - 销售与分销凭证
- **CO Service** - 成本控制凭证
- **TR Service** - 资金管理凭证

### 同步机制（未来实现）
1. **事件驱动同步** - 各模块过账后发送事件到 Universal Journal
2. **批量同步** - 定期批量同步数据
3. **增量同步** - 仅同步变更数据

## SAP 字段映射 (SAP Field Mapping)

| Rust 字段 | SAP 字段 | 描述 |
|----------|---------|------|
| ledger | RLDNR | 分类账 |
| company_code | RBUKRS | 公司代码 |
| fiscal_year | GJAHR | 会计年度 |
| document_number | BELNR | 凭证号 |
| document_line | DOCLN | 凭证行号 |
| document_type | BLART | 凭证类型 |
| document_date | BLDAT | 凭证日期 |
| posting_date | BUDAT | 过账日期 |
| fiscal_period | MONAT | 会计期间 |
| gl_account | RACCT | 总账科目 |
| amount_in_document_currency | WRBTR | 凭证货币金额 |
| amount_in_local_currency | DMBTR | 本位币金额 |
| cost_center | KOSTL | 成本中心 |
| profit_center | PRCTR | 利润中心 |
| ... | ... | ... |

完整字段映射请参考 `protos/fi/uj/uj.proto` 文件。

## 未来计划 (Future Plans)

### 短期计划
- [ ] 添加集成测试
- [ ] 添加性能测试
- [ ] 实现字段选择功能
- [ ] 添加 Dockerfile

### 中期计划
- [ ] 实现数据同步机制
- [ ] 添加 Redis 缓存
- [ ] 实现物化视图
- [ ] 添加监控和告警

### 长期计划
- [ ] 实现表分区
- [ ] 添加数据归档功能
- [ ] 实现实时数据流
- [ ] 添加 GraphQL 接口

## 贡献指南 (Contributing)

欢迎贡献代码！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证 (License)

本项目采用 MIT 许可证。

## 联系方式 (Contact)

如有问题或建议，请通过以下方式联系：

- 提交 Issue
- 发送邮件至项目维护者

## 致谢 (Acknowledgments)

- SAP S/4HANA Universal Journal 设计
- Rust 社区
- Tonic gRPC 框架
- SQLx 数据库库
