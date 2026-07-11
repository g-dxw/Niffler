# Codex 图片生成桥接

## 目标

让通过 Codex / ChatGPT OAuth 账号转发的明确生图请求走独立图片执行链路，并以 Codex App 能实际展示的 Responses 助手图片消息返回结果；普通 Responses 请求保持客户端原始工具能力。

## 非目标

- 不改变本机 Codex、CLI 或浏览器工具栏能力。
- 不把普通请求强制改成图片生成请求。
- 不改变用户分组、套餐、钱包和模型价格规则。
- 不改变 OpenAI Responses Compact 的请求能力。
- 不把 `gpt-5.4-mini` 解释成图片模型。
- 不改 Gemini 协议和 Gemini 图片转换逻辑。
- 不在所有请求上无条件删除 `X-OpenAI-Internal-Codex-Responses-Lite` 请求头；只有目标模型不支持 Lite 时才移除。
- 不移除或改写客户端已经声明的 function、custom、namespace 和客户端执行的工具搜索。

## 行为变化

- 判断位置在候选端点选择之前，只读取最后一条用户消息，并仅识别带明确生成动作和图片对象的请求。
- 普通对话请求最终走 Codex / ChatGPT OAuth 的 OpenAI Responses 端点时，不再补充托管 `image_generation` 工具。
- 明确生图请求优先选择同一模型服务中的 `openai:image` 端点，复用现有图片生成、下载和 Responses 事件转换能力。
- Codex App 当前不会把服务端直接返回的 Responses `image_generation_call` 转成可见图片。图片桥接收到最终 Base64 图片并确认 `response.completed` 后，会额外输出一条助手 `message`，内容为 Markdown `data:image/...;base64,...` 图片；保留原生图片事件用于协议兼容和计费，并把助手消息同步写入完成响应的 `output`。
- 只有收到非空最终图片结果后才生成助手图片消息；处理中事件和失败事件不会产生“生成好了”文本或空图片占位。
- Codex App / CLI 带 `X-OpenAI-Internal-Codex-Responses-Lite: true` 或 `1` 的请求不会自动补充 `image_generation` 工具，也不会注入图片工具说明；请求头和客户端工具列表保持原样。
- `gpt-5.6-sol` 已确认支持 Responses Lite，转发时保留 Lite 请求头；`gpt-5.5` 等未确认支持 Lite 的模型会移除该请求头，改走同一 Codex 上游的完整 Responses 能力。
- 客户端已经声明 `image_gen` namespace、`image_gen.imagegen` 函数或同名 custom 工具时，不再补充托管的 `image_generation` 工具，避免同一请求同时存在两套图片工具。
- 第三方 OpenAI 兼容端点默认不补充图片工具。只有管理员在 Provider 或 Endpoint 配置 `openai_responses_image_generation_tool_enabled: true` 后，普通对话才补充 `image_generation` 工具。
- 如果请求没有设置 `tool_choice`，补充图片工具时会设置为 `auto`；如果请求已经设置了 `tool_choice`，不会覆盖用户原有选择。
- 只有已经进入 `openai:image` 专用链路的请求才会构造 Responses 原生 `image_generation` 工具和说明。
- 明确图片请求统一按 CPA / Sub2API 的桥接方式处理：顶层 `model` 使用 Responses 主模型，图片模型放到 `tools[].model`，并强制 `tool_choice` 为 `image_generation`。
- 明确图片请求包括：`openai:image` 路径、顶层模型为 `gpt-image-*`、或 `tool_choice` 明确选择 `image_generation`。
- 如果请求只是普通工具请求，且 `tool_choice` 是 `auto` 或未设置，不会强制改成图片生成请求。
- 整理图片工具参数时，只读取真正的 `type=image_generation` 工具，不会把普通函数工具误改成图片工具。
- 图片桥接请求拆成两个模型角色：
  - 顶层 `model` 是 Responses 主模型，用来承载对话和调用 `image_generation` 工具，默认 `gpt-5.4-mini`。
  - `tools[].model` 是图片工具模型，用来生成图片和计费，默认 `gpt-image-2`。
