# Findings

## 2026-07-15 本地未提交改动审查与上线

- 当前分支 `codex/fix-codex-lite-tools` 停在 `153046f0`，生产后续提交 `fcc5a165`（同步生图保活）和 `eb44f662`（sub2api OAuth 导入）与其构成线性提交链。
- 如果直接从当前分支构建并部署，会回退上述两个已上线功能；本次提交必须接到 `eb44f662` 之后再进入生产构建。
- 当前未提交业务代码主要实现 Codex 模型目录客户端版本自动更新；生图部分只有架构文档、验收文档和一次性验证脚本，没有新的生产生图代码。
- `planning-with-files` 恢复脚本返回的是旧会话描述，与当前 `git status` 不一致；本次审查只采用当前工作区、提交历史和生产状态作为依据。
- 审查发现上游返回 HTTP 200 和空模型数组时，`has_success=true` 会进入成功持久化并将 `allowed_models` 设为空；这与设计中“同步失败不清空权限”不一致，必须将空目录视为失败。
- 审查发现持久化的客户端版本状态无法解析时，版本刷新函数直接返回；损坏记录会使自动发现永久停止，应告警后使用进程内版本或内置已验证版本继续检查，并在成功后覆盖损坏记录。
- 手动刷新和后台同步已经通过 `ModelFetchTransportRuntime::resolve_codex_model_fetch_client_version` 使用同一有效版本，未发现两条路径版本不一致。

---

## 2026-07-14 sub2api JSON 导入兼容性

- 截图显示错误发生在点击“导入”后，提示为“无法解析输入内容，请检查格式”；需要继续确认是前端 `JSON.parse` 失败，还是业务格式适配失败被统一映射成该提示。
- 文件本身是有效 UTF-8 JSON，无 BOM，约 18 KB，不存在 JSON 语法或编码错误。
- 顶层为对象，字段为 `exported_at`、`proxies`、`accounts`；`accounts` 包含 12 条记录。
- 每条账号记录包含 `name`、`type`、`platform`、`priority`、`concurrency`、`credentials`、`extra` 等字段，凭证位于嵌套的 `credentials.access_token`。
- 前端错误文案来自 `OAuthAccountDialog.vue`，由 `parseImportText(inputText)` 返回空结果时统一显示，说明报错不等于 `JSON.parse` 语法失败。
- 前端 `isBatchImport` 只把顶层数组或多行输入识别为批量导入；任何可解析的顶层对象都强制走单条导入。
- 单条导入只读取顶层的 `access_token` / `refresh_token`，不会读取 `accounts[].credentials.access_token`，因此当前文件必然解析失败。
- 后端批量解析同样只支持顶层数组、单账号对象或逐行 Token/JSON；顶层 `{ accounts: [...] }` 会被当成单账号对象，解析为 0 条。
- 文件中的 12 条记录都有 `credentials.access_token`，并携带 `chatgpt_account_id`、`chatgpt_user_id`、`email`、`plan_type=team` 等可映射字段；现有 Niffler 批量导入结构本身可以承载这些数据，只缺少 sub2api 包装结构展开和嵌套字段映射。
- sub2api 将该结构定义为正式的 `AdminDataPayload`：顶层必须包含 `proxies` 与 `accounts`，账号凭证固定放在 `credentials` 对象中；不是临时或异常文件格式。
- sub2api 当前导入界面也以此结构做类型和版本校验，并支持多个此类文件合并，因此 Niffler 应按明确格式适配，而不是让用户手工改成数组。
- 现有 Niffler 后端已经支持仅有 Access Token 的 Codex 临时账号，并有相应测试；本文件没有 Refresh Token 不构成导入阻塞。
- 文件内 12 个 Access Token 都不是三段式 JWT，而是 `personalAccessToken` 模式；适配时不能依赖 JWT 解码补齐账号身份，必须读取 `credentials` 中的账号 ID、用户 ID、邮箱和套餐字段。
- Niffler 只要将嵌套令牌明确映射为 `access_token`，现有执行层就会按 Access Token 临时账号导入，不会误走 Refresh Token 交换。
- 推荐同时修复前后端：前端负责识别 sub2api 包装对象并走批量任务；后端负责展开 `accounts[]`、校验账号平台并做字段映射，避免只有网页入口可用。
- 本功能应限定为“授权凭证导入”：账号名、身份信息和套餐可以保留；sub2api 的代理、并发、优先级、倍率和自动暂停字段与 Niffler 语义不同，不应在这个入口静默套用。
- 无效或平台不匹配的账号应计入失败结果并显示原因，不能像当前 `filter_map` 一样静默消失；日志和错误样本不得包含令牌正文。

### 方案复核补充

