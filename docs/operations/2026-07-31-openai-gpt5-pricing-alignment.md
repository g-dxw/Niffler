# OpenAI GPT-5 价格对齐

## 目标

- 禁用 `gpt-5.5-pro` 和 `gpt-5.4-pro`，避免无价格模型继续出现在可用路由中。
- 将 GPT-5.4、GPT-5.5、GPT-5.6 系列的全局价格调整为 OpenAI 官方 API 当前价格。
- 同步 `Plus号池` 和 `Pro号池` 的 Codex 模型覆盖价格；自定义提供商价格保持不变。
- 对官方没有缓存写入价格的模型，使用显式 `0` 表示该计费维度不产生费用，不能按输入价格自动补价。

## 非目标

- 不修改自定义提供商的供应商成本或倍率。
- 不回溯修改已经完成的历史用量和结算快照。

## 行为变化

- 全局模型和同名 Codex 提供商模型不再提供 `gpt-5.5-pro`、`gpt-5.4-pro` 路由。
- GPT-5.6 Sol、Terra、Luna 使用官方短上下文和长上下文分层价格。
- GPT-5.4、GPT-5.4 Mini、GPT-5.5 的缓存写入价格使用显式 `0`，避免旧版本计费逻辑把缺失价格按输入价的 1.25 倍兜底；计费结果为零。
- `niffler_model_base_prices` 按 `effective_from_unix_ms` 追加价格版本，不修改已经生效的历史记录。

## 数据来源

- OpenAI 官方定价页：<https://developers.openai.com/api/docs/pricing>
- 单位：美元 / 1,000,000 tokens。

## 验证

- 迁移执行后查询全局模型、Codex 号池模型和价格影子表。
- 运行 `aether-billing` 相关单元测试，确认显式 `null`（兼容历史数据）不会触发默认缓存写入价。
