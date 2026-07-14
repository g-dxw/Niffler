# Codex 服务端生图桥接真实接口验收

## 目标

通过同一脚本验证本地隔离实例、rn01 隔离实例、下游中转和正式 `niffler.org` 的 Responses API 托管图片结果。脚本不携带 `X-OpenAI-Actor-Authorization`，也不在请求体声明图片工具，因此只验证服务端桥接，不验证 Codex App 是否加载本地 `image_gen`。

## 非目标

- 不使用生产客户 Key。
- 不打印或保存 API Key、OAuth Token 和图片 Base64。
- 不把 API 返回成功等同于 Codex App 预览成功；App 界面仍需单独验收。
- 不使用该脚本判断客户端是否需要 `X-OpenAI-Actor-Authorization`。

## 运行方式

```bash
export NIFFLER_TEST_BASE_URL='http://127.0.0.1:测试端口'
export NIFFLER_TEST_API_KEY='测试 Key'
python3 scripts/oneoff/verify_codex_image_bridge.py --include-follow-up
```

`NIFFLER_TEST_BASE_URL` 可以填写网关根地址或以 `/v1` 结尾的 API 地址，脚本会统一请求正确的 Responses 路径。API Key 只允许通过环境变量传入，避免出现在命令历史和进程参数中。

rn01 隔离实例使用独立 Postgres 和 Redis。数据库副本只保留鉴权、Provider、Endpoint、模型、价格和 OAuth 账号等测试所需数据，排除用量、请求明细、统计和后台任务记录；测试配置不得直接修改生产数据库。

默认测试 `gpt-5.5` 和 `gpt-5.6-sol`。需要增加模型时：

```bash
export NIFFLER_TEST_MODELS='gpt-5.5,gpt-5.6-sol,gpt-5.6-terra,gpt-5.6-luna'
```

## 通过标准

- 普通文本请求只返回 `4`，不调用图片工具。
- 包含 `image_generation` 字样但明确要求解释的请求只返回文本。
- 不含“生成、图片、图像、照片、画、生图”的正向语料返回非空原生图片结果。
- 图片项状态是 `completed`，响应包含 `response.completed`。
- SSE 没有 `data:data:` 或 `data:event:`。
- HTTP 响应明确包含 `Content-Type: text/event-stream`，不依赖客户端猜测正文格式。
- 图片文件可被解码，格式签名正确，实际尺寸为 1536×1024。
- 开启续聊检查后，回放图片项只包含 `type/id/status/result`，后续请求成功且不再次生图。

## 结果记录

只记录测试时间、网关 commit、入口地址、模型、HTTP 状态、事件类型、图片格式、图片字节数、上游用量、用户费用和使用记录编号。测试完成后删除测试 Key；图片文件可按需要保留。

网络超时或连接重置必须明确报告为失败，不输出 Python 调用栈，也不得把未收到终态的请求记录为成功。

## 2026-07-13 rn01 隔离验收

- 测试入口仅监听 rn01 `127.0.0.1:18082`，使用独立 Postgres、Redis 和手工构建的 Linux 测试镜像；生产数据库配置保持不变。
- 客户端请求没有 `X-OpenAI-Actor-Authorization`，也没有声明图片工具。
- `gpt-5.5` 返回 1536×1024 PNG，2,392,005 字节；图片请求用量为 2,415 输入、95 输出、2,510 总 Token，测试价格下用户费用为 0.00373125 美元。图片续聊输入为 478 Token。
- `gpt-5.6-sol` 返回 1536×1024 PNG，2,108,371 字节；图片请求用量为 2,522 输入、202 输出、2,724 总 Token，测试价格下用户费用为 0.0046675 美元。图片续聊输入为 478 Token。
- 两个模型都通过普通文本、包含图片术语但不要求成品的负向语料、旧关键词规则无法识别的正向语料和图片续聊检查；SSE 事件可直接解析，图片项和响应终态均为 `completed`。
- 5.5 首次图片请求选中的 OAuth 账号超过 300 秒仍未返回首字节，重试换用其他账号后通过。该账号异常不能记作桥接成功，正式发布后仍需监控单账号首字节超时和重试。
- 隔离前台节点没有启动后台任务，费用结论来自其写入独立 Redis 的完成事件和计费快照，没有执行钱包扣费。生产后台节点消费同结构事件的持久化逻辑仍由现有回归测试覆盖。
- 生产 `Pro号池` 当前明确配置 `openai_responses_image_generation_tool_enabled: false`。正式发布时必须改为 `true` 或删除该关闭值，否则该 Provider 仍不会注入图片工具。
- 本次验证只证明 Responses API 返回了原生图片项；后续 Codex App 实测未证明无请求头时会加载本地 `image_gen`，因此不能把该结果表述为客户端零配置可用。

## Codex App 单独验收

Codex App 的 Provider 配置必须包含非空的 `X-OpenAI-Actor-Authorization`。验收时检查客户端实际加载 `image_gen`，并完成图片生成、界面预览、同会话续聊和编辑。API 脚本通过但 App 没有加载工具，仍视为客户端验收失败。