- 前一版将 sub2api 外层 `name` 映射为 `account_name` 不够准确：Niffler 新建号池记录时优先使用邮箱命名，无法保留 sub2api 的 12 个自定义名称。应给批量导入条目增加独立的记录名称字段，并仅在创建新记录时使用；替换既有记录时保留原名称。
- 文件中 12 个账号名称、邮箱和用户 ID 均唯一；`chatgpt_account_id` 相同，符合 Team 工作区多个成员的结构。现有重复检测会组合账号 ID 与用户 ID，不会把这 12 条误判为同一个账号。
- 重复导入不会产生重复记录：活动中的同账号会返回“已存在”错误；失效、停用或过期记录才会被替换。该行为应在导入结果中明确显示，不应改成无条件覆盖。
- `access_token_import_temporary=true` 会阻止自动刷新；这些 Personal Access Token 没有过期字段，Niffler 会持续使用到上游拒绝，再由健康状态标记异常。这是凭证自身限制，需要在结果中提示“不可自动刷新”。
- `chatgpt_account_is_fedramp`、`openai_auth_mode`、`model_mapping` 和 WebSocket 模式目前没有对应的 Niffler 账号级语义，不应伪造映射。

---

## 2026-07-14 线上生图超时

- 截图中出现“已加载工具”和“生成服务超时”，说明不是 Codex App 未加载生图工具。
- 生产日志确认请求通过鉴权后进入 `/v1/images/generations`，路由到 `Pro号池` / `gpt-image-2`。
- 多个不同 Pro 账号连续出现超时，排除单个账号失效。
- Niffler 当前图片同步请求总超时为 900000ms（15 分钟）；客户端约 2 分钟后取消并重试，因此整个任务可持续十几分钟。
- 04:59 曾有同一图片链路在 77.2 秒返回 HTTP 200；随后多个图片请求开始长时间无响应。
- 数据库统计：近 6 小时该图片端点 1 次成功、30 次取消；30 次取消都在 124.2–124.7 秒，状态码 499。
- 其中大多数取消请求的上游首响应为 0.3–2.8 秒，已接收 8–9 个 SSE 事件，最后事件是 `keepalive`；上游一直在保持连接。
- Niffler 为了最终转换成 OpenAI Images JSON，把上游 SSE 全部缓存，没有把 `keepalive` 发给 Cloudflare/Codex App。
- Cloudflare 官方当前默认 Proxy Read Timeout 为 120 秒；线上请求经 Cloudflare 代理，实际 499 时间与该限制一致。
- 代码中已存在每 15 秒输出换行符的 JSON 空白心跳实现，但开关函数自 2026-05-10 的合并提交起固定为 `false`；线上记录也确认 `downstream_heartbeat_count=0`。
- 单纯打开旧心跳仍需要验证错误状态和账号切换：该包装层会先固定返回 HTTP 200，且内层返回 `Ok(None)` 时不能直接交还外层调度循环。
- 最终采用的是完整账号候选流程外层的保活包装，不是旧的单账号内层开关；因此保留失败后的账号切换能力。
- 每 15 秒写出的内容是 JSON 合法空白，最终响应仍可被标准 JSON 解析并用于 Codex App 图片预览。
- 生产冒烟请求的响应开头实际包含 2 个空白字节，随后是合法 JSON；图片 Base64 解码后的文件头为标准 PNG。

---

# 原任务：后台钱包检索、用户时间展示与前端表格体验优化

## 上线前复核

- 钱包管理需要按用户名检索的问题，代码层已经扩展为 `user_search` 查询参数。
- 后台接口已经覆盖：
  - 钱包列表 `/api/admin/wallets`
  - 资金流水 `/api/admin/wallets/ledger`
  - 退款审批 `/api/admin/wallets/refund-requests`
  - 充值订单 `/api/admin/payments/orders`
- Postgres 查询匹配用户名、邮箱、用户 ID；涉及独立密钥钱包时也匹配密钥名称和密钥 ID。
- SQLite、MySQL 测试仓库、内存仓库的查询结构已同步，避免不同环境字段不一致。
- 用户管理列表和详情的创建时间已经改为 `formatDateTime(user.created_at)`。
- 已新增设计记录 `docs/architecture/admin-wallet-user-search-and-created-time.md`。
- 新增 SQLite 回归测试会验证钱包列表、资金流水、退款审批、充值订单都能按 Alice / 邮箱搜索到。

## 测试环境问题

- Node 25 会提供实验性的 `globalThis.localStorage`，但未配置 `--localstorage-file` 时对象缺少 `getItem/clear/setItem/removeItem`。
- Vitest 在 jsdom 环境下没有覆盖这个不完整对象，导致依赖 localStorage 的测试失败。
- 修复点放在 `frontend/src/test/setup.ts`，只影响测试环境，不进入生产构建。

## UI Workflow

- 平台：Web / 浏览器页面。
- 界面类型：Niffler 后台管理页面。
- 主任务：管理员高效筛选、查看和操作后台数据。
- 视觉方向：沿用现有后台工具型设计，不引入新风格。
- 重做级别：medium，优先优化结构、表格和响应式，不重做业务功能。

## 待确认

- 本地工作区有大量前序任务改动，发布前需要确认哪些已经在生产，哪些需要随本次上线。
- 全局前端表格优化需要先找到现有表格组件和后台高频页面，避免逐页硬改。
