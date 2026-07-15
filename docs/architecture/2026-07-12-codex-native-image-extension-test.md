# Codex 原生生图隔离验证

> 状态：历史隔离测试记录，已由 `codex-image-generation-bridge.md` 中的服务端托管生图方案取代，不代表当前生产设计。

## 目标

在不修改生产实例和客户路由的前提下，验证 Codex App 通过 Niffler 使用原生 `image_gen` 扩展时，能够完成图片生成、本地保存和图片预览，并且后续请求不会把图片 Base64 当作文本 token 计费。

## 非目标

- 本阶段不发布正式代码，不修改生产网关容器。
- 不依赖 `store:true`；Codex OAuth Responses 通道要求 `store:false`。
- 不通过自然语言关键词判断是否生图。
- 不继续向普通 Responses 请求注入托管 `image_generation` 作为最终方案。
- 不使用 Markdown `data:image/...;base64,...` 承载图片结果。

## 目标行为

- Codex 客户端注册并执行原生 `image_gen` 工具，由模型根据完整语义选择工具。
- `image_gen` 通过当前模型提供商的 Images API 发起独立生图请求；Niffler 的 `/v1/images/generations` 负责桥接到 OAuth 图片能力。
- Codex 客户端将返回的 Base64 解码并保存到本机 `generated_images` 目录。
- 工具结果以 `FunctionCallOutput` 的 `input_image` 内容项写入会话，图片预览由 Codex App 原生渲染。
- Niffler 的缺失用量估算忽略 `input_image.image_url`、图片结果和文件、音频等二进制字段；存在上游 usage 时始终以真实 usage 为准。
- 图片工具用量和顶层模型用量分别进入计费维度，不能遗漏图片生成成本，也不能把图片字节按文本重复收费。
- `response.failed`、上游错误或不完整图片流必须记录为失败，不得按成功请求估算收费。

## 影响范围

- 隔离测试只使用临时 API 密钥、临时模型别名和临时网关，不进入 `niffler.org` 的客户路由。
- 正式方案预计影响 Codex / ChatGPT OAuth 的 Responses 生图桥接、OpenAI Images API 和缺失用量估算。
- 普通文本 Responses、其他提供商、文件下载和音频返回不改变响应内容，只调整缺失 usage 时的本地估算。

## 验收方式

1. 使用不包含旧关键词的画面描述，确认模型自主调用 `image_gen`。
2. 确认 `/v1/images/generations` 返回有效图片，Codex 保存本地 PNG，App 显示图片预览。
3. 在同一会话发送普通后续消息，确认请求仍成功，输入 token 不随 Base64 字符数增长。
4. 发送一次基于上一张图片的编辑请求，确认 `input_image` 能被读取并返回新图片。
5. 核对 Niffler usage：顶层模型 token、缓存 token、图片工具用量、实际成本和用户费用均可解释。
6. 构造缺失 usage 的请求，确认 Base64、文件和音频字段不进入文本 token 估算。
7. 构造 `response.failed` 和不完整图片流，确认状态为失败且不产生错误的成功估算费用。
8. 删除临时配置并确认生产密钥、模型和路由未发生变化。
