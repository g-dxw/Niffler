# 流式响应头统一规则

## 目标

Niffler 对外返回流式响应时，响应头必须描述最终返回给客户端的协议，而不是盲目透传上游响应头。已知输出为 Server-Sent Events 的链路必须返回 `Content-Type: text/event-stream`，避免反向代理或客户端把 SSE 文本当普通文本、压缩包或其他类型处理。

## 非目标

- 不把所有流式响应都强制改成 SSE。
- 不改变文件下载、视频内容、图片二进制、普通 JSON 或未知 raw stream 的响应类型。
- 不改动流式响应正文内容；只修正最终客户端响应头。

## 行为变化

- 对最终输出明确是 SSE 的本地执行流，统一设置 `Content-Type: text/event-stream`。
- 对 SSE 响应删除与流式传输不一致的 `content-length` 和 `content-encoding`。
- 响应头名称按大小写不敏感处理，避免远端运行时返回 `Content-Encoding` 这类写法时漏删旧头。
- SSE 响应继续带上 `Cache-Control: no-cache, no-transform` 和 `X-Accel-Buffering: no`，防止中间层缓存、改写或缓冲。
- 管理端响应头规则先应用，平台最后再清理 SSE 必需的协议头；如果规则明确把响应标成 `text/event-stream`，也会删除 `content-length` 和 `content-encoding`。
- 是否发送网关心跳仍沿用原判断，避免因为补响应头额外改变客户看到的正文。

## 影响范围

- 影响 OpenAI Chat streaming、OpenAI Responses streaming、OpenAI Image streaming、Claude Messages streaming、Gemini streamGenerateContent 等最终输出为 SSE 的本地执行流。
- 不影响 Gemini 文件下载、OpenAI 视频内容下载等非 SSE 流。

## 验证方式

- 单元测试覆盖上游缺失或返回错误 `Content-Type` 时，SSE 链路仍对客户端返回 `text/event-stream`。
- 单元测试覆盖非 SSE 流不会被误改为 `text/event-stream`。
- 生产验证可通过 `curl -N -D -` 检查 `/v1/responses` 流式响应头和首批事件行。
