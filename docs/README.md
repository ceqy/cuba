# CUBA ERP 文档中心

欢迎使用 CUBA ERP 文档！这里包含了所有你需要的文档和指南。

## 📚 文档目录

### 🚀 快速开始

- **[开发环境搭建](../README.md)** - 如何设置和启动开发环境
- **[服务测试脚本](../scripts/test-services.sh)** - 自动化测试所有服务

### 📖 API 文档

- **[API 完整文档](./API_DOCUMENTATION.md)** - 所有服务的 API 参考文档
  - IAM 模块 (Auth Service, RBAC Service)
  - 财务模块 (GL, AP, AR, COA Service)
  - 请求/响应格式
  - 错误处理
  - 多语言客户端示例

- **[API 使用示例](./API_EXAMPLES.md)** - 实用的 API 调用示例
  - 认证流程完整示例
  - 用户管理操作
  - 权限管理操作
  - 财务操作示例
  - 常见业务场景
  - 最佳实践
  - 故障排查指南

### 🛠️ 技术文档

- **[架构设计](./ARCHITECTURE.md)** - 系统架构和设计原则 *(待创建)*
- **[数据库设计](./DATABASE_SCHEMA.md)** - 数据库表结构和关系 *(待创建)*
- **[部署指南](./DEPLOYMENT.md)** - 生产环境部署指南 *(待创建)*

### 📋 Skills 文档

- **[Skills 完整指南](../.claude/skills/README.md)** - Claude Code Skills 使用指南
- **[Skills 快速参考](../.claude/skills/QUICK_REFERENCE.md)** - 常用命令速查

## 🎯 快速链接

### 常用命令

```bash
# 启动开发环境
docker-compose up -d

# 运行测试
./scripts/test-services.sh

# 查看服务状态
docker ps

# 查看服务日志
docker logs cuba-auth-service -f

# 连接数据库
docker exec -it cuba-postgres psql -U postgres -d cuba_iam
```

### 服务端点

