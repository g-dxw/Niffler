# rn01 PostgreSQL 备份与恢复

## 目标

为 `rn01` 上的 Niffler PostgreSQL 创建可恢复的异地备份，并将备份保存到私有
Cloudflare R2 存储桶 `niffler-db-backups`。

目标角色是负责生产数据恢复的运维人员。判断备份成功必须同时满足：

1. PostgreSQL 一致性导出成功。
2. 本地备份结构检查成功。
3. 备份和 SHA-256 校验文件已上传 R2。
4. 从 R2 重新下载的文件校验一致。
5. PostgreSQL 15 隔离实例完整恢复成功，关键表能够读取。

## 非目标

- 本流程不切换数据库主从，不迁移生产数据库。
- 本流程不停止或重启生产 PostgreSQL。
- 本流程不在 `rn01` 上创建恢复测试数据库。
- 本流程不保存图片、视频或 HTTP 正文对象。

## 行为变化

- 首次备份使用 PostgreSQL 15 的自定义格式一致性导出。
- 备份先写入临时文件，完成 `pg_restore --list` 检查和 SHA-256 计算后再上传。
- R2 存储桶保持私有，不启用 `r2.dev` 公共地址或自定义域名。
- 生产主机只保存访问该存储桶所需的最小权限凭据。
- 自动任务每天创建一份完整备份；保留最近 7 份每日备份、4 份每周备份和
  6 份每月备份。清理操作只能删除已超过对应保留期限的备份对象。

## 影响范围

- `pg_dump` 会增加数据库顺序读取、CPU 和网络使用，但不会锁住普通读写请求。
- 临时备份文件会占用 `rn01` 磁盘。执行前必须确认可用空间大于数据库当前体积
  加 10 GB；空间不足时必须停止，不得继续导出。
- 恢复验证只使用本机临时 Docker 容器和临时卷，不连接生产应用。
- R2 费用由备份实际压缩体积和保留数量决定。

## 首次备份

备份对象使用以下目录：

```text
postgres/aether/daily/YYYY/MM/aether-YYYYMMDDTHHMMSSZ.dump
postgres/aether/daily/YYYY/MM/aether-YYYYMMDDTHHMMSSZ.dump.sha256
```

导出参数：

```text
--format=custom
--compress=6
--no-owner
--no-privileges
```

自定义格式由 `pg_dump` 创建一致性快照并压缩，恢复时使用 `pg_restore`。任何一步
失败都必须保留错误并停止后续清理，不能上传截断文件并标记成功。

## 恢复验证

恢复环境必须使用 PostgreSQL 15，且不得暴露公网端口。恢复时使用：

```text
pg_restore --exit-on-error --no-owner --no-privileges
```

恢复完成后至少验证：

- 数据库能够正常连接。
- 用户表数量与备份清单一致。
- `users`、`provider_api_keys`、`usage`、`usage_settlement_snapshots` 可读取。
- 恢复数据库大小合理，且 `pg_restore` 没有忽略错误。

## 凭据

- 本机凭据：`~/.config/domain-transfer/niffler-r2-backup.env`，权限 `0600`。
- 生产凭据必须放在 `/etc/niffler-backup/`，目录权限 `0700`、文件权限 `0600`。
- 凭据不得写入仓库、日志、备份文件名、进程输出或聊天记录。
- 令牌只能读写 `niffler-db-backups`，访问其他存储桶应返回拒绝。

## 验证方式

首次执行记录以下证据：

- 生产导出开始和完成时间。
- 备份压缩体积和 SHA-256。
- R2 上传后对象存在且大小一致。
- 从 R2 下载后的 SHA-256 一致。
- 隔离恢复命令返回成功及关键表读取结果。
- 测试完成后临时容器、卷和本地明文备份已删除。

## 自动任务

项目内文件：

- `scripts/rn01-postgres-backup.sh`
- `scripts/rn01-postgres-backup.service`
- `scripts/rn01-postgres-backup.timer`

生产安装位置：

- `/usr/local/sbin/niffler-postgres-backup`
- `/etc/systemd/system/niffler-postgres-backup.service`
- `/etc/systemd/system/niffler-postgres-backup.timer`

定时器每天北京时间 04:30 执行，并增加最多 10 分钟随机延迟。脚本通过文件锁防止
重复执行，使用较低 CPU 和磁盘优先级，状态写入
`/var/lib/niffler-backup/status.env`，详细日志写入 systemd journal。

检查命令：

```bash
systemctl status niffler-postgres-backup.timer
systemctl list-timers niffler-postgres-backup.timer
journalctl -u niffler-postgres-backup.service
cat /var/lib/niffler-backup/status.env
```

备份服务失败时，systemd 会启动
`niffler-postgres-backup-alert.service`，通过 Telegram Bot 向运维人员发送失败通知。
备份服务成功时，`ExecStartPost` 会发送成功通知。通知凭据位于
`/etc/niffler-backup/telegram.env`，权限必须为 `0600`。

## 2026-07-28 执行结果

首份备份：

- 对象：`postgres/aether/daily/2026/07/aether-20260727T162328Z.dump`
- 大小：1,459,124,818 字节
- SHA-256：`059705e60c37061461b12ac955c3f7ecbca28220224d142389e840918609e113`
- PostgreSQL 15.18 隔离恢复：成功
- 恢复结果：120 张 public 表，关键业务表可读取，无无效索引或未验证约束

自动任务真实运行结果：

- 对象：`postgres/aether/daily/2026/07/aether-20260727T165106Z.dump`
- 大小：1,459,812,843 字节
- SHA-256：`7276373c159414f2ca84116c1d195e2fb4fc94cbd1a56ad7fbf43724b8af3a23`
- systemd 结果：`success`
- 上传后对象大小和校验文件复核：通过
- 本地临时文件清理：通过

成功或失败信息会写入 `/var/lib/niffler-backup/status.env` 和 systemd 日志，并发送
Telegram 私人消息。Telegram API 暂时不可用时，原有备份状态和日志仍会保留；成功
通知发送失败不会改变备份任务本身的成功结果。
