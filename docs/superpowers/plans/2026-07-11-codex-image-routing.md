# Codex Image Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 Codex App 的模型按完整语义自主选择原生图片工具，并禁止残缺图片流被记录为成功。

**Architecture:** Codex / ChatGPT OAuth 的普通 Responses 请求注入原生 `image_generation` 工具并使用 `tool_choice:auto`，由模型按语义选择；Niffler 不再匹配自然语言关键词。带图片工具的请求移除 Lite 请求头，完整 Responses 端点负责执行图片工具；流处理保留原生图片事件并追加 Codex App 可见的助手图片消息。

**Tech Stack:** Rust、Axum、OpenAI Responses SSE、Cargo tests

---

### Task 1: 图片意图与专用路由

**Files:**
- Modify: `apps/aether-gateway/src/ai_serving/planner/standard/openai/image_intent.rs`
- Test: `apps/aether-gateway/src/ai_serving/planner/standard/openai/image_intent.rs`

- [ ] **Step 1: Write the failing tests**

增加测试，要求中文和英文的明确生图请求返回 `true`，历史消息里的生图请求与解释生图方法的请求返回 `false`。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p aether-gateway image_intent -- --nocapture`

Expected: 新增的自然语言生图测试失败。

- [ ] **Step 3: Write minimal implementation**

只读取 `input` 或 `messages` 中最后一条用户消息；仅匹配“生成、画、制作”等动作和“图片、图像、照片、海报、插画”等图片对象同时出现的明确命令。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p aether-gateway image_intent -- --nocapture`

Expected: PASS。

### Task 2: 分离普通 Responses 与图片执行

**Files:**
- Modify: `crates/aether-ai-formats/src/formats/openai/responses/codex.rs`
- Modify: `apps/aether-gateway/src/ai_serving/planner/standard/normalize/responses.rs`
- Modify: `apps/aether-gateway/src/ai_serving/planner/standard/normalize/chat.rs`
- Test: `crates/aether-ai-formats/src/formats/openai/responses/codex.rs`

- [ ] **Step 1: Write the failing tests**

增加测试，要求 Codex 的普通 `openai:responses` 请求不允许自动注入托管 `image_generation`，而明确路由到 `openai:image` 时仍允许桥接图片工具。

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p aether-ai-formats codex_hosted_image -- --nocapture`

Expected: 普通 Codex Responses 仍允许图片工具，测试失败。

- [ ] **Step 3: Write minimal implementation**

让图片工具许可函数同时接收提供商类型和端点格式：Codex 普通 Responses 返回 `false`，Codex `openai:image` 返回 `true`，第三方兼容端点维持现有配置语义。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p aether-ai-formats codex_hosted_image -- --nocapture`

Expected: PASS。

### Task 3: 图片流完整性保护

**Files:**
- Modify: `apps/aether-gateway/src/execution_runtime/stream/execution.rs`
- Test: `apps/aether-gateway/src/execution_runtime/stream/execution.rs`

- [x] **Step 1: Write the failing test**

构造只包含 `response.image_generation_call.in_progress` 和 `generating` 后 EOF 的 Responses 流，要求终态为失败；包含非空图片结果和 `response.completed` 的流保持成功。

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test -p aether-gateway incomplete_image_stream -- --nocapture`

Expected: 当前 EOF 被当作成功，测试失败。

- [x] **Step 3: Write minimal implementation**

增加轻量终态跟踪器，记录图片调用是否开始、是否收到非空结果以及是否收到 `response.completed`。图片调用开始后遇到 EOF 且缺少结果或完成事件时，生成 `response.failed` 并将使用记录标记为失败。

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test -p aether-gateway incomplete_image_stream -- --nocapture`

Expected: PASS。

### Task 4: 回归与真实接口验证

**Files:**
- Modify: `docs/architecture/codex-image-generation-bridge.md`

- [ ] **Step 1: Run focused regression tests**

Run: `cargo test -p aether-ai-formats codex -- --nocapture`

Run: `cargo test -p aether-gateway image_intent -- --nocapture`

Run: `cargo test -p aether-gateway incomplete_image_stream -- --nocapture`

Expected: 全部 PASS。

- [ ] **Step 2: Run package checks**

Run: `cargo check -p aether-ai-formats -p aether-gateway`

Expected: PASS，无新增警告。

- [ ] **Step 3: Verify through Niffler**

使用测试 API Key 验证普通 5.5、5.6 Sol 文本请求仍成功，并验证明确生图请求返回含非空 `image_generation_call.result` 与 `response.completed` 的 Responses 流。

### Task 5: Codex App 可见图片消息

**Files:**
- Modify: `crates/aether-ai-formats/src/formats/openai/image/stream.rs`
- Modify: `crates/aether-ai-formats/src/formats/shared/stream_rewrite.rs`
- Test: `crates/aether-ai-formats/src/formats/shared/stream_rewrite.rs`

- [x] **Step 1: Write the failing test**

构造 `openai:image` 到 `openai:responses` 的完整图片流，要求最终输出除了原生 `image_generation_call` 外，还包含一条助手 `message` 完成事件，正文是 `data:image` Markdown。

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test -p aether-ai-formats rewrites_openai_image_stream_to_codex_responses_visible_image -- --nocapture`

Expected: 当前没有 Responses 图片展示重写模式，测试失败。

- [x] **Step 3: Write minimal implementation**

为 `openai:image -> openai:responses` 增加专用流重写模式。原生图片事件保持不变；收到非空最终图片结果时，追加 Codex 能按普通助手消息处理的 Markdown `data:image` 图片完成事件。

- [x] **Step 4: Run test to verify it passes**

Run: `cargo test -p aether-ai-formats rewrites_openai_image_stream_to_codex_responses_visible_image -- --nocapture`

Expected: PASS，并确认缺少最终结果时不会产生助手图片消息。

### Task 6: 用语义工具选择替代关键词路由

**Files:**
- Modify: `apps/aether-gateway/src/ai_serving/planner/standard/openai/image_intent.rs`
- Modify: `crates/aether-ai-formats/src/formats/openai/responses/codex.rs`
- Modify: `crates/aether-ai-formats/src/formats/shared/stream_rewrite.rs`
- Test: 对应模块单元测试与 Niffler 真实接口

- [x] **Step 1: 先做协议实验**

使用旧关键词规则无法识别的正向表达测试 `gpt-5.6-sol` 和 `gpt-5.6-terra`。两者都由模型选择原生 `image_generation` 并返回非空图片；负向的讲解和代码请求只返回文本。带现有图片的编辑请求正确传递图片数量。

- [x] **Step 2: 写失败测试**

固定四项边界：自然语言不再触发专用路由；普通 Codex Responses 允许原生图片工具；带图片工具的 Sol 请求移除 Lite；同格式 Responses 图片流追加可见助手图片消息。

- [x] **Step 3: 最小实现**

删除自然语言关键词判断，保留协议级显式图片判断；启用 OAuth Responses 原生图片工具；根据工具声明处理 Lite 请求头；扩展同格式 Responses 图片流展示转换。

- [ ] **Step 4: 回归、评审与上线验证**

运行相关单元测试和包级检查，检查变更差异，再通过 Niffler 使用无法被旧规则匹配的正向表达和容易误判的负向表达做线上验证。
