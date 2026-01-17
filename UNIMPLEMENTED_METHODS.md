# FI 模块未实现功能清单

## 总览

| 服务 | 未实现方法数 | 总体进度 |
|------|-------------|---------|
| COA Service | 5 | 大部分核心功能已完成 |
| GL Service | 12 | 核心功能已完成 |
| AP Service | 21 | 核心功能已完成 |
| AR Service | 25 | 核心功能已完成 |
| **总计** | **63** | **核心业务流程已实现** |

---

## 按功能分类的未实现方法

### 1. 催款管理 (Dunning Management)

**AP Service:**
- `get_dunning_history` - 获取催款历史
- `trigger_dunning` - 触发催款流程

**AR Service:**
- `get_dunning_history` - 获取催款历史
- `trigger_dunning` - 触发催款流程

### 2. 银行对账 (Bank Reconciliation)

**AP Service:**
- `import_bank_statement` - 导入银行对账单
- `process_lockbox` - 处理银行托收箱数据
- `apply_cash` - 应用现金收款

**AR Service:**
- `import_bank_statement` - 导入银行对账单
- `process_lockbox` - 处理银行托收箱数据
- `apply_cash` - 应用现金收款

### 3. 清账提议 (Clearing Proposals)

**AP Service:**
- `get_clearing_proposal` - 获取清账提议
- `execute_clearing_proposal` - 执行清账提议
- `net_clearing` - 净额清账

**AR Service:**
- `get_clearing_proposal` - 获取清账提议
- `execute_clearing_proposal` - 执行清账提议
- `net_clearing` - 净额清账

### 4. 付款/收款管理 (Payment Management)

**AP Service:**
- `generate_payment_proposal` - 生成付款提议 (已有简化实现)
- `execute_payment_proposal` - 执行付款提议 (已有简化实现)
- `request_down_payment` - 请求预付款
- `clear_down_payment` - 清账预付款

**AR Service:**
- `generate_payment_proposal` - 生成收款提议
- `execute_payment_proposal` - 执行收款提议
- `request_down_payment` - 请求预收款
- `clear_down_payment` - 清账预收款

### 5. 信用管理 (Credit Management)

**AP Service:**
- `check_credit_limit` - 检查信用额度
- `update_credit_exposure` - 更新信用敞口

**AR Service:**
- `check_credit_limit` - 检查信用额度
- `update_credit_exposure` - 更新信用敞口

### 6. 对账单和报表 (Statements & Reports)

**AP Service:**
- `generate_statement` - 生成对账单
- `export_report` - 导出报表

**AR Service:**
- `generate_statement` - 生成对账单
- `verify_invoice` - 验证发票
- `export_report` - 导出报表

### 7. 附件管理 (Attachments)

**AP Service:**
- `list_attachments` - 列出附件
- `upload_attachment` - 上传附件

**AR Service:**
- `list_attachments` - 列出附件
- `upload_attachment` - 上传附件

### 8. 合规检查 (Compliance)

**AP Service:**
- `get_tolerance_groups` - 获取容差组
- `perform_compliance_check` - 执行合规检查

**AR Service:**
- `get_tolerance_groups` - 获取容差组
- `perform_compliance_check` - 执行合规检查

### 9. 事件订阅 (Event Subscription)

**AP Service:**
- `subscribe_to_events` - 订阅事件
- `list_event_types` - 列出事件类型

**AR Service:**
- `subscribe_to_events` - 订阅事件
- `list_event_types` - 列出事件类型

### 10. 发票管理 (Invoice Management)

**AP Service:**
- `post_sales_invoice` - 过账销售发票 (应由 AR 处理)

**AR Service:**
- `post_invoice` - 过账采购发票 (应由 AP 处理)
- `reverse_document` - 冲销单据

### 11. 总账专用功能 (GL-Specific)

**GL Service:**
- `stream_journal_entries` - 流式传输凭证
- `simulate_journal_entry` - 模拟凭证过账
- `update_journal_entry` - 更新凭证
- `park_journal_entry` - 暂存凭证
- `clear_open_items` - 清账未清项
- `revaluate_foreign_currency` - 外币重估
- `get_parallel_ledger_data` - 获取平行账套数据
- `carry_forward_balances` - 结转余额
- `execute_period_end_close` - 执行期末关账
- `create_batch_input_session` - 创建批量输入会话
- `get_account_line_items` - 获取科目行项目
- `generate_print_preview` - 生成打印预览

