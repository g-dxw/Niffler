# Niffler Email Service Stability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 Niffler 的注册验证码邮件稳定可用、发送失败可见、后台配置更容易理解。

**Architecture:** 保留现有 SMTP 配置和邮件模板，不引入 Cloud Mail。把重复的 SMTP 发送代码收拢为一个邮件服务模块；注册验证码接口只负责校验、生成验证码和创建发送任务，实际发送交给后台任务执行；发送结果复用现有 `background_task_runs/background_task_events` 持久记录，并在邮件配置页展示最近发送结果。

**Tech Stack:** Rust、Axum、Tokio、现有 background task repository、Vue 3、TypeScript、现有 `asyncTasksApi`。

---

## Scope

### In Scope

- SMTP 配置继续使用现有系统配置项：`smtp_host`、`smtp_port`、`smtp_user`、`smtp_password`、`smtp_use_tls`、`smtp_use_ssl`、`smtp_from_email`、`smtp_from_name`。
- 注册验证码邮件改为后台发送，接口不再等待 SMTP 完整发送结束。
- 后台可查看最近邮件发送记录、发送状态和失败原因。
- 邮件配置页文案调整为管理员能看懂的操作流程：先保存配置，再发送测试邮件，最后开启注册邮箱验证。
- SMTP 发送代码只保留一份，注册和后台测试共用同一实现。
- 继续支持现有邮件模板：注册验证码、找回密码。

### Out of Scope

- 第一批不接 Cloud Mail。
- 第一批不做短信验证码。
- 第一批不改变用户登录、注册账号结构和邮箱唯一性规则。
- 第一批不新增新的邮件供应商概念。

## Current State

- 注册验证码接口：`POST /api/auth/send-verification-code`。
- 注册验证码逻辑：`apps/aether-gateway/src/handlers/public/support/auth_registration.rs`。
- 注册侧 SMTP 实现：`apps/aether-gateway/src/handlers/public/support/auth_email.rs`。
- 后台 SMTP 测试实现：`apps/aether-gateway/src/handlers/admin/system/shared/smtp.rs`。
- 邮件模板定义：`apps/aether-gateway/src/handlers/shared/email_templates.rs`。
- 邮件配置页：`frontend/src/views/admin/EmailSettings.vue`。
- 异步任务接口：`/api/admin/tasks`，前端封装在 `frontend/src/api/async-tasks.ts`。
- 后台任务持久表：`background_task_runs`、`background_task_events`，已经存在，不需要新建邮件日志表。

## File Structure

### New Files

- `docs/architecture/email-delivery-stability.md`
  - 记录目标、非目标、行为变化、影响范围、验证方式。
- `apps/aether-gateway/src/email/mod.rs`
  - 邮件服务模块入口，导出 SMTP、模板渲染、发送任务相关类型。
- `apps/aether-gateway/src/email/smtp.rs`
  - 唯一 SMTP 实现，负责连接、认证、发送、测试。
- `apps/aether-gateway/src/email/delivery.rs`
  - 创建邮件发送任务、执行邮件发送任务、写入任务状态和事件。
- `apps/aether-gateway/src/email/templates.rs`
  - 复用后台邮件模板，生成注册验证码邮件正文。
- `apps/aether-gateway/src/email/worker.rs`
  - 后台扫描 `trigger = auth_email` 且状态为 queued/retrying 的任务，按小批量发送。
- `frontend/src/features/admin-email/components/EmailDeliveryHistory.vue`
  - 邮件配置页里的“最近发送记录”。

### Modified Files

- `apps/aether-gateway/src/lib.rs`
  - 增加 `mod email;`。
- `apps/aether-gateway/src/task_runtime/mod.rs`
  - 增加 `TASK_KEY_AUTH_EMAIL_DELIVERY_WORKER`，用于记录后台邮件发送 worker 的启动状态。
- `apps/aether-gateway/src/state/core.rs`
  - 在 `spawn_background_tasks()` 中启动邮件发送 worker。
- `apps/aether-gateway/src/handlers/public/support/auth_email.rs`
  - 删除重复 SMTP 发送实现，只保留验证码状态读写等注册专用逻辑。
- `apps/aether-gateway/src/handlers/public/support/auth_registration.rs`
  - 发验证码时创建邮件发送任务，不再直接等待 SMTP。
- `apps/aether-gateway/src/handlers/admin/system/shared/smtp.rs`
  - 使用统一 SMTP 服务实现“测试连接”。
- `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs`
  - 增加“发送测试邮件”接口。
- `apps/aether-gateway/src/control/route/admin/system_families.rs`
  - 注册“发送测试邮件”接口路由分类。
