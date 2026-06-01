# Codex 图片生成桥接

## 目标

让通过 Codex OAuth 账号转发的 OpenAI Responses 请求，在用户要求生成或编辑图片时，可以使用 OpenAI Responses 原生 `image_generation` 工具，避免模型误以为当前客户端没有图片生成能力。

## 非目标

- 不改变本机 Codex、CLI 或浏览器工具栏能力。
- 不把普通请求强制改成图片生成请求。
- 不改变用户分组、套餐、钱包和模型价格规则。
- 不改变 OpenAI Responses Compact 的请求能力。

## 行为变化

- 当请求最终走 Codex 提供商的 OpenAI Responses 端点时，系统会在上游请求中补充 `image_generation` 工具。
- 系统会在上游 `instructions` 中补一句说明：即使本地客户端没有 `image_gen` 命名空间，也可以使用 Responses 原生 `image_generation` 工具。
- 如果请求明确选择 `image_generation` 工具，仍按图片生成请求处理。
- 如果请求只是普通工具请求，且 `tool_choice` 是 `auto` 或未设置，不会强制改成图片生成请求。
- 整理图片工具参数时，只读取真正的 `type=image_generation` 工具，不会把普通函数工具误改成图片工具。

## 影响范围

- 只影响 `provider_type=codex` 且上游端点为 `openai:responses` 的请求。
- 不影响 `openai:responses:compact`。
- `openai:image` 仍走已有图片接口转换逻辑。
- 请求记录中仍保留用户原始请求，上游请求记录会体现系统补充后的工具和说明。

## 验证方式

- 单元测试覆盖 Codex Responses 普通请求自动补充图片工具和说明。
- 单元测试覆盖明确选择图片工具时不会丢失工具和 `tool_choice`。
- 单元测试覆盖普通工具列表里同时存在图片工具时不会误改成图片请求。
- 单元测试覆盖图片工具参数整理不会复制普通函数工具的 `description`、`parameters` 等字段。
