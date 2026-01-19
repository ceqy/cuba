# 企业级微服务架构 Proto 定义 (v3.0)

## 项目概述

本项目为企业级核心ERP系统的微服务提供 gRPC Proto 接口定义。设计遵循领域驱动设计（DDD）原则，覆盖 **10 大业务领域**、**41 个微服务**。

## 目录结构 (v3.0 - 域代码)

```
protos/
├── common/             # 通用消息
├── events/             # 事件模型
├── iam/                # 身份认证 (3服务)
│   ├── auth/           # 认证中心
│   ├── oauth/          # OAuth服务
│   └── rbac/           # 权限控制
├── fi/                 # 财务 (6服务)
│   ├── gl/             # 总账
│   ├── ap/             # 应收应付
│   ├── co/             # 成本控制
│   ├── tr/             # 资金管理
│   ├── coa/            # 会计科目表
│   └── uj/             # 统一日记账
├── pm/                 # 采购 (6服务)
│   ├── po/             # 采购订单
│   ├── ct/             # 合同管理
│   ├── iv/             # 发票处理
│   ├── sp/             # 供应商门户
│   ├── sa/             # 支出分析
│   └── se/             # 寻源事件
├── mf/                 # 制造 (5服务)
│   ├── pp/             # 生产计划
│   ├── sf/             # 车间执行
│   ├── qi/             # 质量检验
│   ├── kb/             # 看板
│   └── om/             # 外协加工
├── sc/                 # 供应链 (6服务)
│   ├── im/             # 库存管理
│   ├── wm/             # 仓库运营
│   ├── tp/             # 运输计划
│   ├── df/             # 需求预测
│   ├── vs/             # 可见性
│   └── bt/             # 批次追溯
├── am/                 # 资产管理 (4服务)
│   ├── pm/             # 资产维护
│   ├── ah/             # 智能健康
│   ├── eh/             # EHS事件
│   └── gs/             # 地理位置
├── sd/                 # 销售 (4服务)
│   ├── so/             # 销售订单
│   ├── pe/             # 定价引擎
│   ├── rr/             # 收入确认
│   └── an/             # 销售分析
├── cs/                 # 客户服务 (3服务)
│   ├── fd/             # 现场调度
│   ├── cb/             # 合同计费
│   └── wc/             # 保修索赔
├── rd/                 # 研发 (2服务)
│   ├── pl/             # PLM集成
│   └── ps/             # 项目成本
├── hr/                 # 人力资源 (2服务)
│   ├── ta/             # 人才招聘
│   └── ex/             # 员工体验
```

## 微服务清单 (41)

| 产品线 | 代码 | 服务 | Proto路径 |
|--------|------|------|-----------|
| **财务** | fi | 总账 | fi/gl/gl.proto |
| | | 应收应付 | fi/ap/ap.proto |
| | | 成本控制 | fi/co/co.proto |
| | | 会计科目表 | fi/coa/coa.proto |
| | | 资金管理 | fi/tr/tr.proto |
| | | 统一日记账 | fi/uj/uj.proto |
| **采购** | pm | 采购订单 | pm/po/po.proto |
| | | 合同管理 | pm/ct/ct.proto |
| | | 发票处理 | pm/iv/iv.proto |
| | | 供应商门户 | pm/sp/sp.proto |
| | | 支出分析 | pm/sa/sa.proto |
| | | 寻源事件 | pm/se/se.proto |
| **制造** | mf | 生产计划 | mf/pp/pp.proto |
| | | 车间执行 | mf/sf/sf.proto |
| | | 质量检验 | mf/qi/qi.proto |
| | | 看板 | mf/kb/kb.proto |
| | | 外协加工 | mf/om/om.proto |
| **供应链** | sc | 库存管理 | sc/im/im.proto |
| | | 仓库运营 | sc/wm/wm.proto |
| | | 运输计划 | sc/tp/tp.proto |
| | | 需求预测 | sc/df/df.proto |
| | | 可见性 | sc/vs/vs.proto |
| | | 批次追溯 | sc/bt/bt.proto |
| **资产** | am | 资产维护 | am/pm/pm.proto |
| | | 智能健康 | am/ah/ah.proto |
| | | EHS事件 | am/eh/eh.proto |
| | | 地理位置 | am/gs/gs.proto |
| **销售** | sd | 销售订单 | sd/so/so.proto |
| | | 定价引擎 | sd/pe/pe.proto |
| | | 收入确认 | sd/rr/rr.proto |
| | | 销售分析 | sd/an/an.proto |
| **服务** | cs | 现场调度 | cs/fd/fd.proto |
| | | 合同计费 | cs/cb/cb.proto |
| | | 保修索赔 | cs/wc/wc.proto |
| **研发** | rd | PLM集成 | rd/pl/pl.proto |
| | | 项目成本 | rd/ps/ps.proto |
| **HR** | hr | 人才招聘 | hr/ta/ta.proto |
| | | 员工体验 | hr/ex/ex.proto |
| **IAM** | iam | 认证中心 | iam/auth/auth.proto |
| | | OAuth服务 | iam/oauth/oauth.proto |
| | | 权限控制 | iam/rbac/rbac.proto |

