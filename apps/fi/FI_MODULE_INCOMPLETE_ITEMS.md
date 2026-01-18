# FI 模块未完成工作清单

## 📋 概述

本文档列出了 FI（Finance - 财务会计）模块中已经开始但尚未完全实现的功能和待办事项。

---

## 🔴 高优先级 - 核心功能未完成

### 1. GL Service - 付款字段数据库映射 ⚠️

**状态**: 领域模型已完成，数据库层未完成

**已完成**:
- ✅ Proto 定义（PaymentExecutionDetail、PaymentTermsDetail）
- ✅ Rust 领域模型完整实现
- ✅ 单元测试（24个测试用例全部通过）
- ✅ 构建器模式
- ✅ 验证逻辑

**未完成**:
- ❌ 数据库 Migration（payment_execution 和 payment_terms_detail 字段）
- ❌ Repository 层数据库持久化代码
- ❌ gRPC 双向映射（proto ↔ domain model）
- ❌ 集成测试

**影响**:
- 付款条件详细信息（单级/双级现金折扣）无法持久化到数据库
- 付款执行信息（付款方式、冻结、优先级）无法持久化到数据库
- AP/AR 服务无法完整使用这些功能

**文档位置**:
- `apps/fi/gl-service/PAYMENT_EXECUTION_IMPLEMENTATION.md`
- `apps/fi/gl-service/PAYMENT_TERMS_IMPLEMENTATION.md`

**后续工作**:
```sql
-- 需要创建 migration 文件
-- 20260119000000_add_payment_fields.sql

-- 添加到 journal_entry_lines 表
ALTER TABLE journal_entry_lines ADD COLUMN payment_execution JSONB;
ALTER TABLE journal_entry_lines ADD COLUMN payment_terms_detail JSONB;

-- 添加索引
CREATE INDEX idx_payment_method ON journal_entry_lines ((payment_execution->>'payment_method'));
CREATE INDEX idx_payment_block ON journal_entry_lines ((payment_execution->>'payment_block'));
CREATE INDEX idx_baseline_date ON journal_entry_lines ((payment_terms_detail->>'baseline_date'));
```

---

### 2. Universal Journal Service - 流式查询未实现 ⚠️

**状态**: 基础查询已实现，流式查询未实现

**已完成**:
- ✅ Proto 定义完整（uj.proto - 完整 ACDOCA 映射）
- ✅ 基础查询接口（QueryUniversalJournal）
- ✅ 单条记录查询（GetUniversalJournalEntry）
- ✅ 聚合查询（AggregateUniversalJournal）
- ✅ gRPC 服务框架

**未完成**:
- ❌ 流式查询实现（StreamUniversalJournal）
- ❌ 大数据量优化
- ❌ 跨模块数据整合（当前只查询 GL，未整合 AP/AR/AA/MM）

**代码位置**:
- `apps/fi/gl-service/src/api/universal_journal_grpc.rs:59`
```rust
async fn stream_universal_journal(...) -> Result<...> {
    // 流式返回大数据量
    Err(Status::unimplemented("流式查询待实现"))  // ← 未实现
}
```

**影响**:
- 大数据量查询性能问题（无法使用流式返回）
- 无法处理百万级数据导出场景
- 实时数据分析受限

**后续工作**:
```rust
// 需要实现流式查询
async fn stream_universal_journal(
    &self,
    request: Request<QueryUniversalJournalRequest>,
) -> Result<Response<Self::StreamUniversalJournalStream>, Status> {
    let req = request.into_inner();

    // 实现流式查询逻辑
    let stream = self.stream_handler.handle_stream(req).await?;

    Ok(Response::new(Box::pin(stream)))
}
```

---

### 3. GL Service - 数据库 Migration 未运行 ⚠️

**状态**: Migration 文件已创建，但未在数据库中运行

**已创建的 Migration 文件**:
1. ✅ `20260118000000_add_parallel_accounting_ledger.sql` - 并行会计
2. ✅ `20260118000001_add_special_gl_indicator.sql` - 特殊总账标识
3. ✅ `20260118000002_add_invoice_reference.sql` - 发票参考
4. ✅ `20260118000002_add_organizational_dimensions.sql` - 组织维度
5. ✅ `20260118000003_add_dunning_detail.sql` - 催款管理
6. ✅ `20260118000003_add_multi_currency_support.sql` - 多币种
7. ✅ `20260118000004_add_transaction_type.sql` - 业务交易类型

