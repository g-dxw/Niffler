# 嵌套 ChatGPT 授权 JSON 导入兼容

## 目标

Provider 的“导入授权”入口支持单账号嵌套授权 JSON：OAuth 凭据位于 `tokens` 对象，账号说明信息位于 `_meta` 对象。

## 非目标

- 不接受任意深层字段搜索，只兼容本文明确列出的结构。
- 不导入 `openai_api_key`、`agent_identity`、`personal_access_token` 或 `bedrock_api_key`。
- 不改变 JSON 数组、JSON Lines、逐行 Token 和现有顶层字段格式的处理方式。
- 不在日志、错误提示或测试数据中记录真实 Token。

## 行为变化

当单个 JSON 对象包含 `tokens` 时，前端按以下规则读取：

- `tokens.access_token` → `access_token`
- `tokens.refresh_token` → `refresh_token`
- `tokens.id_token` → `id_token`
- `tokens.account_id` → `account_id`
- `_meta.email` → `email`
- `_meta.plan_type` → `plan_type`
- `last_refresh` → `last_refresh`

同一字段同时出现在顶层和嵌套对象时，继续优先使用现有顶层字段，避免改变旧格式语义。只有 `tokens` 中存在有效 Access Token 或 Refresh Token 时，才把该对象识别为可导入授权。

## 影响范围

- 仅影响前端 Provider OAuth 单条授权导入解析。
- 后端接口和持久化结构不变。
- Codex、ChatGPT Web 等使用现有单条 OAuth 导入接口的 Provider 可以使用该格式。

## 验证方式

- 使用脱敏数据验证嵌套 Token、账号 ID、邮箱、套餐和最后刷新时间被映射到现有单条导入请求。
- 验证顶层字段优先于嵌套字段。
- 验证缺少有效 Access Token 和 Refresh Token 的嵌套对象仍然提示格式错误。
- 运行现有 OAuth 导入回归测试，确认旧格式和批量导入行为不变。
