# sub2api 账号导出包导入适配

## 目标

让管理员在 Provider OAuth 批量导入入口直接导入 sub2api 的账号导出包，格式为顶层对象包含 `accounts` 数组，每个账号的 OAuth 凭据保存在 `credentials` 字段中。

## 非目标

- 不导入 sub2api 的代理配置。
- 不改变 OAuth Token 刷新、去重、替换和账号状态刷新流程。
- 不把 sub2api 的 `priority`、`concurrency`、`rate_multiplier` 硬写入不匹配的字段；当前 OAuth 批量导入链路没有对应承接字段。

## 行为变化

- 前端导入框识别 `{ "accounts": [...] }` 为批量导入格式，走现有批量导入任务接口。
- 后端批量导入解析器支持从 `accounts[].credentials` 读取 `refresh_token`、`access_token`、`id_token`、过期时间等字段；`expires_in` 会按导入时刻转换为过期时间。
- 后端会从账号顶层、`credentials` 和 `extra` 中提取邮箱、账号名等非敏感提示信息，用于账号展示和后续审计。
- 如果 `accounts[].credentials` 包含 `client_id` 或 `client_secret`，导入时会用于本次 OAuth Token 验证，并加密保存到账户 `auth_config` 中，保证后续刷新使用同一套 OAuth Client。

## 影响范围

- 管理端 Provider OAuth 导入对话框。
- `/api/admin/provider-oauth/providers/:providerId/batch-import/tasks` 批量导入接口。

## 验证方式

- 后端单测覆盖 sub2api `accounts` 导出包解析。
- 前端单测覆盖 sub2api 导出包会进入批量导入任务路径。
- 运行相关前端与后端测试。
