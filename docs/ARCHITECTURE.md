# CUBA 架构文档

## 1. 架构概述

CUBA (Corporate Unified Business Architecture) 是一个基于 Rust 构建的企业级微服务架构，采用领域驱动设计（DDD）、命令查询职责分离（CQRS）和事件溯源（Event Sourcing）模式。

### 1.1 核心设计原则

- **领域驱动设计（DDD）**: 每个微服务围绕一个清晰的业务领域边界构建
- **CQRS**: 命令和查询分离，优化读写性能
- **事件溯源**: 通过事件流记录所有状态变更
- **事件驱动架构（EDA）**: 服务间通过异步事件实现解耦
- **API 优先**: 使用 gRPC 和 Protocol Buffers 定义强类型服务契约

### 1.2 技术栈

| 组件 | 技术选型 | 说明 |
|------|---------|------|
| 编程语言 | Rust 2024 | 高性能、内存安全 |
| RPC 框架 | gRPC (Tonic) | 高效的服务间通信 |
| 数据库 | PostgreSQL | 事件存储和读模型 |
| 消息队列 | Apache Kafka | 事件总线 |
| 容器化 | Docker | 服务打包和部署 |
| 编排 | Docker Compose / Kubernetes | 本地开发 / 生产环境 |

## 2. 系统架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        API Gateway                           │
│                    (gRPC / REST 转换)                      ────────────────────────┬────────────────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
┌───────▼──────┐  ┌─────▼──────┐  ┌─────▼──────┐
│   Finance    │  │ Procurement │  │   Sales    │
│   Services   │  │  Services   │  │  Services  │
│   (5个)      │  │   (6个)     │  │   (4个)    │
└──────┬───────┘  └──────┬──────┘  └──────┬─────┘
       │               │                 │
       └─────────────────┼─────────────────┘
                         │
                ┌────────▼─────────┐
                │   Event Bus      │
                │   (Kafka)        │
                └────────┬─────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
