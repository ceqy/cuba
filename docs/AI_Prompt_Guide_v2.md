# 企业级微服务架构 v2.0：gRPC Proto 定义与 AI 使用指南

## 1. 架构升级概述

此 v2.0 版本是对初始微服务设计的重大升级，旨在满足大型企业项目的复杂性和合规性要求。核心改进包括：

- **深度参考核心ERP底表**: 所有数据模型都严格参考了主流ERP系统（如SAP S/4HANA）的核心数据表（例如，财务的 `BKPF`/`BSEG`，采购的 `EKKO`/`EKPO`），确保了字段级别的兼容性和数据完整性。
- **系统性关键字脱敏**: 为了降低合规风险和提高通用性，所有ERP专有术语（如 `SAP`, `BAPI`, `Tcode`）都已被替换为行业标准或通用业务术语（如 `Core ERP`, `API`, `Transaction`）。详细的映射关系请参见 `field_mapping_reference.md`。
- **全面的中文注释**: 所有 `.proto` 文件中的字段和RPC都添加了详细的中文注释，解释了其业务含义和对应的原始ERP字段，极大地提升了可读性和可维护性。
- **企业级字段完整性**: 相比初版，v2.0 的消息结构包含了更多在实际业务中至关重要的字段，能够支撑更复杂的业务场景。

## 2. Proto 文件结构 (v2.0)

提供的 `sap-microservices-proto-v2.zip` 文件包含了所有升级后的微服务 `.proto` 定义。

```
/sap-microservices-proto-v2
|-- common/                  # 通用消息定义 (v2)
|   `-- common.proto
|-- events/                  # 跨领域事件定义 (v2)
|   `-- events.proto
|-- finance/                 # 财务领域 (v2, 参考BKPF/BSEG等)
|-- procurement/             # 采购领域 (v2, 参考EKKO/EKPO等)
|-- manufacturing/           # 制造领域 (v2, 参考AUFK/AFRU等)
|-- supplychain/             # 供应链领域 (v2, 参考MKPF/MSEG等)
|-- asset/                   # 资产管理领域 (v2, 参考EQUI/ILOA等)
|-- sales/                   # 销售领域 (v2, 参考VBAK/VBAP等)
|-- service/
|-- rd/
|-- hr/
`-- docs/                    # 文档
    |-- field_mapping_reference.md  # 字段脱敏映射参考
    `-- AI_Prompt_Guide_v2.md       # 本指南
```

## 3. AI 使用指南 (v2.0 提示词)

作为AI，你可以利用这些高度结构化和详细的 v2.0 Proto 文件来执行更复杂和精确的任务。以下是针对新版设计优化的提示词模式：

### 提示词模式 1：生成符合企业规范的服务实现

**目标**：基于详细的Proto定义，生成一个符合大型项目编码规范（如包含日志、指标、配置管理）的服务端实现框架。

**提示词模板**：

> “你是一名资深软件架构师，熟悉使用 [语言，如 Go/Java] 构建企业级微服务。请基于以下 v2.0 gRPC proto 定义，为 `enterprise.finance.gl.GeneralLedgerJournalEntryService` 生成一个完整的服务端实现框架。框架应包括：
> 1.  **清晰的目录结构**：区分 `internal` 和 `pkg` 目录。
> 2.  **依赖注入**：通过构造函数注入依赖项（如数据库连接、日志记录器）。
> 3.  **配置管理**：从环境变量或配置文件加载配置。
> 4.  **结构化日志**：在每个RPC方法的入口和出口记录关键信息。
> 5.  **错误处理**：定义并使用统一的错误处理机制。
> 6.  **方法存根**：为 `CreateJournalEntry` 方法提供一个详细的实现骨架，包括输入验证、调用下游依赖的伪代码，以及构建响应的逻辑。
>
> ```proto
> // --- common.proto ---
> [在此处粘贴 common.proto 的内容]
>
> // --- gl_journal_entry_service.proto ---
> [在此处粘贴 gl_journal_entry_service.proto 的内容]
> ```”

**使用技巧**：
- **提供多文件上下文**：由于 v2.0 的服务都依赖 `common.proto`，一次性提供所有相关的 proto 文件内容至关重要。
- **强调企业级实践**：明确要求代码中体现日志、配置、错误处理等企业级开发实践。

### 提示词模式 2：进行数据模型转换和验证

**目标**：编写一个函数，将来自外部系统（如第三方发票识别服务）的JSON数据，转换为 `ReceiveInvoiceRequest` proto 消息，并进行业务规则验证。

**提示词模板**：

