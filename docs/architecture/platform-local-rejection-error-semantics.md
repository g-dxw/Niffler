# 平台本地拒绝错误语义

## 目标

让用户侧请求返回的 HTTP 状态码、错误码和文案能直接说明失败原因，避免把认证、余额、权限和限流问题混在一起。

## 非目标

- 不改变上游真实返回错误的透传和改写策略。
- 不新增自动封禁、自动充值或自动重试行为。
- 不改变管理员后台对历史 usage 的查询结构。

## 行为变化

- API Key 缺失或无效返回 `401`，错误类型为 `authentication_error`，错误码为 `missing_api_key` 或 `invalid_api_key`。
- 余额不足返回 `402`，错误码统一为 `insufficient_balance`，不再返回 `429`。
- 只有请求频率或并发限制才返回 `429`，错误码为 `rate_limit_exceeded`。
- API Key 锁定、Provider 不允许、接口格式不允许、模型不允许继续返回 `403`，但错误类型不再使用泛化的 `http_error`。
- 平台错误文案规则继续兼容历史 `balance_exceeded`，管理员已有的余额不足文案规则仍可命中。

## 影响范围

- 用户侧 AI 请求的本地认证、余额、权限和限流拒绝响应。
- 管理端“内容文案 / 错误返回规则”中平台错误原因的默认状态码。
- 使用记录中的失败状态码和错误分类。

## 验证方式

- 后端测试覆盖余额不足返回 `402 insufficient_balance`。
- 后端测试覆盖 API Key 缺失返回 `401 authentication_error` 且携带 `missing_api_key`，无效返回 `invalid_api_key`。
- 后端测试覆盖平台错误文案规则对历史 `balance_exceeded` 的兼容。
- 前端测试或类型检查覆盖平台错误原因列表中的状态码展示。