┌───────▼──────┐  ┌─────▼──────┐  ──────┐
│Manufacturing │  │Supply Chain│  │   Asset    │
│  Services    │  │  Services  │  │  Services  │
│   (│  │   (6个)    │  │   (4个)    │
└──────────────┘  └────────────┘  └────────────┘
```

### 2.2 服务清单（40个微服务）

#### 财务领域 (Finance) - 5个服务
1. **总账凭证服务** (GL Journal Entry Service)
2. **应收* (AR/AP Service)
3. **成本分配服务** (Controlling Allocation Service)
4. **资金管理服务** (Treasury Service)
5. **集团合并服务** (Consolidation Service - TBD)

#### 采购领域 (Procurement) - 6个服务
1. **采购订单服务** (Purchase Order Service)
2. **合同管理服务** (Contract Management Service)
3. **发票处理服务** (Invoice Processing Service)
4. **供应商门户服务** (Supplier Portal Service)
5. **支出分析服务** (Spend Analytics Service)
6. **寻源事件服务** (Sourcing Event Service)

#### 制造领域 (Manufacturing) - 5个服务
1. **生产计划服务** (Production Planning Service)
2. **车间执行服务** (Shop Floor Exion Service)
3. **质量检验服务** (Quality Inspection Service)
4. **看板服务** (Kanban Service)
5. **外协加工服务** (Outsourced Manufacturing Service)

#### 供应链领域 (Supply Chain) - 6个服务
1. **库存管理服务** (Inventory Management Service)
2. **仓库运营服务** (Warehouse Operations Service)
3. **运输计划服务** (Transportation Planning Service)
4. **需求预测服务** (Demand Forecasting Service)
5. **可见性服务** (Visibility Service)
6. **批次追溯服务** (Batch Traceability Service)

#### 资产管理领域 (Asset) - 4个服务
1. **资产维护服务** (Asset Maintenance Service)
2. **智能资产健康服务** (Intelligent Asset Health Service)
3. **EHS事件服务** (EHS Incident Service)
4. **地理位置服务** (Geo Service)

#### 销售领域 (Sales) - 4个服务
1. **销售订单履行服务** (Sales Order Fulfillment Service)
2. **定价引擎服务** (Pricing Engine Service)
3. **收入确认服务** (Revenue Recognition Service)
4. **销售分析服务** (Analytics Service)

#### 服务领域 (Service) - 3个服务
1. **现场服务调度服务** (Field Service Dispatch Service)
2. **服务合同计费服务** (Contract Billing Service)
3. **保修索赔服务** (Warranty Claims Service)

#### 研发领域 (R&D) - 2个服务
1. **PLM集成服务** (PLM Integration Service)
2. **项目成本控制服务** (Project Cost Controlling Service)

#### 人力资源领域 (HR) - 2个服务
1. **人才招聘服务** (Talent Acquisition Service)
2. **员工体验服务** (Employee Experience Service)

#### 基础服务 (Infrastructure) - 3个服务
1. **认证服务** (Auth Service)
2. **配置服务** (Config Service - TBD)
3. **通知服务** (Notification Service - TBD)

## 3. 分层架构

每个微服务遵循清晰的分层架构：

```
┌─────────────────────────────────────┐
│         Presentation Layer          │
│         (gRPC Handlers)             │
├─────────────────────────────────────┤
│        Application Layer            │
│   (Command/Query Handlers)          │
├─────────────────────────────────────┤
│          Domain Layer               │
│  (Aggregates, Entities, Events)     │
├─────────────────────────────────────┤
│       Infrastructure Layer          │
│  (Repositories, Event Store, DB)    │
└─────────────────────────────────────┘
```

### 3.1 各层职责

#### Presentation Layer (表现层)
- gRPC 服务实现
- 请求验证
- DTO 转换
- 错误处理

#### Application Layer (应用层)
- 命令处理器（Command Handlers）
- 查询处理器（Query Handlers）
- 应用服务编排
- 事务管理

#### Domain Layer (领域层)
- 聚合根（Aggregates）
- 实体（Entities）
- 值对象（Value Objects）
- 领域事件（Domain Events）
- 领域服务（Domain Services）

#### Infrastructure Layer (基础设施层)
- 数据库访问（Repository 实现）
- 事件存储（Event Store）
- 消息发布（Event Publisher）
- 外部服务集成

## 4. 数据架构

### 4.1 事件溯源模式

每个聚合的状态变更都通过事件记录：

```sql
-- 事件存储表结构
CREATE TABLE events (
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB,
    version BIGINT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    UNIQUE(aggregate_id, version)
);

CREATE INDEX idx_events_aggregate ON events(aggregate_id, version);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_created ON events(created_at);
```

### 4.2 读模型（Read Model）

为查询优化的非规范化视图：

```sql
-- 示例：销售订单读模型
CREATE TABLE sales_order_read_model (
    order_id UUID PRIMARY KEY,
    order_number VARCHAR(50) NOT NULL,
    customer_id VARCHAR(50) NOT NULL,
    customer_name VARCHAR(200),
    total_amount DECIMAL(15,2),
    currency VARCHAR(3),
    status VARCHAR(50),
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);
```

### 4.3 数据库策略

- **每服务独立数据库**: 每个微服务拥有自己的数据库实例
- **事件存储**: 使用 PostgreSQL 存储事件流
- **读写分离**: 写操作使用事件溯源，读操作使用优化的读模型
- **最终一致性**: 通过事件传播实现跨服务数据一致性

## 5. 事件驱动架构

### 5.1 事件类型

所有事件遵循 CloudEvents 规范：

```protobuf
message CloudEvent {
    string id = 1;                    // 事件唯一ID
    string source = 2;                // 事件源服务
    string spec_version = 3;          // CloudEvents版本
    string type = 4;                  // 事件类型
    google.protobuf.Timestamp time = 5;
    google.protobuf.Any data = 6;     // 事件数据
    map<string, string> attributes = 7;
}
```

### 5.2 关键事件

| 事件类型 | 生产者 | 消费者 |
|---------|--------|--------|
| JournalEntostedEvent | 总账凭证服务 | 成本分配服务、报表服务 |
| PurchaseOrderReleasedEvent | 采购订单服务 | 库存管理服务、发票处理服务 |
| SalesOrderCreatedEvent | 销售订单服务 | 库存管理服务、定价引擎 |
| StockChangedEvent | 库存管理服务 | 生产计划服务、销售服务 |
| ProductionOrderReleasedEvent | 生产计划服务 | 车间执行服务、库存服务 |

### 5.3 事件流处理

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│   Service A  │─────▶│    Kafka     │─────▶│   Service B  │
│  (Producer)  │      │   (Topic)    │      │  (Consumer)  │
└──────────────┘      └──────────────┘      └──────────────┘
                             │
                             ▼
                      ┌──────────────┐
                      │   Service C  │
                      │  (Consumer)  │
                      └──────────────┘
```

## 6. 共享库（Shared Libraries）

### 6.1 cuba-core
核心 DDD/CQRS 抽象和 trait：
- `Aggregate` trait
- `DomainEvent` trait
- `Command` / `CommandHandler` trait
- `Query` / `QueryHandler` trait
- `Repository` trait
- `EventPublisher` trait

### 6.2 cuba-config
配置管理：
- YAML 配置文件加载
- 环境变量覆盖
- 配置验证

### 6.3 cuba-errors
统一错误处理：
- 领域错误类型
- gRPC 错误映射
- 错误码标准化

### 6.4 cuba-database
数据库工具：
- SQLx 连接池管理
- 事件存储实现
- Repository 基础实现

### 6.5 cuba-messaging
Kafka 集成：
- 事件发布器
- 事件消费者
- 序列化/反序列化

### 6.6 cuba-telemetry
可观测性：
- 分布式追踪（Tracing）
- 指标收集（Metrics）
- 结构化日志

## 7. 安全架构

### 7.1 认证与授权

```
┌─────────┐      ┌──────────────┐      ┌─────────────┐
│ Client  │─────▶│ Auth Service │─────▶│   Service   │
└─────────┘      │  (JWT Token) │      │ (Validate)  │
                 └──────────────┘      └─────────────┘
```

- **认证**: JWT Token 基于认证
- **授权**: 基于角色的访问控制（RBAC）
- **传输安全**: TLS/mTLS 加密通信

### 7.2 数据安全

- **敏感数据加密**: 数据库字段级加密
- **审计日志**: 所有操作记录审计事件
- **数据脱敏**: 日志和监控中的敏感信息脱敏

## 8. 可观测性

### 8.1 三大支柱

1. **日志（Logging）**
   - 结构化日志（JSON 格式）
   - 统一日志级别
   - 关联 ID 追踪

2. **指标（Metrics）**
   - 请求延迟
   - 错误率
   - 吞吐量
   - 资源使用率

3. **追踪（Tracing）**
   - 分布式追踪
   - 服务依赖图
   - 性能瓶颈分析

### 8.2 监控栈

```
┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│  Services    │─────▶│  Prometheus  │─────▶│   Grafana    │
│  (Metrics)   │      │  (Collect)   │      │  (Visualize) │
└──────────────┘      └──────────────┘      └──────────────┘

┌──────────────┐      ┌──────────────┐      ┌──────────────┐
│  Services    │─────▶│    Jaeger    │─────▶│   Jaeger UI  │
│  (Traces)    │      │   (Collect)  │      │  (Visualize) │
└──────────────┘      └──────────────┘      └──────────────┘
```

## 9. 部署架构

### 9.1 本地开发环境

使用 Docker Compose 快速启动：

```yaml
services:
  postgres:
    image: postgres:16
  kafka:
    image: confluentinc/cp-kafka:latest
  auth-service:
    build/auth-service
  sales-service:
    build: ./apps/sales-service
```

### 9.2 生产环境（Kubernetes）

```
┌────────────────────────────────────┐
│              Kubernetes Cluster              │
│                                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  │
│  │  Pod 1   │  │  Pod 2   │  │  Pod 3   │  │
│  │ Service  │  │ Service  │  │ Service  │  │
│  └──────────┘  └──────────┘  └──────────┘  │
│                                              │
│  ┌──────────────────────────────────────┐  │
│  │         Ingress Controller            │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

关键特性：
- **水平扩展**: 根据负载自动扩缩容
- **健康检查**: Liveness 和 Readiness 探针
- **滚动更新**: 零停机部署
- **服务发现**: Kubernetes DNS

## 10. 性能优化

### 10.1 缓存策略

- **本地缓存**: 热点数据内存缓存
- **分布式缓存**: Redis 集群
- **CDN**: 静态资源加速

### 10.2 数据库优化

- **连接池**: 复用数据库连接
- **索引优化**: 查询性能优化
- **读写分离**: 主从复制
- **分片**: 水平分表

### 10.3 消息队列优化

- **批量处理**: 批量消费事件
- **并行消费**: 多分区并行处理
- **背压控制**: 防止消费者过载

## 11. 灾难恢复

### 11.1 备份策略

- **数据库备份**: 每日全量 + 实时增量
- **事件存储备份**: 持久化到对象存储
- **配置备份**: 版本控制

### 11.2 故障恢复

- **服务降级**: 非关键功能降级
- **熔断机制**: 防止级联故障
- **重试策略**: 指数退避重试
- **事件重放**: 从事件流重建状态

## 12. 扩展性考虑

### 12.1 垂直扩展

- 增加单个服务实例的资源（CPU、内存）
- 适用于计算密集型服务

### 12.2 水平扩展

- 增加服务实例数量
- 通过负载均衡分发请求
- 适用于大多数无状态服务

### 12.3 数据分片

- 按业务维度分片（如按公司代码）
- 按时间分片（如按年度）
- 提高数据库吞吐量

## 13. 未来演.1 短期目标

- 完成所有 40 个微服务的实现
- 建立完整的 CI/CD 流程
- 完善监控和告警体系

### 13.2 中期目标

- 引入服务网格（Service Mesh）
- 实现多租户支持
- 增强安全审计能力

### 13.3 长期目标

- 支持多云部署
- AI/ML 能力集成
- 实时数据分析平台