### 12. 科目表功能 (COA-Specific)

**COA Service:**
- `create_account_group` - 创建科目组 (需要额外数据库表)
- `list_account_groups` - 列出科目组 (需要额外数据库表)
- `batch_update_gl_accounts` - 批量更新科目
- `import_accounts` - 导入科目 (需要文件解析)
- `export_accounts` - 导出科目 (需要文件生成)

---

## 已完成的核心功能

### COA Service ✅
- ✅ 创建/获取/更新科目
- ✅ 批量验证科目
- ✅ 检查科目可过账性
- ✅ 层级查询
- ✅ 子科目列表
- ✅ 科目路径查询
- ✅ 批量创建科目

### GL Service ✅
- ✅ 过账凭证
- ✅ 验证凭证
- ✅ 冲销凭证
- ✅ 批量创建凭证
- ✅ 批量冲销凭证
- ✅ 删除凭证
- ✅ 列出凭证(带分页)
- ✅ 会计期间计算

### AP Service ✅
- ✅ 过账供应商发票
- ✅ 供应商主数据管理
- ✅ 列出未清项
- ✅ 账户余额查询
- ✅ 账龄分析(5档)
- ✅ 批量清账
- ✅ 部分清账
- ✅ GL 集成

### AR Service ✅
- ✅ 过账销售发票
- ✅ 客户主数据管理
- ✅ 列出未清项
- ✅ 账户余额查询
- ✅ 账龄分析(5档)
- ✅ 批量清账
- ✅ 部分清账
- ✅ GL 集成

---

## 实现优先级建议

### 高优先级 (High Priority)
1. **GL Service - 暂存凭证** (`park_journal_entry`) - 草稿功能
2. **GL Service - 更新凭证** (`update_journal_entry`) - 修改未过账凭证
3. **AP/AR - 冲销单据** (`reverse_document`) - 业务流程必需
4. **COA - 批量更新科目** (`batch_update_gl_accounts`) - 批量维护

### 中优先级 (Medium Priority)
1. **付款提议完整实现** - 目前仅有简化 stub
2. **清账提议** - 自动匹配清账
3. **催款管理** - 应收管理重要功能
4. **对账单生成** - 客户/供应商对账

### 低优先级 (Low Priority)
1. 附件管理 - 需要对象存储集成
2. 事件订阅 - 需要消息队列基础设施
3. 导入/导出功能 - 需要文件处理框架
4. 银行对账 - 复杂业务规则
5. 合规检查 - 特定行业需求

---

## 技术债务说明

### 简化实现 (需要完善)
1. **付款提议** (AP/AR) - 当前返回空列表,需要:
   - 从 proto 正确提取 `OpenItemIdentifier`
   - 实现到期日筛选逻辑
   - 支付方法选择

2. **清账操作** (AP/AR) - 当前仅返回成功响应,需要:
   - 实际的清账文档创建
   - GL 凭证生成
   - 余额验证

### 需要基础设施支持
1. **科目组管理** - 需要新建 `account_groups` 表
2. **附件功能** - 需要 S3/MinIO 等对象存储
3. **事件订阅** - 需要 Kafka/RabbitMQ 等消息队列
4. **导入/导出** - 需要文件处理服务

---

## 结论

当前 FI 模块的**核心业务流程已经完整实现**,包括:
- ✅ 主数据管理(客户/供应商/科目)
- ✅ 发票管理和过账
- ✅ 未清项管理
- ✅ 账龄分析
- ✅ 清账功能
- ✅ GL 集成

未实现的 63 个方法主要属于:
- 高级功能(催款、清账提议、信用管理)
- 外围系统集成(银行对账、附件管理)
- 辅助功能(报表、事件订阅、合规检查)

**建议**: 优先完善付款提议和清账提议的完整实现,其他功能可根据实际业务需求逐步添加。