**问题**:
- ❌ 数据库 schema 与 Migration 文件不同步
- ❌ 新字段在数据库中不存在
- ❌ 统计视图和索引未创建
- ❌ 主数据表未创建

**影响**:
- 所有新功能无法实际使用
- 测试只能在内存中进行
- 无法进行集成测试和端到端测试

**执行步骤**:
```bash
cd apps/fi/gl-service
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/gl_db
sqlx migrate run

# 验证
psql $DATABASE_URL -c "\d journal_entry_lines"
psql $DATABASE_URL -c "SELECT * FROM transaction_type_master LIMIT 5;"
```

---

## 🟡 中优先级 - 功能增强未完成

### 4. AP Service - 拒绝原因字段缺失

**状态**: TODO 注释，功能未实现

**代码位置**:
- `apps/fi/ap-service/src/application/handlers.rs:375`
```rust
// TODO: Store rejection reason  // ← 需要实现
```

**影响**:
- 发票拒绝时无法存储拒绝原因
- 审计追踪不完整
- 无法生成拒绝原因报表

**后续工作**:
1. 添加 rejection_reason 字段到 domain model
2. 添加 rejection_reason 字段到数据库 schema
3. 更新 gRPC proto 定义
4. 实现存储逻辑

---

### 5. Universal Journal - 跨模块数据整合未完成

**状态**: 当前只查询 GL 模块数据

**已完成**:
- ✅ GL (General Ledger) 数据查询

**未完成**:
- ❌ AP (Accounts Payable) 数据整合
- ❌ AR (Accounts Receivable) 数据整合
- ❌ AA (Asset Accounting) 数据整合
- ❌ MM (Materials Management) 数据整合
- ❌ SD (Sales and Distribution) 数据整合
- ❌ CO (Controlling) 数据整合

**影响**:
- Universal Journal 不"Universal"（不统一）
- 无法提供完整的财务数据视图
- 跨模块分析受限

**后续工作**:
```rust
// 需要实现多数据源查询
pub async fn query_universal_journal(&self, filter: Filter) -> Result<Vec<Entry>> {
    let mut results = Vec::new();

    // 从 GL 查询
    results.extend(self.gl_repository.query(filter).await?);

    // 从 AP 查询
    results.extend(self.ap_repository.query(filter).await?);

    // 从 AR 查询
    results.extend(self.ar_repository.query(filter).await?);

    // ... 其他模块

    Ok(results)
}
```

---

### 6. gRPC 映射层不完整

**状态**: 部分映射已实现，双向映射不完整

**未完成**:
- ❌ PaymentExecutionDetail proto ↔ domain 双向映射
- ❌ PaymentTermsDetail proto ↔ domain 双向映射
- ❌ DunningDetail proto ↔ domain 双向映射
- ❌ InvoiceReference proto ↔ domain 双向映射

**影响**:
- 无法通过 gRPC API 创建带有付款详细信息的凭证
- 无法通过 gRPC API 查询付款详细信息
- API 功能不完整

**后续工作**:
```rust
// 需要在 grpc_server.rs 中实现
impl From<proto::PaymentExecutionDetail> for domain::PaymentExecutionDetail {
    fn from(proto: proto::PaymentExecutionDetail) -> Self {
        // 实现 proto -> domain 转换
    }
}

impl From<domain::PaymentExecutionDetail> for proto::PaymentExecutionDetail {
    fn from(domain: domain::PaymentExecutionDetail) -> Self {
        // 实现 domain -> proto 转换
    }
}
```

---

## 🟢 低优先级 - 优化和增强

### 7. 自动付款程序未实现

**状态**: 字段支持已完成，业务逻辑未实现

**后续工作**:
- 实现自动付款建议生成
- 实现付款批次处理
- 实现付款执行确认
- 实现付款冻结管理

---

### 8. 催款自动化流程未实现

**状态**: 字段支持已完成，自动化流程未实现

**后续工作**:
- 实现自动催款级别升级
- 实现催款通知发送
- 实现催款费用计算
- 实现逾期分析报表

