# test01 Niffler 测试环境

`niffler-test` 部署在 `test01:/opt/niffler-test`，公网入口为
`https://niffler-test.123.253.224.101.sslip.io`。应用只监听
`127.0.0.1:18084`，PostgreSQL 和 Redis 仅在 Compose 内部网络开放。
服务器部署资产位于 `deploy/test01/`；运行时 `.env` 只保存在服务器，不进入仓库。

GitHub Actions 只允许从 `test` 分支手动触发。部署 job 使用
`niffler-test` Environment，并通过受限 SSH Key 执行两个固定命令：

```text
niffler-test receive <exact-sha>
niffler-test deploy <exact-sha>
```

固定发布器要求目标等于远端 `test` 头、包含当前 `main` 和当前已部署提交；
目标代码树还必须与当前 `main` 完全一致。随后核对镜像 revision、PostgreSQL
迁移兼容性，创建 `pg_dump` 备份，且只重建
`app` 服务。数据库不会自动回滚。

初次迁移后，管理员新密码以 `INITIAL_ADMIN_PASSWORD` 保存在服务器 root-only
`.env` 中；应用容器不会读取这个名称，避免启动时自动创建迁移白名单之外的钱包。

Environment 配置：

- Secrets: `NIFFLER_TEST_SSH_PRIVATE_KEY`, `NIFFLER_TEST_SSH_HOST_FINGERPRINT`
- Variables: `NIFFLER_TEST_HOST`, `NIFFLER_TEST_USER`, `NIFFLER_TEST_REMOTE_DIR`,
  `NIFFLER_TEST_PUBLIC_URL`

部署完成必须同时核对 Actions `head_sha`、远端镜像 revision、
`.niffler-deployed-commit`、Compose 健康状态和公网 `/health`。