- `apps/aether-gateway/src/control/tests/admin_core.rs`
  - 增加新后台邮件测试接口的路由分类测试。
- `apps/aether-gateway/src/control/tests/public_support.rs`
  - 增加验证码发送异步化后的路由/行为测试。
- `apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs`
  - 对邮件发送任务隐藏正文，只返回邮件类型、脱敏收件人、状态和错误。
- `frontend/src/api/auth.ts`
  - `SendVerificationCodeResponse` 增加 `delivery_id?: string`。
- `frontend/src/api/admin.ts`
  - 增加 `sendTestEmail` API。
- `frontend/src/views/admin/EmailSettings.vue`
  - 调整页面结构和文案，增加测试收件人输入和最近发送记录。

## Behavior Changes

- 发送验证码接口成功返回表示“验证码已生成，邮件发送任务已创建”，不再表示 SMTP 已经确认投递成功。
- 如果 SMTP 后续发送失败，后台发送记录显示失败原因；用户侧仍可按冷却时间重新发送验证码。
- 后台“测试连接”只验证 SMTP 连接和认证；新增“发送测试邮件”验证真实投递链路。
- 注册邮箱验证开关仍然要求 SMTP 基础配置完整后才能开启。

## Task 1: Write Architecture Record

**Files:**
- Create: `docs/architecture/email-delivery-stability.md`

- [ ] **Step 1: Add design record**

Create the document with these sections:

```markdown
# 邮件发送稳定性改造

## 目标

让注册验证码邮件稳定可用，发送失败对管理员可见，注册接口不再被 SMTP 发送耗时拖慢。

## 非目标

本次不接 Cloud Mail，不新增短信验证，不改变用户账号结构，不改变邮箱唯一性规则。

## 行为变化

发送验证码接口成功返回表示验证码已生成且邮件任务已创建；邮件是否发送成功在后台发送记录查看。

## 影响范围

影响注册验证码发送、后台 SMTP 测试、邮件配置页和后台任务记录。

## 验证方式

运行注册验证码相关 Rust 测试、后台系统接口路由测试、前端类型检查，并在本地后台完成一次 SMTP 配置保存和测试发送流程。
```

- [ ] **Step 2: Verify document exists**

Run:

```bash
test -f docs/architecture/email-delivery-stability.md
```

Expected: exit code `0`.

- [ ] **Step 3: Commit**

```bash
git add docs/architecture/email-delivery-stability.md
git commit -m "docs: record email delivery stability design"
```

## Task 2: Extract Shared SMTP Service

**Files:**
- Create: `apps/aether-gateway/src/email/mod.rs`
- Create: `apps/aether-gateway/src/email/smtp.rs`
- Modify: `apps/aether-gateway/src/lib.rs`
- Modify: `apps/aether-gateway/src/handlers/public/support/auth_email.rs`
- Modify: `apps/aether-gateway/src/handlers/admin/system/shared/smtp.rs`

- [ ] **Step 1: Create email module entry**

Add `apps/aether-gateway/src/email/mod.rs`:

```rust
mod smtp;

pub(crate) use smtp::{
    send_email_blocking, test_smtp_connection_blocking, EmailMessage, SmtpConfig, SmtpTestResult,
};
```

- [ ] **Step 2: Move SMTP types and send implementation**

Move the SMTP connection, TLS, AUTH LOGIN, MIME header encoding and send logic from `auth_email.rs` into `apps/aether-gateway/src/email/smtp.rs`.

The public types must be:

```rust
#[derive(Debug, Clone)]
pub(crate) struct SmtpConfig {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    pub(crate) use_tls: bool,
    pub(crate) use_ssl: bool,
    pub(crate) from_email: String,
    pub(crate) from_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct EmailMessage {
    pub(crate) to_email: String,
    pub(crate) subject: String,
    pub(crate) html_body: String,
    pub(crate) text_body: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct SmtpTestResult {
    pub(crate) success: bool,
    pub(crate) message: String,
}
```

Expose these functions:

```rust
pub(crate) fn send_email_blocking(
    config: SmtpConfig,
    email: EmailMessage,
) -> Result<(), GatewayError>

pub(crate) fn test_smtp_connection_blocking(
    config: SmtpConfig,
) -> SmtpTestResult
```

- [ ] **Step 3: Register module**

Add this line in `apps/aether-gateway/src/lib.rs`:

```rust
mod email;
```

- [ ] **Step 4: Update registration email code**

In `auth_email.rs`, replace local `AuthSmtpConfig` and `AuthComposedEmail` with aliases or conversions to `crate::email::SmtpConfig` and `crate::email::EmailMessage`.

