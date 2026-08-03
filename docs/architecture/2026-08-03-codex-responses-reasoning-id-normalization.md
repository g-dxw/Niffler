# Codex Responses 历史推理项 ID 兼容修复

## 目标

兼容旧客户端或旧历史记录将 OpenAI Responses `reasoning` 项 ID 写成 `item_*` 的请求，
避免上游因前缀不符合 `rs_*` 约束直接返回 HTTP 400；同一确定性参数错误不得重复切换账号。

## 非目标

- 不修改已经符合协议的 `rs_*` ID。
- 不修正其他类型的历史项 ID。
- 不接受空 ID、无前缀 ID 或任意未知前缀。
- 不改变认证、模型路由、计费或正常的临时故障切换行为。

## 行为变化

Codex 提供商的 OpenAI Responses 请求在发送前检查 `input` 数组：

- 项类型必须为 `reasoning`；
- ID 必须为非空 `item_*`；
- 满足以上条件时保留原后缀，只将前缀改为 `rs_`；
- 其他输入保持原样。

当上游直接返回 HTTP 400 且错误正文表明 `invalid_request_error` 时，执行流程立即返回该错误，
不再尝试其他账号。其他可重试故障继续遵守既有策略。

## 影响范围

仅影响 Codex 提供商的 `/v1/responses` 请求转换和该请求族的账号切换判断。
普通 OpenAI 提供商、Chat Completions、图片接口及合法历史记录不受影响。

## 验证方式

- 单元测试验证 `item_*` 推理项转换为 `rs_*`，合法推理项和非推理项不变。
- 执行测试验证直接 HTTP 400 `invalid_request_error` 只请求一个账号。
- 运行相关 Rust 测试、格式检查和差异检查。