| 服务 | 端口 | 文档链接 |
|------|------|----------|
| Auth Service | 50051 | [API 文档](./API_DOCUMENTATION.md#auth-service) |
| RBAC Service | 50052 | [API 文档](./API_DOCUMENTATION.md#rbac-service) |
| GL Service | 50060 | [API 文档](./API_DOCUMENTATION.md#gl-service) |
| AP Service | 50061 | [API 文档](./API_DOCUMENTATION.md#ap-service) |
| AR Service | 50062 | [API 文档](./API_DOCUMENTATION.md#ar-service) |
| COA Service | 50065 | [API 文档](./API_DOCUMENTATION.md#coa-service) |
| PostgreSQL | 5432 | - |

### API 示例

**注册用户**:
```bash
grpcurl -plaintext -d '{
  "username": "john_doe",
  "email": "john@example.com",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Register
```

**登录**:
```bash
grpcurl -plaintext -d '{
  "username": "john_doe",
  "password": "SecurePass123!",
  "tenant_id": "default"
}' localhost:50051 iam.auth.v1.AuthService/Login
```

**创建会计分录**:
```bash
grpcurl -plaintext \
  -H "authorization: Bearer YOUR_TOKEN" \
  -d '{
    "companyCode": "1000",
    "documentDate": "2026-01-19",
    "postingDate": "2026-01-19",
    "documentType": "SA",
    "lineItems": [
      {"account": "110000", "debitCredit": "D", "amount": 10000, "currency": "CNY"},
      {"account": "600000", "debitCredit": "C", "amount": 10000, "currency": "CNY"}
    ],
    "postImmediately": true
  }' localhost:50060 fi.gl.v1.GlJournalEntryService/CreateJournalEntry
```

更多示例请查看 [API 使用示例](./API_EXAMPLES.md)。

## 🔍 按场景查找

### 我想...

- **设置开发环境** → [开发环境搭建](../README.md)
- **了解 API 接口** → [API 完整文档](./API_DOCUMENTATION.md)
- **查看代码示例** → [API 使用示例](./API_EXAMPLES.md)
- **实现用户认证** → [认证流程示例](./API_EXAMPLES.md#认证流程)
- **管理用户权限** → [权限管理示例](./API_EXAMPLES.md#权限管理)
- **创建会计分录** → [财务操作示例](./API_EXAMPLES.md#财务操作)
- **排查问题** → [故障排查指南](./API_EXAMPLES.md#故障排查)
- **使用 Claude Skills** → [Skills 指南](../.claude/skills/README.md)

## 📦 项目结构

```
cuba-erp/
├── apps/                    # 微服务应用
│   ├── iam/                # 身份认证模块
│   │   ├── auth-service/
│   │   └── rbac-service/
│   └── fi/                 # 财务模块
│       ├── gl-service/
│       ├── ap-service/
│       ├── ar-service/
│       └── coa-service/
├── docs/                   # 📖 文档目录（你在这里）
│   ├── README.md          # 文档索引
│   ├── API_DOCUMENTATION.md
│   └── API_EXAMPLES.md
├── scripts/               # 工具脚本
│   └── test-services.sh
├── .claude/               # Claude Code 配置
│   └── skills/           # Skills 定义
├── docker-compose.yaml   # Docker 编排配置
└── README.md            # 项目主 README
```

## 🤝 贡献指南

### 文档贡献

欢迎贡献文档！请遵循以下规范：

1. **Markdown 格式** - 使用标准 Markdown 语法
2. **代码示例** - 提供可运行的完整示例
3. **清晰的标题** - 使用层级标题组织内容
4. **实用性** - 关注实际使用场景

### 提交文档

```bash
# 1. 创建或修改文档
vim docs/NEW_DOCUMENT.md

# 2. 提交更改
git add docs/
git commit -m "docs: 添加新文档"
git push
```

## 📞 获取帮助

### 问题反馈

- **GitHub Issues**: [提交 Issue](https://github.com/your-org/cuba-erp/issues)
- **邮件**: dev@cuba.local
- **文档问题**: 直接在文档中提 PR

### 常见问题

**Q: 服务启动失败怎么办？**
A: 查看 [故障排查指南](./API_EXAMPLES.md#故障排查)

**Q: 如何获取 API Token？**
A: 查看 [认证流程](./API_EXAMPLES.md#认证流程)

**Q: 数据库表不存在？**
A: 需要运行数据库迁移，参考 [开发环境搭建](../README.md)

**Q: 如何使用 Claude Skills？**
A: 查看 [Skills 指南](../.claude/skills/README.md)

## 🔗 相关资源

### 官方文档

- [gRPC 官方文档](https://grpc.io/docs/)
- [Protocol Buffers](https://protobuf.dev/)
- [PostgreSQL 文档](https://www.postgresql.org/docs/)
- [Docker 文档](https://docs.docker.com/)

### 工具

- [grpcurl](https://github.com/fullstorydev/grpcurl) - gRPC 命令行工具
- [BloomRPC](https://github.com/bloomrpc/bloomrpc) - gRPC GUI 客户端
- [Postman](https://www.postman.com/) - API 测试工具（支持 gRPC）

### 学习资源

- [gRPC 入门教程](https://grpc.io/docs/languages/python/quickstart/)
- [Protocol Buffers 教程](https://protobuf.dev/getting-started/)
- [微服务架构模式](https://microservices.io/patterns/)

## 📊 文档统计

- **API 文档**: 1 个主文档
- **示例文档**: 1 个完整示例集
- **Skills 文档**: 14+ 个 Skills
- **代码示例**: 50+ 个实用示例
- **支持语言**: Python, Go, Node.js, Bash

## 🎉 开始使用

1. **阅读** [API 完整文档](./API_DOCUMENTATION.md) 了解所有 API
2. **运行** [测试脚本](../scripts/test-services.sh) 验证环境
3. **参考** [API 使用示例](./API_EXAMPLES.md) 开始开发
4. **使用** [Claude Skills](../.claude/skills/README.md) 提高效率

---

**文档版本**: 1.0.0
**最后更新**: 2026-01-19
**维护团队**: CUBA Enterprise Team

**许可证**: Apache-2.0
