# Usage 缺失上游用量时的计费保护

## 目标

当上游请求已经成功并可能产生上游成本，但响应里没有返回可解析的 usage 时，Niffler 不能继续把请求按 0 token、0 费用静默结算。

## 非目标

- 不改变上游请求成功与失败的判定。
- 不保存更多请求或响应原文。
- 不追溯修改历史账单。

## 行为变化

- 对普通文本生成类请求，若终态为成功且没有解析到上游 usage，则使用执行时内存中的请求体估算输入 token。
- 若响应体可见，则同时估算输出 token；若响应体不可见，只估算输入 token。
- 估算产生的用量会参与现有价格规则和钱包结算。
- 审计元数据增加 `usage_estimated_due_to_missing_upstream_usage=true`，便于后续筛选和复盘。
- 若请求体也不可用，则仍保持 0 token，但不额外保存原文。

## 影响范围

- 影响 Claude Messages、OpenAI Chat/Responses、Gemini Generate Content 等普通文本生成请求。
- 图片、嵌入、重排、视频、文件、取消请求、失败请求沿用原有逻辑。
- 原有能解析到上游 usage 的请求不受影响。

## 验证方式

- 增加单元测试覆盖 Claude 流式成功但没有 usage、没有保存响应体时，仍按请求体估算输入 token。
- 增加单元测试覆盖嵌入接口成功但没有 usage 时，不套用文本生成估算。
- 运行 `cargo test -p aether-usage-runtime completed_text_usage_estimates_request_tokens_when_provider_usage_is_missing`。