> “作为一名 [语言，如 Python] 开发专家，请编写一个函数，该函数接收一个代表供应商发票的JSON对象，并将其转换为 `enterprise.procurement.invoice.ReceiveInvoiceRequest` gRPC 请求消息。函数需要：
> 1.  **数据映射**：将JSON字段精确映射到Proto消息的字段，注意 `MonetaryValue` 和 `Timestamp` 等复杂类型的转换。
> 2.  **业务验证**：在转换后，执行以下业务规则验证：
>     - `gross_amount` 必须大于0。
>     - `document_date` 不能晚于今天。
>     - 每个 `InvoiceItem` 的 `purchase_order_number` 必须存在。
> 3.  **返回错误**：如果验证失败，返回一个包含清晰错误信息的列表。
>
> **输入JSON示例**：
> ```json
> {
>   "supplierInvoiceId": "INV-12345",
>   "companyCode": "1000",
>   "date": "2026-01-10",
>   "totalAmount": 1190.00,
>   "tax": 190.00,
>   "currency": "USD",
>   "items": [
>     { "poNumber": "4500000001", "poItem": 10, "quantity": 100, "amount": 1000.00 }
>   ]
> }
> ```
>
> **Proto 定义**：
> ```proto
> // --- common.proto ---
> [在此处粘贴 common.proto 的内容]
>
> // --- invoice_processing_service.proto ---
> [在此处粘贴 invoice_processing_service.proto 的内容]
> ```”

### 提示词模式 3：基于事件模型设计解耦的业务流程

**目标**：设计一个跨领域的业务流程，该流程由事件触发，并与其他服务异步交互。

**提示词模板**：

> “我需要设计一个“销售订单创建后自动触发信控检查”的业务流程。作为解决方案架构师，请基于 `events.proto` v2.1 的定义，描述此流程的实现方案：
> 1.  **触发机制**：流程如何由 `SalesOrderCreatedEvent` 事件启动？描述消息队列（如Kafka）的Topic和消费者组设置。
> 2.  **事件消费**：信控服务（一个新的微服务）的消费者如何解析 `CloudEvent` 并获取 `SalesOrderCreatedEvent` 的数据？
> 3.  **业务逻辑**：信控服务在收到事件后，需要执行哪些步骤？（例如：调用 `ar_ap_service` 的 `GetCustomerDetails` 和 `ListCustomerOpenItems` 来评估客户信用）。
> 4.  **结果处理**：
>     - 如果信用检查通过，应该做什么？
>     - 如果信用检查失败，应该如何通知销售部门？（例如，是调用 `sales_order_fulfillment_service` 的API去冻结订单，还是发出一个新的 `CreditCheckFailedEvent` 事件？）
> 5.  **可靠性**：如何处理事件消费失败或信控逻辑执行超时的情况？（例如，死信队列、重试机制）。
>
> **Proto 定义**：
> ```proto
> // --- events.proto ---
> [在此处粘贴 events.proto v2.1 的内容]
> ```”

### 提示词模式 4：利用脱敏映射文档进行系统集成分析

**目标**：分析将一个外部系统（如 Salesforce）与我们的微服务架构集成的可行性和工作量。

**提示词模板**：

> “你是一名集成架构师。我们计划将 Salesforce 与我们的企业微服务架构进行双向同步。请基于 `field_mapping_reference.md` 和相关的Proto文件，完成以下分析：
> 1.  **数据实体映射**：将 Salesforce 的核心对象 (Account, Opportunity, Order) 与我们架构中的实体 (Customer, SalesOrder) 进行映射。创建一个Markdown表格来展示此映射关系。
> 2.  **字段级映射**：对于 Salesforce 的 `Order` 对象，将其关键字段与 `enterprise.sales.fulfillment.SalesOrderHeader` 和 `SalesOrderItem` 的字段进行详细映射。请参考 `field_mapping_reference.md` 中 `VBAK`/`VBAP` 的原始字段来确保准确性。
> 3.  **集成点识别**：
>     - 当 Salesforce 中创建或更新一个 `Order` 时，应该调用我们哪个微服务的哪个RPC？
>     - 当我们的 `SalesOrderFulfillmentService` 创建一个销售订单后，应该如何通知 Salesforce？（是通过API回调还是通过事件驱动的中间件？）
> 4.  **潜在挑战**：识别在数据转换、主数据一致性（如客户主数据）、事务完整性等方面可能遇到的主要挑战。
>
> **参考资料**：
> ```
> [在此处粘贴 field_mapping_reference.md 的相关部分，特别是销售领域和脱敏关键字部分]
> 
> [在此处粘贴 sales_order_fulfillment_service.proto 的内容]
> ```”

通过这些更具体、更深入的提示词，你可以充分利用 v2.0 设计的丰富细节，生成更高质量、更贴近实际项目需求的代码和文档。