---

### 9. 集成测试覆盖不足

**状态**: 单元测试完整，集成测试缺失

**已完成**:
- ✅ 领域模型单元测试（100+ 测试用例）
- ✅ 特殊总账标识集成测试

**未完成**:
- ❌ 付款字段端到端测试
- ❌ Universal Journal 集成测试
- ❌ 跨模块集成测试
- ❌ 性能测试

---

### 10. 文档和示例不完整

**已完成**:
- ✅ 功能总结文档（5个）
- ✅ 测试指南（1个）
- ✅ SAP 字段映射文档

**未完成**:
- ❌ API 使用示例
- ❌ 端到端场景示例
- ❌ 部署指南
- ❌ 故障排查指南

---

## 📊 未完成工作统计

### 按优先级统计
- 🔴 高优先级: **3项**（核心功能影响）
- 🟡 中优先级: **3项**（功能完整性影响）
- 🟢 低优先级: **4项**（优化和增强）

### 按类型统计
- 数据库层: 2项
- 业务逻辑层: 3项
- API层: 2项
- 测试: 1项
- 文档: 1项
- 自动化流程: 2项

### 按模块统计
- GL Service: 5项
- AP Service: 1项
- AR Service: 0项
- Universal Journal: 2项
- 跨模块: 2项

---

## 🎯 推荐的实施顺序

### 第一阶段（核心功能完善）- 1-2周
1. **运行 GL Service Migration**（1天）
   - 验证所有 migration 文件
   - 运行 migration
   - 验证数据库 schema

2. **完成付款字段数据库映射**（3-5天）
   - 创建 payment fields migration
   - 实现 Repository 层持久化
   - 实现 gRPC 双向映射
   - 编写集成测试

3. **实现 Universal Journal 流式查询**（2-3天）
   - 实现流式查询逻辑
   - 测试大数据量性能
   - 优化查询性能

### 第二阶段（功能增强）- 2-3周
4. **实现跨模块数据整合**（5-7天）
   - 整合 AP 数据
   - 整合 AR 数据
   - 实现统一查询接口

5. **完善 gRPC 映射层**（3-5天）
   - 实现所有双向映射
   - 编写映射测试
   - 验证 API 完整性

6. **修复 AP Service 拒绝原因**（1天）
   - 添加字段
   - 实现存储逻辑
   - 更新测试

### 第三阶段（业务流程自动化）- 3-4周
7. **实现自动付款程序**（1-2周）
8. **实现催款自动化流程**（1-2周）

### 第四阶段（测试和文档）- 1-2周
9. **补充集成测试**（1周）
10. **完善文档和示例**（3-5天）

---

## 📝 关键依赖关系

```
运行 Migration (1)
    ↓
付款字段数据库映射 (2) → gRPC 映射层 (5) → 集成测试 (9)
    ↓
自动付款程序 (7)

Universal Journal 流式查询 (3) → 跨模块数据整合 (4) → 集成测试 (9)
    ↓
文档和示例 (10)

AP 拒绝原因 (6) → 文档和示例 (10)

催款自动化 (8) → 集成测试 (9)
```

---

## ⚠️ 风险提示

1. **数据库 Migration 风险**
   - 多个 migration 文件名重复（20260118000002、20260118000003）
   - 需要重命名避免冲突
   - 建议执行前备份数据库

2. **性能风险**
   - Universal Journal 流式查询未实现，大数据量查询可能超时
   - 跨模块查询可能性能较差，需要优化

3. **兼容性风险**
   - 新字段都是可选的，向后兼容
   - 但需要确保老数据能正常访问

---

## 🎉 总结

FI 模块已经完成了大量核心功能的实现，包括：
- ✅ 6个核心服务（GL/AP/AR/CO/COA/TR）
- ✅ 40+ Proto 字段定义
- ✅ 完整的领域模型
- ✅ 100+ 单元测试

**核心待办事项（3项高优先级）**:
1. 运行数据库 Migration
2. 完成付款字段数据库映射
3. 实现 Universal Journal 流式查询

完成这3项核心工作后，FI 模块即可投入生产使用。其他功能可以在后续迭代中逐步完善。

---

**文档版本**: v1.0
**更新日期**: 2026-01-19
**状态**: 待处理