- [ ] **Step 5: Update admin SMTP test code**

In `admin/system/shared/smtp.rs`, remove the duplicated SMTP networking code and call `crate::email::test_smtp_connection_blocking(config)`.

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test -p aether-gateway classifies_admin_system_maintenance_write_routes_as_admin_proxy_route classifies_auth_routes_as_public_support_route
```

Expected: both tests pass.

- [ ] **Step 7: Commit**

```bash
git add apps/aether-gateway/src/email apps/aether-gateway/src/lib.rs apps/aether-gateway/src/handlers/public/support/auth_email.rs apps/aether-gateway/src/handlers/admin/system/shared/smtp.rs
git commit -m "refactor: share smtp email sender"
```

## Task 3: Add Email Delivery Task Runtime

**Files:**
- Create: `apps/aether-gateway/src/email/delivery.rs`
- Create: `apps/aether-gateway/src/email/templates.rs`
- Create: `apps/aether-gateway/src/email/worker.rs`
- Modify: `apps/aether-gateway/src/email/mod.rs`
- Modify: `apps/aether-gateway/src/task_runtime/mod.rs`
- Modify: `apps/aether-gateway/src/state/core.rs`

- [ ] **Step 1: Add task key**

In `apps/aether-gateway/src/task_runtime/mod.rs`, add:

```rust
pub(crate) const TASK_KEY_AUTH_EMAIL_DELIVERY_WORKER: &str = "auth.email.delivery.worker";
pub(crate) const TASK_TRIGGER_AUTH_EMAIL: &str = "auth_email";
```

Add a task definition:

```rust
TaskDefinition::new(
    TASK_KEY_AUTH_EMAIL_DELIVERY_WORKER,
    TaskKind::Daemon,
    "daemon",
    true,
    true,
    RETRY_ONCE,
),
```

- [ ] **Step 2: Define delivery payload**

Create `apps/aether-gateway/src/email/delivery.rs` with:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct EmailDeliveryPayload {
    pub(crate) message_type: String,
    pub(crate) to_email: String,
    pub(crate) subject: String,
    pub(crate) html_body: String,
    pub(crate) text_body: String,
}
```

Add `queue_email_delivery(state, payload, created_by)` that writes a `background_task_runs` record:

- `id`: UUID。
- `task_key`: `auth.email.delivery:{id}`。
- `kind`: `FireAndForget`。
- `trigger`: `auth_email`。
- `status`: `Queued`。
- `payload_json`: the payload JSON。
- `result_json`: safe summary only, containing `message_type` and masked `to_email`。
- `max_attempts`: `2`。
- `progress_message`: `等待发送邮件`。

- [ ] **Step 3: Add delivery executor**

In `delivery.rs`, add `execute_email_delivery(state, run)`:

- Parse `run.payload_json` into `EmailDeliveryPayload`。
- Read SMTP config from system config。
- Mark run as `Running`。
- Call `send_email_blocking` inside `tokio::task::spawn_blocking`。
- On success, set `status = Succeeded` and `progress_message = 邮件已发送`。
- On failure and `attempt < max_attempts`, set `status = Retrying` and `error_message`。
- On final failure, set `status = Failed` and `error_message`。
- Insert one `background_task_events` record for start and one for result。

- [ ] **Step 4: Add worker**

Create `apps/aether-gateway/src/email/worker.rs`:

- Every 5 seconds list tasks with `kind = FireAndForget`、`trigger = auth_email`、`status = Queued`。
- Then list tasks with `status = Retrying`。
- Process at most 10 tasks per tick。
- Run at most 2 sends concurrently。
- Do not query request records or user tables。

- [ ] **Step 5: Spawn worker**

In `apps/aether-gateway/src/state/core.rs`, add:

```rust
supervise_worker(
    crate::task_runtime::TASK_KEY_AUTH_EMAIL_DELIVERY_WORKER,
    crate::email::spawn_auth_email_delivery_worker(self.clone()),
);
```

- [ ] **Step 6: Export module functions**

Update `apps/aether-gateway/src/email/mod.rs`:

```rust
mod delivery;
mod smtp;
mod templates;
mod worker;

pub(crate) use delivery::{execute_email_delivery, queue_email_delivery, EmailDeliveryPayload};
pub(crate) use smtp::{
    send_email_blocking, test_smtp_connection_blocking, EmailMessage, SmtpConfig, SmtpTestResult,
};
pub(crate) use templates::build_verification_email_payload;
pub(crate) use worker::spawn_auth_email_delivery_worker;
```