## 事件模型 (30+ 核心业务事件)

所有跨服务的异步通信都通过事件进行，事件定义在 `events/events.proto` 中。事件遵循 CloudEvents 1.0 规范。

### 1. 财务领域 (Finance)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| JournalEntryPostedEvent | 总账凭证已过账 | 总账 (fi/gl) |
| InvoicePostedEvent | 发票已过账 | 发票处理 (pm/iv) |
| PaymentReceivedEvent | 收款已确认 | 应收应付 (fi/ap) |
| CostSettlementEvent | 成本结算完成 | 成本控制 (fi/co) |
| AssetDepreciatedEvent | 固定资产折旧计提 | 财务 (fi/gl) |

### 2. 采购领域 (Procurement)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| PurchaseOrderReleasedEvent | 采购订单已下达 | 采购订单 (pm/po) |
| ContractSignedEvent | 框架协议/合同已签署 | 合同管理 (pm/ct) |
| SourcingEventCompletedEvent | 寻源事件已完成 | 寻源事件 (pm/se) |
| SupplierOnboardedEvent | 供应商已入驻 | 供应商门户 (pm/sp) |

### 3. 销售领域 (Sales)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| SalesOrderCreatedEvent | 销售订单已创建 | 销售订单 (sd/so) |
| RevenueRecognizedEvent | 收入已确认 | 收入确认 (sd/rr) |
| PricingUpdatedEvent | 定价已更新 | 定价引擎 (sd/pe) |

### 4. 供应链领域 (Supply Chain)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| StockChangedEvent | 库存变更 (收/发/转) | 库存管理 (sc/im) |
| ShipmentDeliveredEvent | 货物已送达目的地 | 运输计划 (sc/tp) |
| TransferOrderConfirmedEvent | 移库单已确认 | 仓库运营 (sc/wm) |
| DemandForecastPublishedEvent | 需求预测已发布 | 需求预测 (sc/df) |

### 5. 制造领域 (Manufacturing)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| ProductionOrderReleasedEvent | 生产订单已下达 | 生产计划 (mf/pp) |
| UsageDecisionMadeEvent | 质量使用决策已做出 | 质量检验 (mf/qi) |
| KanbanSignalEvent | 看板信号切换 | 看板服务 (mf/kb) |
| WorkOrderCompletedEvent | 工单报工完成 | 车间执行 (mf/sf) |

### 6. 资产管理 (Asset Management)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| MaintenanceNotificationCreatedEvent | 维护通知已创建 | 资产维护 (am/pm) |
| MaintenanceOrderReleasedEvent | 维护工单已下达 | 资产维护 (am/pm) |
| MachineHealthAlertEvent | 设备健康告警 | 智能健康 (am/ah) |
| IncidentReportedEvent | EHS事件已报告 | EHS事件 (am/eh) |

### 7. 研发领域 (R&D)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| BoMReleasedEvent | 物料清单已发布 | PLM集成 (rd/pl) |
| ProjectWBSCreatedEvent | 项目WBS已创建 | 项目成本 (rd/ps) |

### 8. 人力资源 (HR)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| EmployeeSurveyCompletedEvent | 员工调查已完成 | 员工体验 (hr/ex) |
| CandidateHiredEvent | 候选人已录用 | 人才招聘 (hr/ta) |
| PayrollProcessedEvent | 薪资计算完成 | 人力资源 (hr) |

### 9. 客户服务 (Service)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| ServiceOrderCompletedEvent | 服务订单已完成 | 现场调度 (cs/fd) |
| WarrantyClaimFiledEvent | 保修索赔提交 | 保修索赔 (cs/wc) |
| ContractBilledEvent | 服务合同计费已生成 | 合同计费 (cs/cb) |

### 10. 基础/身份 (IAM)
| 事件类型 | 描述 | 生产者 |
|----------|------|--------|
| UserLoginEvent | 用户登录事件 | 认证中心 (iam/auth) |

## 命名规则

- **产品线 (Domain)**: 2-3字母代码 (fi, pm, mf, sc, am, sd, cs, rd, hr, iam)
- **子领域 (Subdomain)**: 2字母代码 (gl, po, pp, im, so, fd, pl, ta 等)
- **Proto文件**: `{子领域代码}.proto` (如 `gl.proto`, `po.proto`)
- **Package**: `{产品线}.{子领域}.v1` (如 `fi.gl.v1`, `pm.po.v1`)
- **Event Type**: `enterprise.{domain}.{entity}.{action}Event`
