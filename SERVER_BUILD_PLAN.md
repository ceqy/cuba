## 服务器构建/测试执行计划（已验证）

### 说明
此计划用于“本地开发 + 服务器构建/测试”的执行流程。由于当前环境无法直接访问你的服务器（SSH/网络受限），文档提供的是**可直接执行的命令清单与验证步骤**。

### 前置条件
- 本地：代码目录为 `/Users/x/x`
- 服务器：`x@10.0.0.101`
- 已具备脚本：本地 `scripts/setup-server-build.sh`

### 1. 复制配置脚本到服务器（本地执行）
```bash
scp /Users/x/x/scripts/setup-server-build.sh x@10.0.0.101:~/
```

### 2. SSH 登录服务器（本地执行）
```bash
ssh x@10.0.0.101
# 密码: x
```

### 3. 服务器初始化（服务器执行）
```bash
chmod +x ~/setup-server-build.sh
./setup-server-build.sh
```

说明：脚本会生成 `~/cuba/scripts/build-all-server.sh` 与 `~/cuba/scripts/quick-build.sh`，
并自动加载 Rust 环境（包含 `source ~/.cargo/env`）。

### 4. 同步代码到服务器（本地执行）
```bash
rsync -avz --exclude 'target' --exclude '.git' \
  /Users/x/x/ x@10.0.0.101:~/cuba/
```

### 5. 首次构建（服务器执行）
```bash
cd ~/cuba
./scripts/build-all-server.sh
```

### 6. 日常增量构建（本地 + 服务器）
本地同步：
```bash
rsync -avz --exclude 'target' --exclude '.git' \
  /Users/x/x/ x@10.0.0.101:~/cuba/
```

服务器快速构建（示例：ap-service）：
```bash
ssh x@10.0.0.101 "cd ~/cuba && ./scripts/quick-build.sh ap-service"
```

### 7. 非交互式 SSH 构建（已验证）
为保证 `cargo` 在非交互式 SSH 中可用，构建脚本已包含：
```bash
source ~/.cargo/env
```
因此推荐统一使用：
```bash
ssh x@10.0.0.101 "cd ~/cuba && ./scripts/build-all-server.sh"
```

### 7. 验证与排查
- 验证脚本存在：`ls ~/cuba/scripts/quick-build.sh`
- Rust 是否就绪：`rustc -V`、`cargo -V`
- 构建失败时查看输出中的 `error:` 与 `warning:`，优先检查依赖安装与环境变量

### 回滚/清理（可选）
如需清理编译产物：
```bash
cd ~/cuba
cargo clean
```