- [ ] **Step 7: Run compile check**

Run:

```bash
cargo check -p aether-gateway
```

Expected: completes without errors.

- [ ] **Step 8: Commit**

```bash
git add apps/aether-gateway/src/email apps/aether-gateway/src/task_runtime/mod.rs apps/aether-gateway/src/state/core.rs
git commit -m "feat: add async email delivery worker"
```

## Task 4: Queue Registration Verification Emails

**Files:**
- Modify: `apps/aether-gateway/src/handlers/public/support/auth_registration.rs`
- Modify: `apps/aether-gateway/src/handlers/public/support/auth_email.rs`
- Modify: `frontend/src/api/auth.ts`
- Modify: `frontend/src/features/auth/components/RegisterDialog.vue`
- Modify: `apps/aether-gateway/src/control/tests/public_support.rs`

- [ ] **Step 1: Change send verification response**

In `auth_registration.rs`, after storing the pending verification code, call:

```rust
let delivery_id = crate::email::queue_email_delivery(
    state,
    crate::email::build_verification_email_payload(state, &email, &code).await?,
    Some("auth:send_verification_code".to_string()),
)
.await?;
```

Return:

```json
{
  "success": true,
  "message": "验证码正在发送，请稍后查收",
  "expire_minutes": 15,
  "delivery_id": "..."
}
```

- [ ] **Step 2: Keep cooldown behavior**

Keep the existing resend cooldown behavior:

- If pending code exists and cooldown has not passed, return the existing cooldown message。
- If pending code expired, clear old pending code and create a new one。

- [ ] **Step 3: Update frontend auth type**

In `frontend/src/api/auth.ts`, update:

```ts
export interface SendVerificationCodeResponse {
  message: string
  success: boolean
  expire_minutes?: number
  delivery_id?: string
}
```

- [ ] **Step 4: Update register dialog copy**

In `RegisterDialog.vue`, after sending verification code, show:

```ts
success(response.message || '验证码正在发送，请稍后查收')
```

Do not say “发送成功” unless backend confirms delivery through a completed send task.

- [ ] **Step 5: Add test**

Add a public support test that sends a verification code with test delivery store enabled and asserts:

- Response contains `success: true`。
- Response contains `delivery_id`。
- A pending verification code exists。
- No SMTP network call is required in the request handler。

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test -p aether-gateway send_verification
```

Expected: verification-code tests pass.

- [ ] **Step 7: Commit**

```bash
git add apps/aether-gateway/src/handlers/public/support/auth_registration.rs apps/aether-gateway/src/handlers/public/support/auth_email.rs frontend/src/api/auth.ts frontend/src/features/auth/components/RegisterDialog.vue apps/aether-gateway/src/control/tests/public_support.rs
git commit -m "feat: queue registration verification emails"
```

## Task 5: Add Real Test Email and Delivery History

**Files:**
- Modify: `apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs`
- Modify: `apps/aether-gateway/src/control/route/admin/system_families.rs`
- Modify: `apps/aether-gateway/src/control/tests/admin_core.rs`
- Modify: `apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs`
- Modify: `frontend/src/api/admin.ts`
- Create: `frontend/src/features/admin-email/components/EmailDeliveryHistory.vue`
- Modify: `frontend/src/views/admin/EmailSettings.vue`

- [ ] **Step 1: Add admin test-send endpoint**

Add `POST /api/admin/system/email/test-send`:

Request:

```json
{
  "to_email": "admin@example.com"
}
```

Response:

```json
{
  "success": true,
  "message": "测试邮件正在发送",
  "delivery_id": "..."
}
```

The endpoint validates email format, composes a simple test email, queues it through `queue_email_delivery`, and does not send SMTP inline.

- [ ] **Step 2: Hide auth email payload in task APIs**

In `apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs`, when serializing runs whose `trigger == "auth_email"`:

- Return `payload: null`。
- Return `result` as the safe summary only。
- Never return `html_body`、`text_body`、`subject` or verification code。

The visible task record should only expose:

```json
{
  "message_type": "verification",
  "to_email": "a***@example.com"
}
```

- [ ] **Step 3: Register route classification**

In `system_families.rs`, classify:

```text
POST /api/admin/system/email/test-send -> email_test_send
```

Add this case to `classifies_admin_system_maintenance_write_routes_as_admin_proxy_route`.

- [ ] **Step 4: Add frontend API**

In `frontend/src/api/admin.ts`, add:

```ts
async sendTestEmail(toEmail: string): Promise<{ success: boolean; message: string; delivery_id?: string }> {
  const response = await apiClient.post('/api/admin/system/email/test-send', { to_email: toEmail })
  return response.data
}
```

- [ ] **Step 5: Add delivery history component**

Create `EmailDeliveryHistory.vue`:

- Use `asyncTasksApi.list({ trigger: 'auth_email', page_size: 10 })`。
- Show columns: 时间、类型、收件人、状态、失败原因。
- Mask recipient as `a***@example.com` in the list。
- Provide a refresh button。

- [ ] **Step 6: Update EmailSettings page**

Restructure the page into four sections:

- 邮件发送服务：SMTP 配置、保存、测试连接、测试收件人、发送测试邮件。
- 注册验证：是否要求邮箱验证、邮箱后缀限制。
- 邮件模板：保留现有模板编辑和预览。
- 最近发送记录：展示 `EmailDeliveryHistory`。

Use these button labels:

- `保存配置`
- `测试连接`
- `发送测试邮件`
- `保存验证规则`
- `保存模板`

- [ ] **Step 7: Run frontend check**

Run:

```bash
pnpm -C frontend type-check
```

Expected: no TypeScript errors.

- [ ] **Step 8: Run backend route tests**

Run:

```bash
cargo test -p aether-gateway classifies_admin_system_maintenance_write_routes_as_admin_proxy_route
```

Expected: route test passes.

- [ ] **Step 9: Commit**

```bash
git add apps/aether-gateway/src/handlers/admin/system/core/system_routes.rs apps/aether-gateway/src/control/route/admin/system_families.rs apps/aether-gateway/src/control/tests/admin_core.rs apps/aether-gateway/src/handlers/admin/features/background_tasks/routes.rs frontend/src/api/admin.ts frontend/src/features/admin-email/components/EmailDeliveryHistory.vue frontend/src/views/admin/EmailSettings.vue
git commit -m "feat: add email test send and delivery history"
```

## Task 6: Final Verification

**Files:**
- No code files unless verification finds a defect.

- [ ] **Step 1: Run backend checks**

```bash
cargo check -p aether-gateway
cargo test -p aether-gateway classifies_auth_routes_as_public_support_route
cargo test -p aether-gateway classifies_admin_system_maintenance_write_routes_as_admin_proxy_route
cargo test -p aether-gateway send_verification
```

Expected: all commands pass.

- [ ] **Step 2: Run frontend checks**

```bash
pnpm -C frontend type-check
```

Expected: no TypeScript errors.

- [ ] **Step 3: Manual local verification**

Start local backend and frontend using the project’s normal detached service method. Then verify:

- Open `/admin/email`。
- Save SMTP config with no password change and confirm password remains set。
- Click `测试连接` and see connection result。
- Enter test recipient and click `发送测试邮件`。
- Confirm “最近发送记录” appears and status changes to succeeded or failed。
- Open register dialog, request verification code, and confirm the API returns quickly。

- [ ] **Step 4: Production rollout guard**

Before production deploy:

- Confirm migrations are not required for this plan。
- Confirm background task table exists on production。
- Deploy backend first, then frontend。
- After deploy, verify `/api/admin/tasks?trigger=auth_email&page_size=10` returns normally。
- Send one test email from `/admin/email`。

- [ ] **Step 5: Commit any verification fix**

If verification requires a fix:

```bash
git add <changed-files>
git commit -m "fix: stabilize email delivery verification"
```

## Self-Review

### Spec Coverage

- SMTP 已有能力：保留。
- 注册验证码：改成任务发送。
- 发送日志：复用现有后台任务表。
- 页面易用性：调整邮件配置页流程和文案。
- Cloud Mail：明确不在第一批。
- 数据库压力：不新增大表，worker 小批量、低并发、固定间隔。

### Risk Check

- 最大风险是发送任务里保存邮件正文会包含验证码。实现时后台任务列表和详情都必须隐藏邮件正文，只展示邮件类型、脱敏收件人、状态和错误。
- 第二个风险是多后台节点重复发送。实现 worker 时必须只在 `node_role` 允许后台任务的节点启动，并在发送前把任务状态从 queued 更新为 running；如果后续发现多节点并发问题，再给 background task repository 增加原子 claim 方法。
- 第三个风险是 SMTP 失败后用户不知道。用户侧保留重新发送入口，管理员侧通过最近发送记录看失败原因。

### Pass Criteria

- 注册接口不再因为 SMTP 慢而卡 30 秒。
- 管理员能在 `/admin/email` 看见测试邮件和注册验证码邮件的发送结果。
- SMTP 实现只剩一份。
- 不新增 Cloud Mail、不新增邮件供应商配置。
- 后端相关测试和前端类型检查通过。
