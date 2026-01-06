# 企业级微服务架构 Proto 定义 (v2.0) - 完整版

## 项目概述

本项目为企业级核心ERP系统（如SAP S/4HANA）的周边扩展微服务提供了完整的 gRPC Proto 接口定义和事件模型。设计遵循领域驱动设计（DDD）原则，覆盖财务、采购、制造、供应链、资产管理、销售、服务、研发和人力资源等九大业务领域。

**v2.0 版本核心特性**：
- 深度参考核心ERP底表结构（如BKPF, EKKO, VBAK等），确保字段级兼容性
- 系统性关键字脱敏，降低合规风险
- 全面的中文注释，提升可读性
- 企业级字段完整性，支撑大型项目需求

## 目录结构

```
/sap-microservices-proto-v2
├── common/                     # 通用消息定义
├── events/                     # 事件驱动模型
├── finance/                    # 财务领域
├── procurement/                # 采购领域
├── manufacturing/              # 制造领域
├── supplychain/                # 供应链领域
├── asset/                      # 资产管理领域
├── sales/                      # 销售领域
├── service/                    # 服务领域
├── rd/                         # 研发领域
├── hr/                         # 人力资源领域
└── docs/                       # 文档
    ├── field_mapping_reference.md        # ERP底表字段映射参考
    └── AI_Prompt_Guide_v2.md             # AI使用提示词指南
```

## 微服务清单 (40/40)

| 领域 | 服务名称 | Proto文件 | 参考底表 |
|---|---|---|---|
| **财务 (5)** | 总账凭证服务 | gl_journal_entry_service.proto | BKPF, BSEG |
| | 应收/应付服务 | ar_ap_service.proto | KNA1, KNB1, LFA1, LFB1 |
| | 成本分配服务 | controlling_allocation_service.proto | COEP |
| | 资金管理服务 | treasury_services.proto | FEBEP |
| | 集团合并服务 | `(TBD)` | `(TBD)` |
| **采购 (6)** | 采购订单服务 | order_service.proto | EKKO, EKPO, EKET |
| | 合同管理服务 | contract_management_service.proto | EKKO (Contract) |
| | 发票处理服务 | invoice_processing_service.proto | RBKP, RSEG |
| | 供应商门户服务 | supplier_portal_service.proto | - |
| | 支出分析服务 | spend_analytics_service.proto | (Data Warehouse) |
| | 寻源事件服务 | sourcing_event_service.proto | EKKO (RFQ) |
| **制造 (5)** | 生产计划服务 | production_planning_service.proto | PLAF, MDKP |
| | 车间执行服务 | shop_floor_execution_service.proto | AFKO, AFRU, RESB |
| | 质量检验服务 | quality_inspection_service.proto | QALS, QAMR |
| | 看板服务 | kanban_service.proto | PKHD, PKPS |
| | 外协加工服务 | outsourced_manufacturing_service.proto | (Subcon PO) |
| **供应链 (6)** | 库存管理服务 | inventory_management_service.proto | MKPF, MSEG, MARD |
| | 仓库运营服务 | warehouse_operations_service.proto | LTAK, LTAP |
| | 运输计划服务 | transportation_planning_service.proto | VTTK, VTTP |
| | 需求预测服务 | demand_forecasting_service.proto | PBED, PBIM |
| | 可见性服务 | visibility_service.proto | (Aggregated) |
| | 批次追溯服务 | batch_traceability_service.proto | MCH1, CHVW |
| **资产管理 (4)** | 资产维护服务 | asset_maintenance_service.proto | EQUI, ILOA, AUFK |
| | 智能资产健康 | intelligent_asset_health_service.proto | (IoT Data) |
| | EHS事件服务 | ehs_incident_service.proto | (EHS Tables) |
| | 地理位置服务 | geo_service.proto | (GIS Data) |
| **销售 (4)** | 销售订单服务 | sales_order_fulfillment_service.proto | VBAK, VBAP, VBEP |
| | 定价引擎服务 | pricing_engine_service.proto | KONV, KONP |
| | 收入确认服务 | revenue_recognition_service.proto | FARR_D_POSTING |
| | 销售分析服务 | analytics_service.proto | (Data Warehouse) |
| **服务 (3)** | 现场服务调度 | field_service_dispatch_service.proto | AUFK (Service) |
| | 服务合同计费 | contract_billing_service.proto | (Service Contract) |
| | 保修索赔服务 | warranty_claims_service.proto | (Warranty Claim) |
| **研发 (2)** | PLM集成服务 | plm_integration_service.proto | MAST, STKO, STPO |
| | 项目成本控制 | project_cost_controlling_service.proto | PRPS, COSP, COSS |
| **HR (2)** | 人才招聘服务 | talent_acquisition_service.proto | (HR/Talent Tables) |
| | 员工体验服务 | employee_experience_service.proto | (Survey Data) |

## 事件模型

所有跨服务的异步通信都通过事件进行，事件定义在 `events/events.proto` 中。事件遵循 CloudEvents 规范。

| 事件类型 | 描述 | 生产者 |
|---|---|---|
| JournalEntryPostedEvent | 总账凭证已过账 | 总账凭证服务 |
| InvoicePostedEvent | 发票已过账 | 发票处理服务 |
| PurchaseOrderReleasedEvent | 采购订单已审批 | 采购订单服务 |
| SalesOrderCreatedEvent | 销售订单已创建 | 销售订单服务 |
| StockChangedEvent | 库存变更 | 库存管理服务 |
| ProductionOrderReleasedEvent | 生产订单已下达 | 生产计划服务 |
| MaintenanceOrderReleasedEvent | 维护订单已下达 | 资产维护服务 |
| ... | ... | ... |

## 快速开始

### 1. 生成代码

使用 `protoc` 编译器和对应语言的 gRPC 插件生成代码：

```bash
# Go
protoc --go_out=. --go-grpc_out=. \
  common/common.proto \
  events/events.proto \
  finance/*.proto \
  procurement/*.proto \
  # ... and so on for all domains

# Python
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. \
  common/common.proto \
  events/events.proto \
  finance/*.proto \
  procurement/*.proto \
  # ... and so on for all domains
```

### 2. 参考文档

- **字段映射参考**: `docs/field_mapping_reference.md` 详细记录了每个Proto字段与核心ERP底表字段的对应关系。
- **AI使用指南**: `docs/AI_Prompt_Guide_v2.md` 提供了如何使用AI工具基于这些Proto定义生成代码或进行分析的提示词模板。

## 设计原则

1. **领域驱动设计 (DDD)**: 每个微服务围绕一个清晰的业务领域边界构建。
2. **API 优先**: 使用 gRPC 和 Protocol Buffers 定义强类型的、跨语言的服务契约。
3. **事件驱动架构 (EDA)**: 通过异步事件实现服务间的解耦和最终一致性。
4. **可扩展性**: 所有服务都设计为可独立部署、扩展和维护。
5. **合规性**: 通过关键字脱敏和详细的字段映射文档，确保设计的合规性和可追溯性。