- Codex 提供商可以通过 `provider.config.codex_image_generation_base_model` 指定桥接主模型；为空或非法时使用默认 `gpt-5.4-mini`。
- Chat/Responses 请求转到第三方 `openai:image` 端点时，也会生成标准 Responses 图片工具请求体，包含 `tools[].type=image_generation`、`tools[].model` 和 `tool_choice`。
- 第三方 API 如果只接入 `openai:image` 端点，需要配置对应图片模型或模型映射，才参与明确图片请求调度。

## 影响范围

- 影响最终上游端点为 `openai:responses` 的 Codex / ChatGPT OAuth 请求。
- Lite 请求保持客户端原始工具能力，避免上游因 Niffler 新增不支持的托管工具返回 400。
- CC Switch 选择 `gpt-5.5` 等非 Lite 模型时，不会再因客户端统一携带 Lite 请求头而被上游拒绝。
- 已经包含客户端图片工具的非 Lite 请求不再重复增加托管图片工具，避免工具名称冲突。
- 影响显式开启 `openai_responses_image_generation_tool_enabled` 的第三方 OpenAI 兼容 Responses 端点。
- 不影响 `openai:responses:compact`。
- `openai:image` 仍走已有图片接口转换逻辑。
- 图片调用开始后，必须收到非空图片结果和 `response.completed` 才能记录成功；提前 EOF 会返回明确失败。
- Codex App 展示不再依赖客户端把原生 `image_generation_call` 保存成 `imageGeneration` 历史项；生成结果会同时以普通助手图片消息交付。
- 第三方 `openai:image` 上游会收到真正的图片工具，而不是只有 `input` 和 `model` 的普通 Responses 请求。
- 请求记录中仍保留用户原始请求，上游请求记录会体现系统补充后的工具和说明。
- 使用记录和计费继续按用户请求的图片模型记录，例如 `gpt-image-2`；桥接主模型只作为上游执行细节保存。

## 验证方式

- 单元测试覆盖 Codex Responses 普通请求自动补充图片工具和说明。
- 单元测试覆盖 Lite 请求头的大小写、`true` 和 `1` 两种有效值。
- 单元测试覆盖 `gpt-5.6-sol` 保留 Lite 请求头，`gpt-5.5` 移除 Lite 请求头。
- 单元测试覆盖 Lite 请求不会自动补充图片工具或说明，并保留客户端允许的工具。
- 单元测试覆盖已有 `image_gen` namespace 或同名客户端函数时，不补充托管图片工具。
- 单元测试覆盖非 Lite 且没有客户端图片工具时，仍会补充现有图片工具和说明。
- 单元测试覆盖第三方 OpenAI 兼容 Responses 默认不补充图片工具。
- 单元测试覆盖第三方 OpenAI 兼容 Responses 显式开启后补充图片工具。
- 单元测试覆盖明确选择图片工具时不会丢失工具和 `tool_choice`。
- 单元测试覆盖顶层模型为 `gpt-image-*` 时，顶层模型改为桥接主模型，图片模型进入 `tools[].model`。
- 单元测试覆盖普通工具列表里同时存在图片工具时不会误改成图片请求。
- 单元测试覆盖图片工具参数整理不会复制普通函数工具的 `description`、`parameters` 等字段。
- 单元测试覆盖自定义桥接主模型时，顶层 `model` 使用自定义值，`tools[].model` 仍保留图片工具模型。
- 单元测试覆盖 Chat/Responses 转第三方 `openai:image` 时会注入图片工具和 `tool_choice`。
- 单元测试覆盖 `openai:image` 转 Codex Responses 时，最终图片结果会生成带 `data:image` Markdown 的助手 `message` 完成事件。
- 单元测试覆盖没有最终图片结果、收到失败事件或完成事件缺失时不会生成助手图片消息。
- 单元测试覆盖多图助手消息使用不冲突的 `output_index`，且完成响应的 `output` 与流事件一致。
