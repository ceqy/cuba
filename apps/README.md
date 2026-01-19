# apps 目录结构与模块说明

本目录包含各业务域的微服务实现，服务名称与 `protos/` 中的域代码一一对应。

## 目录结构约定

```
apps/
├── <domain>/                # 业务域（am/cs/fi/hr/iam/mf/pm/rd/sc/sd）
│   └── <service>-service/   # 具体微服务
│       ├── src/             # Rust 代码
│       │   ├── api/         # gRPC/HTTP 接口层
│       │   ├── application/ # 用例编排、命令与处理器
│       │   ├── domain/      # 领域模型与业务规则
│       │   └── infrastructure/ # 存储、外部依赖适配
│       ├── migrations/      # 数据库迁移脚本
│       ├── Cargo.toml       # 服务依赖
│       └── Dockerfile       # 容器构建
```

## 业务域与模块职责

### am - 资产管理（Asset Management）

- `pm-service`：资产维护、维护通知与工单管理  
- `ah-service`：智能健康（设备状态监测、告警）  
- `eh-service`：EHS 事件（安全/环境/健康事件）  
- `gs-service`：地理位置与定位相关能力  

### cs - 客户服务（Customer Service）

- `fd-service`：现场调度与服务工单  
- `cb-service`：合同计费  
- `wc-service`：保修索赔  

### fi - 财务（Finance）

- `gl-service`：总账  
- `ap-service`：应收应付  
- `co-service`：成本控制  
- `tr-service`：资金管理  
- `coa-service`：会计科目表  
- `uj-service`：统一日记账  
- `ar-service`：应收账款（部分财务能力）  

### hr - 人力资源（HR）

- `ta-service`：人才招聘  
- `ex-service`：员工体验  

### iam - 身份与权限（IAM）

- `auth-service`：认证中心  
- `oauth-service`：OAuth 服务  
- `rbac-service`：权限控制  

### mf - 制造（Manufacturing）

- `pp-service`：生产计划  
- `sf-service`：车间执行  
- `qi-service`：质量检验  
- `kb-service`：看板  
- `om-service`：外协加工  

### pm - 采购（Procurement）

- `po-service`：采购订单  
- `ct-service`：合同管理  
- `iv-service`：发票处理  
- `sp-service`：供应商门户  
- `sa-service`：支出分析  
- `se-service`：寻源事件  

### rd - 研发（R&D）

- `pl-service`：PLM 集成  
- `ps-service`：项目成本  

### sc - 供应链（Supply Chain）

- `im-service`：库存管理  
- `wm-service`：仓库运营  
- `tp-service`：运输计划  
- `df-service`：需求预测  
- `vs-service`：可见性  
- `bt-service`：批次追溯  

### sd - 销售（Sales）

- `so-service`：销售订单  
- `pe-service`：定价引擎  
- `rr-service`：收入确认  
- `an-service`：销售分析  

## 备注

- 服务职责描述与 `protos/README.md` 保持一致，便于从接口定义映射到实现。
