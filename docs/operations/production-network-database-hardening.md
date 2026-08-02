# 生产网络与数据库连接加固

## 目标

降低 `hd0526` 与 `rn01` 之间生产链路的暴露范围：

1. `hd0526:8084` 只允许本机访问，公网用户只能通过 Caddy 的 80/443 端口进入。
2. Niffler 使用独立的 PostgreSQL 登录账号，不再使用 `postgres` 超级用户。
3. `hd0526` 到 `rn01` 的 PostgreSQL 连接强制使用 TLS 加密。
4. 保留数据库备份脚本使用本机 `postgres` 账号的能力。

## 非目标

- 本次不迁移 PostgreSQL 或 Redis，不改变服务器地区。
- 本次不改变 Cloudflare、域名或 Caddy 路由。
- 本次不开放新的公网端口。
- 本次不宣称 PostgreSQL 已具备双向证书认证；当前应用只支持强制加密，不支持
  校验自建服务器证书身份。
- 没有外部接收地址时，不发送无法验证的备份失败通知。

## 行为变化

- frontdoor 的 Docker 端口从 `0.0.0.0:8084` 改为
  `127.0.0.1:8084`。Caddy 继续通过 Docker 网络访问
  `niffler-frontdoor:8084`，服务器本机健康检查继续访问
  `127.0.0.1:8084`。
- 新建 `niffler_app` PostgreSQL 登录账号，不授予超级用户、创建角色、创建数据库
  或复制权限。该账号拥有 `aether` 数据库和 public 业务对象，以便执行项目已有的
  SQLx 数据库迁移。
- PostgreSQL 启用服务器 TLS，只接受远程 TLS 登录；Unix Socket 本机连接继续
  可供健康检查和备份使用。
- frontdoor 与 background 设置
  `AETHER_GATEWAY_DATA_POSTGRES_REQUIRE_SSL=true`。

## 影响范围

- 重新创建 frontdoor 容器时可能出现数秒连接切换。
- PostgreSQL 启用 TLS 需要重新创建数据库容器，应用会在数据库恢复后重新连接。
- 切换数据库账号需要重新创建 frontdoor 和 background 容器。
- 数据库对象只改变所有者，不改变表结构和数据。

## 执行顺序

1. 保存两台服务器的 Compose、环境变量和 PostgreSQL 访问规则副本。
2. 收紧 `hd0526:8084`，验证公网拒绝、本机健康检查和域名访问正常。
3. 创建非超级用户账号并转移 public 业务对象所有权。
4. 生成仅用于 PostgreSQL 的服务器证书，启用 TLS 和远程 `hostssl` 访问规则。
5. 更新应用数据库连接和强制 TLS 设置，重新创建应用容器。
6. 验证应用连接用户、TLS 状态、迁移权限、公开健康接口和备份任务。

## 回退

- `8084`：恢复变更前的 `docker-compose.yml` 并重新创建 frontdoor。
- 数据库账号：恢复应用原连接串即可，`postgres` 登录不会在本次删除。
- TLS：恢复变更前的 PostgreSQL Compose 和访问规则后重新创建数据库容器。
- 所有回退文件只保存在对应生产服务器的 root 专用目录，权限为 `0600`。

## 验证方式

- 外部连接 `hd0526:8084` 必须失败，本机 `127.0.0.1:8084` 必须成功。
- `https://niffler.org/` 和公开健康接口必须正常。
- PostgreSQL 中应用会话的用户必须是 `niffler_app`，且 `ssl=true`。
- `niffler_app` 的 `rolsuper`、`rolcreaterole`、`rolcreatedb` 和
  `rolreplication` 必须全部为 false。
- frontdoor、background、PostgreSQL 和 Redis 容器必须健康。
- 自动备份服务仍能成功执行结构检查。

## 备份失败通知

`rn01` 使用 Telegram Bot `niffler_ops_alert_bot` 发送备份失败通知。Bot Token 和
私人 Chat ID 保存在 `/etc/niffler-backup/telegram.env`，权限为 `0600`，不得写入
仓库、日志或命令输出。

`niffler-postgres-backup.service` 失败后由 systemd 启动
`niffler-postgres-backup-alert.service`。通知包含服务器、备份任务状态、退出码、
备份编号和对象路径，不包含数据库密码、R2 凭据、Telegram Token 或备份内容。

每次成功或失败备份都发送消息。首次安装必须发送一条明确标注“测试”的消息，并
检查 Telegram API 返回成功。

2026-07-28 已完成首次测试，Telegram API 返回成功，测试消息已投递。
2026-07-28 本次定时备份的成功消息已补发，Telegram API 返回成功。

## 2026-07-28 执行结果

- `hd0526` 的 frontdoor 只监听 `127.0.0.1:8084`。外部连接测试被拒绝，本机健康
  接口和 Caddy 域名入口正常。
- PostgreSQL 数据库和 120 张 public 业务表、2 个序列、4 个枚举类型已由
  `niffler_app` 拥有。
- `niffler_app` 没有超级用户、建库、建角色或复制权限；事务内建表、改表、建索引
  测试通过并完整回滚。
- PostgreSQL 已启用 TLS，自签名服务器证书有效期至 2028-10-29。
- 远程明文连接测试被 `pg_hba.conf` 拒绝，TLS 测试连接使用 TLSv1.3。
- frontdoor 与 background 已设置
  `AETHER_GATEWAY_DATA_POSTGRES_REQUIRE_SSL=true`。最终检查时 15 条远程应用会话
  全部为 `niffler_app` 和 TLSv1.3。
- frontdoor、background、Caddy、PostgreSQL 和 Redis 正常；公开健康接口、认证
  设置接口和主页均返回 HTTP 200。
- 自动备份定时器保持启用和 active，无数据的备份结构检查通过。
- 回退文件保存在两台服务器的
  `/root/niffler-hardening-backup-20260728T0120Z/`，权限仅 root 可读。

应用当前强制加密，但 SQLx 的 `Require` 模式不会验证自签名证书身份。后续需要在
应用支持 `verify-ca` 或 `verify-full` 后分发私有 CA，才能同时验证服务器身份。
