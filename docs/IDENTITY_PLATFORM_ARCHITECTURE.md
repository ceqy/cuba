# 身份平台架构文档

## 概述
本身份平台（Identity Platform）提供统一的用户身份认证、授权、审计以及多因素认证（2FA）功能，面向企业级 SaaS 场景。平台基于 **微服务** 架构实现，核心服务包括 `auth-service`（认证服务）和 `api-gateway`（网关），通过 gRPC 与前端、业务系统交互。

## 关键组件
| 组件 | 主要职责 | 技术栈 |
|------|----------|--------|
| **auth-service** | 用户注册、登录、Token 生成与校验、2FA、OAuth2/OIDC、审计日志、API Key 管理 | Rust、tonic、PostgreSQL |
| **api-gateway** | 统一入口，路由转发、流量控制、统一鉴权 | Nginx / Envoy、Lua 脚本 |
| **数据库** | 持久化用户、角色、权限、审计日志、Token 数据 | PostgreSQL |
| **邮件/短信服务** | 发送验证码、密码重置链接、2FA 恢复码 | 外部第三方服务（如 SendGrid、Twilio） |

## 数据流
1. **注册**：前端调用 `Register` RPC，`auth-service` 写入 `users` 表并返回 `UserInfo`。 
2. **登录**：调用 `Login`，返回 `access_token`、`refresh_token`，若开启 2FA，返回 `temp_token`。 
3. **2FA 验证**：使用 `Verify2FACode`，校验一次性密码后颁发正式 Token。 
4. **Token 刷新**：`RefreshToken` RPC 根据 `refresh_token` 生成新 Token。 
5. **审计日志**：每次关键操作（登录、密码修改、角色变更）记录到 `audit_logs` 表，`GetAuditLogs` 支持分页查询。 

## 安全模型
- **密码**：使用 Argon2 哈希存储。 
- **Token**：JWT 包含 `sub`（用户 ID）和 `tid`（租户 ID），使用 RSA 私钥签名。 
- **2FA**：基于 TOTP（RFC 6238），支持恢复码。 
- **最小权限**：RBAC 通过角色‑权限映射实现，所有 RPC 均在服务端校验用户角色与权限。 

## 部署架构
```
+-------------------+      +-------------------+      +-------------------+
|   前端 (Web/App)  | ---> |   API Gateway     | ---> |  auth-service (Rust) |
+-------------------+      +-------------------+      +-------------------+
                                   |
                                   v
                           +-------------------+
                           |   PostgreSQL DB   |
                           +-------------------+
```
- **容器化**：使用 Docker，K8s 部署，水平扩展 `auth-service`。 
- **监控**：Prometheus 采集 `GetMetrics`，Grafana 可视化。 
- **日志**：审计日志写入数据库，同时通过 Loki 收集。 

## 接口说明（部分）
| RPC | 请求 | 响应 | 说明 |
|-----|------|------|------|
| `Register` | `RegisterRequest` | `RegisterResponse` | 创建新用户，返回 `UserInfo` |
| `Login` | `LoginRequest` | `LoginResponse` | 返回 Access/Refresh Token，若开启 2FA 返回 `temp_token` |
| `Enable2FA` | `Enable2FARequest` | `Enable2FAResponse` | 生成 TOTP 秘钥与二维码 URL |
| `Verify2FASetup` | `Verify2FARequest` | `Verify2FAResponse` | 完成 2FA 启用，返回成功标识 |
| `Verify2FACode` | `Verify2FARequest` | `Verify2FACodeResponse` | 登录阶段校验一次性密码 |
| `GetAuditLogs` | `GetAuditLogsRequest` | `GetAuditLogsResponse` | 分页返回审计日志 |
| `ListAPIKeys` | `ListAPIKeysRequest` | `ListAPIKeysResponse` | 列出用户的 API Key |

---

> **备注**：本文档仅为平台概览，详细设计请参考对应的 `.proto` 文件及代码实现。
