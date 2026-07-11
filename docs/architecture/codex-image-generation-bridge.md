# Codex 图片生成桥接

## 目标

让通过 Codex / ChatGPT OAuth 账号转发的普通 Responses 请求具备原生图片生成能力。模型根据完整语义自主决定是否调用 `image_generation`，Niffler 不再通过自然语言关键词判断生图意图；图片结果同时以 Codex App 能展示的助手图片消息返回。

## 非目标

- 不改变本机 Codex、CLI 或浏览器工具栏能力。
- 不通过“生成、图片、画”等自然语言词语决定是否生图。
- 不改变用户分组、套餐、钱包和模型价格规则。
- 不改变 `openai:responses:compact`。
- 不移除或改写客户端已经声明的 function、custom、namespace 和客户端执行的工具搜索。
- 不为未启用此能力的第三方 OpenAI 兼容端点注入图片工具。

## 行为变化

- Codex / ChatGPT OAuth 的普通 `openai:responses` 请求默认补充原生 `image_generation` 工具；没有设置 `tool_choice` 时使用 `auto`。
- 模型读取完整会话并按语义选择工具。Niffler 不读取最后一条用户消息做关键词匹配，也不把自然语言请求提前改路由到 `openai:image`。
- 顶层模型保持用户选择的 Responses 模型，图片工具模型默认使用 `gpt-image-2`。
- 请求含原生 `image_generation` 工具时，移除 `X-OpenAI-Internal-Codex-Responses-Lite`。Lite 端点不接受该托管工具；完整 Responses 端点已验证 `gpt-5.6-sol` 和 `gpt-5.6-terra` 都能执行图片工具。
- 没有图片工具的 `gpt-5.6-sol` 请求继续保留 Lite 请求头；不支持 Lite 的模型仍按原规则移除。
- 已经声明 `image_gen` namespace、`image_gen.imagegen` 函数或同名 custom 工具时，不再补充第二套托管图片工具，避免冲突。
- Provider 或 Endpoint 可以通过 `openai_responses_image_generation_tool_enabled: false` 关闭默认图片工具；第三方兼容端点仍需显式设为 `true` 才启用。
- 顶层模型为 `gpt-image-*`、请求路径为 `openai:image`、或 `tool_choice` 明确选择 `image_generation` 时，仍可进入现有专用图片桥接链路。这些都是协议字段，不是文本匹配。
- Codex App 当前不会稳定展示服务端原生 `image_generation_call`。收到非空图片结果并确认 `response.completed` 后，流处理会保留原生图片事件，并追加一条带 Markdown `data:image/...;base64,...` 的助手 `message`；失败或提前结束不会输出“生成好了”或空图片。

## 影响范围

- 影响最终上游端点为 `openai:responses` 的 Codex / ChatGPT OAuth 请求。
- 开启配置的第三方 OpenAI 兼容 Responses 端点使用相同行为。
- 普通文本请求会多携带一个托管图片工具声明，由模型决定是否调用。
- 带图片工具的 Sol 请求改用完整 Responses 端点；没有图片工具的 Sol 请求仍可使用 Lite。
- `openai:responses:compact`、Gemini 协议和 Gemini 图片转换逻辑不受影响。
- 图片调用开始后，必须收到非空结果和 `response.completed` 才记录成功；提前结束会明确失败。

## 验证方式

- 使用旧关键词规则无法识别的正向表达验证模型会调用图片工具，例如描述期望画面但不出现“生成、画、图片、图像、照片”等词。
- 使用包含图片相关词、但实际要求解释或编写代码的负向表达验证模型不会调用图片工具。
- 使用带已有图片的编辑请求验证模型传递 `num_last_images_to_include`。
- 分别验证 `gpt-5.6-sol` 和 `gpt-5.6-terra` 返回非空 `image_generation_call.result` 与 `response.completed`。
- 单元测试覆盖自然语言不再触发专用图片路由、协议级显式图片请求仍保留专用路由。
- 单元测试覆盖普通 Codex Responses 自动补充图片工具、已有客户端图片工具不重复补充、配置关闭时不补充。
- 单元测试覆盖带图片工具时移除 Lite 请求头、没有图片工具的 Sol 保留 Lite 请求头。
- 单元测试覆盖同格式 Responses 图片流生成可见助手图片消息，普通文本流保持不变，失败或提前结束不生成图片消息。
