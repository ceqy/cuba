# CUBA ERP - 服务端口分配表

## 端口分配规则

- **50051-50059**: IAM 身份认证服务
- **50060-50069**: FI 财务服务
- **50070-50079**: SD 销售与分销服务
- **50080-50089**: PM 采购管理服务
- **50090-50099**: MF 制造管理服务
- **50100-50109**: SC 供应链管理服务
- **50110-50119**: HR 人力资源服务
- **50120-50129**: AM 资产管理服务
- **50130-50139**: CS 客户服务
- **50140-50149**: RD 研发管理服务

## 当前端口分配

### IAM - Identity & Access Management (50051-50059)
| 服务 | 端口 | 说明 |
|------|------|------|
| auth-service | 50051 | 认证服务 |
| rbac-service | 50052 | 基于角色的访问控制 |
| oauth-service | 50053 | OAuth2 服务 |

### FI - Finance (50060-50069)
| 服务 | 端口 | 说明 | 原端口 | 状态 |
|------|------|------|--------|------|
| gl-service | 50060 | 总账 | 50052 ⚠️ | 需修改 |
| ap-service | 50061 | 应付账款 | 50053 ⚠️ | 需修改 |
| ar-service | 50062 | 应收账款 | 50054 | OK |
| co-service | 50063 | 成本控制 | 50055 | OK |
| tr-service | 50064 | 资金管理 | 50056 | OK |
| coa-service | 50065 | 会计科目表 | 50057 | OK |

### SD - Sales & Distribution (50070-50079)
| 服务 | 端口 | 说明 |
|------|------|------|
| so-service | 50070 | 销售订单 |
| pe-service | 50071 | 定价引擎 |
| rr-service | 50072 | 退货管理 |
| an-service | 50073 | 销售分析 |

### PM - Procurement Management (50080-50089)
| 服务 | 端口 | 说明 |
|------|------|------|
| po-service | 50080 | 采购订单 |
| vs-service | 50081 | 供应商管理 |
| ct-service | 50082 | 合同管理 |
| iv-service | 50083 | 发票验证 |
| sa-service | 50084 | 采购分析 |
| se-service | 50085 | 采购策略 |

### MF - Manufacturing (50090-50099)
| 服务 | 端口 | 说明 |
|------|------|------|
| om-service | 50090 | 生产订单 |
| sf-service | 50091 | 车间管理 |
| qi-service | 50092 | 质量检验 |
| kb-service | 50093 | 看板管理 |
| pp-service | 50094 | 生产计划 |

### SC - Supply Chain (50100-50109)
| 服务 | 端口 | 说明 |
|------|------|------|
| im-service | 50100 | 库存管理 |
| wm-service | 50101 | 仓库管理 |
| tp-service | 50102 | 运输管理 |
| df-service | 50103 | 需求预测 |
| bt-service | 50104 | 批次追踪 |
| vs-service | 50105 | 供应商管理 |

### HR - Human Resources (50110-50119)
| 服务 | 端口 | 说明 |
|------|------|------|
| ta-service | 50110 | 考勤管理 |
| ex-service | 50111 | 费用报销 |

### AM - Asset Management (50120-50129)
| 服务 | 端口 | 说明 |
|------|------|------|
| ah-service | 50120 | 资产层级 |
| eh-service | 50121 | 设备健康 |
| gs-service | 50122 | 地理空间 |
| pm-service | 50123 | 预防性维护 |

### CS - Customer Service (50130-50139)
| 服务 | 端口 | 说明 |
|------|------|------|
| cb-service | 50130 | 案例管理 |
| fd-service | 50131 | 反馈管理 |
| wc-service | 50132 | 工单管理 |

### RD - R&D (50140-50149)
| 服务 | 端口 | 说明 |
|------|------|------|
| pl-service | 50140 | 产品生命周期 |
| ps-service | 50141 | 项目管理 |

## 端口冲突修复清单

### 需要修改的服务

1. **gl-service**: 50052 → 50060
   - 文件: `apps/fi/gl-service/src/main.rs`
   - 配置: `deploy/k8s/values/gl-service.yaml`
   - Envoy: `deploy/envoy/envoy.yaml`

2. **ap-service**: 50053 → 50061
   - 文件: `apps/fi/ap-service/src/main.rs`
   - 配置: `deploy/k8s/values/ap-service.yaml`
   - Envoy: `deploy/envoy/envoy.yaml`

## 修改步骤

1. 更新服务代码中的端口号
2. 更新 K8s Deployment 配置
3. 更新 Envoy 网关路由配置
4. 更新服务间调用的端点地址
5. 更新文档和注释

## 验证清单

- [ ] 所有服务端口无冲突
- [ ] Envoy 路由配置正确
- [ ] 服务间调用端点正确
- [ ] K8s Service 端口映射正确
- [ ] 文档已更新
