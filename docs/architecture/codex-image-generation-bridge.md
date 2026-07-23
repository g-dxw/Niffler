# Codex 图片生成桥接

## 目标

让通过 Codex / ChatGPT OAuth 账号转发的普通 Responses 请求具备托管图片生成能力。模型根据完整语义自主决定是否调用 `image_generation`，Niffler 不再通过自然语言关键词判断生图意图；图片结果按 Responses 原生 `image_generation_call` 返回。

该服务端桥接只保证 Responses API 的协议结果，不负责让 Codex App 加载本机 `image_gen`。Codex App 是否提供本地生图工具由客户端在发出请求前决定；当前稳定使用方式仍需 Provider 配置携带非空的 `X-OpenAI-Actor-Authorization`。后续由 Niffler 定制版 CC Switch 自动写入该配置，用户无需手工编辑 TOML。

## 非目标

- 不改变本机 Codex、CLI 或浏览器工具栏能力。
- 不通过“生成、图片、画”等自然语言词语决定是否生图。
- 不改变用户分组、套餐、钱包和模型价格规则。
- 不改变 `openai:responses:compact`。
- 不移除或改写客户端已经声明的 function、custom、namespace 和客户端执行的工具搜索。
- 不为未启用此能力的第三方 OpenAI 兼容端点注入图片工具。
- 不声称服务端响应能够远程启用客户端本地 `image_gen`。
- 不把图片 Base64 转换成 Markdown、普通助手文本或其他会进入后续文本上下文的内容。

## 行为变化

- Codex / ChatGPT OAuth 的普通 `openai:responses` 请求默认补充原生 `image_generation` 工具；没有设置 `tool_choice` 时使用 `auto`。
- 模型读取完整会话并按语义选择工具。Niffler 不读取最后一条用户消息做关键词匹配，也不把自然语言请求提前改路由到 `openai:image`。
- 顶层模型保持用户选择的 Responses 模型，图片工具模型默认使用 `gpt-image-2`。
- 请求含原生 `image_generation` 工具时，移除 `X-OpenAI-Internal-Codex-Responses-Lite` 及请求体内对应的 Lite 镜像标记。Lite 端点不接受该托管工具；完整 Responses 端点已验证 `gpt-5.6-sol` 和 `gpt-5.6-terra` 都能执行图片工具。
- 普通 Codex Responses 请求若携带布尔型 Lite 镜像标记，网关先将其规范为上游接受的字符串 `"true"` 或 `"false"`，不直接传递布尔值。
- 没有图片工具的 `gpt-5.6-sol` 请求继续保留 Lite 请求头；不支持 Lite 的模型仍按原规则移除。
- 已经声明 `image_gen` namespace、`image_gen.imagegen` 函数或同名 custom 工具时，不再补充第二套托管图片工具，避免冲突。
- Provider 或 Endpoint 可以通过 `openai_responses_image_generation_tool_enabled: false` 关闭默认图片工具；第三方兼容端点仍需显式设为 `true` 才启用。
- 顶层模型为 `gpt-image-*`、请求路径为 `openai:image`、或 `tool_choice` 明确选择 `image_generation` 时，仍可进入现有专用图片桥接链路。这些都是协议字段，不是文本匹配。
- 流处理按 SSE 事件块透传 `event:` 与 `data:`，保留原生 `image_generation_call`。如果终态 `response.output` 为空，可使用此前的 `response.output_item.done.item` 重建终态输出，但不得追加 Markdown Base64 助手消息。
- `response.output_item.done` 已携带非空图片结果时，网关将该图片项的状态规范为 `completed`，并在重建的终态 `response.output` 中保持一致，避免客户端把完整结果继续识别为生成中。
- 下一轮请求回放 `image_generation_call` 时，只保留上游接受的 `type`、`id`、`status`、`result`；移除响应展示使用的 `action`、`background`、`output_format`、`quality`、`revised_prompt`、`size` 等字段。
- 桥接指令要求模型在用户目标是栅格成品或编辑结果时必须调用托管工具；不得用提示词、外部链接、Markdown 图片或没有工具结果的“已经完成”代替。
- ChatGPT Codex OAuth 上游继续强制 `store: false`；图片预览依赖当前响应中的原生图片事件，不依赖上游存储。
- 同步生图上游即使返回 HTTP 200，只要正文包含 `response.failed`、非空 `error` 或非完成状态，Niffler 必须先将状态改为真实错误，再记录用量、更新账号状态和构造客户端响应，不能继续作为成功请求结算。标准成功响应中的 `error: null` 不算失败。
- 同步生图事件流同时接受 LF 和 CRLF 换行，避免不同上游或代理使用 Windows 风格换行时漏掉失败事件。
- 文本用量读取上游 `usage`；图片工具用量读取 `tool_usage.image_gen`。计费估算、日志和补偿统计不得把 `result`、`partial_image_b64` 等二进制字段计作文本 Token。
- 同步 Images 接口默认开启长连接心跳：等待上游 SSE 生成结果时，每 15 秒向客户端输出 JSON 合法空白，避免 Cloudflare 在 120 秒无下行数据时中断请求。
- 心跳包装覆盖完整的账号候选重试流程；单个上游失败后仍可切换账号，不会因为已开始下行响应而跳过重试。
- 心跳开启后外层 HTTP 状态固定为 200，上游错误放在标准 JSON `error` 中，并在 `error.upstream_status` 保留原状态码。管理员仍可显式关闭心跳，但经 CDN 或反向代理部署时不建议关闭。

## 影响范围

- 影响最终上游端点为 `openai:responses` 的 Codex / ChatGPT OAuth 请求。
- 影响 `/v1/images/generations`、`/v1/images/edits` 等同步 OpenAI Images 请求的等待方式；非图片接口不受影响。
- 开启配置的第三方 OpenAI 兼容 Responses 端点使用相同行为。
- 普通文本请求会多携带一个托管图片工具声明，由模型决定是否调用；这项能力可用于直接 API 调用，但不等同于 Codex App 已加载本地 `image_gen`。
- Codex App 使用本地生图工具时，客户端 Provider 配置必须携带非空的 `X-OpenAI-Actor-Authorization`。下游中转如果直接向终端用户提供 Codex App 接入，也必须通过配置模板或定制版 CC Switch 写入该请求头。
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
- 单元测试覆盖同格式 Responses 图片流原样保留图片事件，不产生 Markdown Base64 助手消息；普通文本流保持不变，失败或提前结束不伪造成功结果。
- 验证图片后的下一轮请求不会将上一张图片的 Base64 作为普通文本重新发送，输入 Token 不出现异常增长。
- 验证带完整上游图片展示字段的 `image_generation_call` 可以在下一轮归一化后继续对话。
- API 层分别验证直连 Niffler 与下游中转的托管 `image_generation` 结果；该结果不能替代 Codex App 界面验收。
- Codex App 界面验收必须使用包含 `X-OpenAI-Actor-Authorization` 的 Provider 配置，确认本地 `image_gen` 已加载、图片可以预览和继续编辑。
- 验证同步图片生成超过 120 秒时，经 Cloudflare 的连接仍持续存活，最终返回可解析图片 JSON。
- 验证心跳模式下第一个账号返回可重试错误时，网关会继续尝试下一个账号；全部失败时响应体包含 `error.upstream_status`。
