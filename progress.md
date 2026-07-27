# Progress Log

## 2026-07-27 GitHub 受保护生产发布与双人审核

- 用户同意继续配置 GitHub 生产发布环境和第二名审核者。
- 已启用文件化计划，创建分支 `codex/secure-production-deploy-20260727`。
- 正在执行 Phase 1，只读审计 GitHub Actions、环境、Secrets、协作者、工作流和
  生产主机权限；尚未生成密钥、修改服务器授权或写入 GitHub Secret。
- 前一轮审查误清理的 172 个本地上游版本标签已从 `upstream` 全部恢复，
  远端仓库和代码提交未受影响。
- GitHub 审计确认仓库没有环境、Secrets、Variables、Deploy Keys 或 Runner，
  只有 `ryfineZ` 一名管理员。
- 生产主机审计确认当前入口直接登录 root，唯一授权公钥无命令限制，且没有专用
  发布用户；下一步必须先确定受限账户和受控命令协议，不能直接上传 Actions 密钥。
- 已检查 `Build App Image` 和 `deploy-ci-artifact.sh`：现有协议要求 SSH 用户查询
  Docker 状态并运行任意远程脚本，不能直接降权复用。
- 已核对 GitHub 官方环境与 Actions 安全文档：生产 Secret 应放在仅允许 `main`
  的环境中，第二名维护者作为环境审核者并禁止自行批准；生产 Job 使用的 Action
  应固定到完整提交 SHA。
- 已确认生产 `.env` 和 Compose 文件仅 root 可读，专用发布用户不需要也不应加入
  Docker 组；固定部署器已经覆盖主线、镜像 revision、迁移、健康和回退验证。
- Phase 1 完成，进入 Phase 2：先更新发布设计与运维文档，再修改脚本和工作流。
- 已先更新 `docs/operations/ci-image-deploy.md`，记录 GitHub 环境保护、专用用户、
  固定 SSH 协议、root 包装器、密钥轮换和验证要求。
- Phase 2 完成，进入 Phase 3：实现并测试受限发布链路。
- 已先新增受限 SSH、root 包装器和安装脚本的安全边界测试；首次运行按预期失败，
  原因是三个实现脚本尚不存在。
- 已实现受限 SSH 命令分派器、root 固定参数包装器和服务器安装脚本；只允许查询
  状态、上传准确提交镜像和部署该提交，不向 Actions 开放交互式 shell 或 Docker。
- 首次成功路径验证发现 macOS 的 `/var` 与 `/private/var` 真实路径差异，正在修正
  包装器的目录边界比较；权限、提交号、所有者、模式和文件大小检查保持不变。
- 路径规范化修正后，受限生产访问测试已全部通过。
- 已先新增发布客户端下载、上传哈希核对、固定命令调用和禁止回滚的测试；下一步
  实现 `deploy-ci-artifact.sh` 的受限 Actions 模式。
- `deploy-ci-artifact.sh` 已支持受限 Actions 模式：沿用成功 CI 产物和提交继承
  检查，通过固定 SSH 命令查询、上传、核对 SHA-256 后部署，并禁止自动化回滚。
- 已新增独立 `Deploy Production` 工作流：仅允许 `main`，要求输入提交等于当前
  `main`，使用 `production` 环境、主机指纹校验、串行发布和完整 SHA 固定 Action。
- 已将新脚本和测试接入 `Build App Image` 与 `Release Tooling CI`；受限入口、
  发布客户端、固定部署器、提交继承和生产工作流安全测试均已通过。
- 隔离 Ubuntu 容器首次安装验证因新增脚本尚未设置可执行位而停止，容器和生产
  均未发生配置变更；正在修正文件模式后重试。
- 新脚本已设置可执行位；隔离 Ubuntu 24.04 容器安装验证通过：用户和目录权限
  正确，固定 `status` 命令可经白名单 sudo 执行，任意 `/bin/true` sudo 被拒绝。
- 生产主机只读兼容性检查通过：Ubuntu 22.04、OpenSSH 8.9、sudo 1.9.9 和安装
  所需命令齐全，SSH 严格权限检查开启，客户端环境变量注入关闭。
- 安全复核发现并修正主机密钥信任范围：新增独立验证脚本，只将指纹精确匹配的
  扫描密钥写入 `known_hosts`；双密钥和完全不匹配回归测试均通过。
- 全部新增和修改 Shell 脚本已通过 ShellCheck，生产工作流 YAML 语法检查通过。
- 已创建本地安全发布提交；首次推送遇到 GitHub 空响应，API 确认远端没有创建
  该分支，正在保留本地提交并安全重试。

## 2026-07-23 Codex 上游配额窗口

- 已确认当前任务范围扩大为所有上游窗口，至少包含 5H、7D、1M。
- 已核对本地 sub2api 源码：上游 `/wham/usage` 返回窗口时长和重置字段，不能只按 `primary/secondary` 命名。
- 已确认 Aether 当前后端、前端、管理号池和用量统计仍存在 `weekly/5h` 硬编码。
- 已创建本阶段任务计划、调查记录和设计方向，下一步先更新设计文档再修改实现。
- 已完成后端通用窗口解析：保留上游 5H、7D、1M 和未知时长，标准代码为 `5h/weekly/1m`，未知窗口按真实秒数生成稳定代码。
- 已完成前端和管理号池通用展示，窗口消失时不再从旧快照补回；旧 `weekly` 仅作为历史兼容别名。
- 已完成账号级耗尽判断、内存和 PostgreSQL 窗口用量统计、手动重置逻辑；feature/model/workspace 窗口不计入普通请求统计。
- 已通过解析、网关快照、调度、内存统计、SQL 源码、手动重置、号池摘要和前端配额展示测试。
- 已通过 `cargo fmt --all -- --check`、`git diff --check`、四个 Rust 包的 `cargo check` 和前端 `vue-tsc --noEmit`。
- 已将未知窗口的用户展示改为中文天/小时/分钟，不再显示 `s`、`m` 或内部窗口代码；保留原始秒数用于准确换算。
- 已确认 7D 沿用 `weekly` 内部标识，界面继续显示 `7D`，避免同义代码导致当前周期用量丢失。
- 审查确认同步生图失败事件仍可能按 HTTP 200 返回、`error: null` 会被误判、未知窗口统计会截断秒数；进入修复和针对性验证。
- 已修复同步生图失败归一化后的成功判断，账号状态和成功副作用不再继续使用上游原始 HTTP 200。
- 已统一同步生图事件流分块逻辑，LF 和 CRLF 换行都能识别失败事件。
- 已为 PostgreSQL Codex 窗口增量统计增加 `BIGINT` 数值和分钟转秒溢出保护，异常快照不会中断整批用量处理。
- 最终验证通过：新增生图失败与 CRLF 测试、Codex 窗口 SQL 测试、相关四个 Rust 包编译、Rust 格式检查和差异空白检查。

## 2026-07-15 本地未提交改动审查与上线

- 已清点 24 个原始未提交文件：18 个已修改文件、6 个未跟踪文件，暂存区为空。
- 已读取项目现有计划、调查和进度记录，并确认旧会话恢复内容不适用于当前工作区。
- 已确认提交链为 `153046f0 -> fcc5a165 -> eb44f662`；本次最终提交必须位于 `eb44f662` 之后。
- 正在逐项审查 Codex 客户端版本管理实现、生图验收脚本、相关文档和测试。
- 初始检查通过：`aether-model-fetch` 41 项测试、网关模型获取 18 项测试、Python 验证脚本 3 项测试、Rust 格式和差异空白检查。
- 审查发现空模型目录会清空现有权限、损坏版本状态会停止自动检查两项问题；已先更新客户端版本管理设计记录。
- 已补充空目录回归测试并确认修改前失败；已实现空目录失败保护和损坏版本状态恢复逻辑。
- 修复后验证通过：网关模型获取 19 项测试、模型获取库 41 项测试、Python 验证脚本 3 项测试、严格 Clippy、Rust 格式和差异空白检查。
- 已完成全部未提交文件的代码、文档、安全信息和提交链复核，未发现遗留密钥或凭证。
- 已创建提交 `44354c0e fix(codex): 自动更新上游模型目录版本`，并接到 `eb44f662` 之后；`fcc5a165` 图片保活和 `eb44f662` sub2api 导入功能均保留。
- 整合后验证通过：严格 Clippy、Rust 格式、模型获取库 41 项、网关模型获取 20 项、Provider 查询 89 项、前端类型检查、sub2api 前端 6 项、Python 验证脚本 3 项和差异空白检查。
- 已创建审查记录提交 `4423c0cf docs(plan): 记录模型目录审查结果`，并将分支 `codex/fix-codex-lite-tools` 推送至 `origin`。
- GitHub Actions `Build App Image` 运行 `29356085297` 构建成功，生产镜像对应提交 `4423c0cff7ef92b8c7b824a00406c8a3cbbd1a1a`。
- 已通过现有 CI 产物部署流程更新 hd0526 的 `frontdoor` 和 `background`，两个服务首次和第二次检查均为 `healthy`，公开健康接口返回 `status: ok`。
- 生产启动后使用真实 Codex 账号检查版本，客户端版本从 `0.144.1` 自动更新为 `0.144.4`，数据库中的下一次检查时间已正常写入。
- 已强制刷新 Plus、Pro、Team 各一个有效账号的上游模型目录；三类账号均返回 `gpt-5.6-luna`、`gpt-5.6-sol`、`gpt-5.6-terra`，模型数分别为 7、10、7。
- 第二次健康检查时，两个容器已连续健康运行 9 分钟；最近 15 分钟未发现 `panic`、`fatal` 或 `error` 日志。

---

## 2026-07-14 sub2api JSON 导入兼容性

- 已收到 sub2api 导出 JSON 和 Niffler 前端“无法解析输入内容”截图。
- 已确认文件是有效 UTF-8 JSON，顶层为 sub2api 批量导出对象，包含 12 个 OAuth 账号。
- 已定位前端报错位置，正在检查 `parseImportText` 接受的具体结构及后端批量导入协议。
- 已确认前端和后端都缺少 sub2api 顶层 `accounts` 包装对象适配。
- 已核对 sub2api 源码，该 JSON 是其正式导出协议，不是损坏文件。
- 已确认 12 条凭证均可映射到 Niffler 现有 Codex Access Token 导入模型。
- 已完成根因、修复边界、字段处理和测试范围梳理；本轮未修改业务代码或生产数据。
- 已再次复核重复检测、记录命名和临时令牌生命周期，发现并修正了“外层名称映射为 account_name”这一遗漏。
- 用户确认号池名称使用“邮箱 · 空间名称”，缺少空间名称时使用“邮箱 · 空间 ID”，不使用 sub2api 外层名称，也不添加 `codex_` 前缀。
- 已在独立分支 `codex/fix-sub2api-import` 新增兼容性设计记录，准备补充失败测试。
- 已完成前端格式识别、后端 `accounts[]` 展开、嵌套凭证映射、逐条失败统计和记录名称生成。
- 已补充 sub2api 正常导入、无效账号、缺少 `accounts`、前端批量分流和网关持久化测试。
- 用户提供的真实文件结构核对通过：12 条记录均为 OpenAI OAuth，均包含访问令牌、邮箱、用户 ID 和空间 ID；按新规则可生成 12 个唯一名称，且没有 `codex_` 前缀。
- 验证通过：前端 5 项组件测试、前端类型检查、后端 12 项 sub2api 相关测试、11 项批量解析测试、2 项既有 OAuth 批量导入回归测试、Rust 格式和差异检查。
- UI 静态审查命中该弹窗原有的交互入口密度和配色规则；本次没有修改可见布局、样式或文案，因此未扩大范围处理这些既有问题。
- 代码审查发现并修复两个边界问题：sub2api 识别不再仅凭单个元数据字段；缺少邮箱和用户 ID 的条目不再允许仅凭空间 ID 创建记录。
- 新增旧单账号 JSON 回归测试和空间 ID 单独导入拒绝测试；前端 6 项测试、类型检查、后端 13 项解析测试、13 项 sub2api 相关测试及既有 OAuth 批量导入测试均通过。

---

## 2026-07-14 线上生图超时

- 已确认客户端生图工具正常加载。
- 已检查 hd0526 的生产 frontdoor/background 日志。
- 已确认多个 Pro 账号在 `gpt-image-2` 链路上长时间等待上游响应。
- 正在核对请求记录、出站配置和超时后的账号切换行为。
- 已查询 rn01 生产库的 `request_candidates`：1 次成功、30 次在约 124.6 秒取消。
- 已确认大多数请求已收到上游响应头和持续 `keepalive`，问题在 Niffler 没有向下游输出任何字节。
- 已确认图片端点未使用独立代理，基础 HTTPS/HTTP2 连通正常，未鉴权请求 0.12 秒返回 401。
- 已完成代码核对：旧心跳实现被固定关闭，与 Cloudflare 120 秒超时共同导致本次失败。
- 已确认完整账号重试流程外层已有同步图片保活实现，生产配置已改为开启。
- 已将缺省配置和配置读取失败时的行为改为默认开启，并同步后台默认值、文案和架构文档。
- 已通过 7 项同步生图保活测试、Rust 格式检查、前端类型检查和差异检查。
- 已提交并推送 `fcc5a165 fix(image): 默认开启同步生图保活`。
- GitHub Actions 生产镜像构建成功，已部署 hd0526 的 frontdoor 和 background。
- 两次健康检查均通过；部署后真实生图返回有效 PNG，首字节 0.87 秒，总耗时 21.56 秒。

---

# 原任务：后台钱包检索、用户时间展示与前端表格体验优化

## 2026-06-12

### Phase 1: 上线前复核

- 已复核钱包检索和用户创建时间展示的代码改动。
- 已运行 `npm --prefix frontend run type-check`，结果通过。
- 已运行 `cargo check -p aether-gateway`，结果通过。
- 已运行 `git diff --check`，结果通过。
- 已按 UI workflow 读取 Web 后台重构规则，后续前端优化按现有设计系统和工具型后台处理。
- 首次全量前端测试失败，根因为 Node 25 的实验性 localStorage 对象缺少标准 Storage 方法。
- 已新增 `frontend/src/test/setup.ts` 并接入 `frontend/vitest.config.ts`。
- 已补充 SQLite 钱包用户搜索回归测试。
- 已确认失败的 3 个前端测试文件重新通过。
- 已确认全量前端测试重新通过。

## Test Results

| Test | Result |
|------|--------|
| `npm --prefix frontend run type-check` | 通过 |
| `cargo check -p aether-gateway` | 通过 |
| `git diff --check` | 通过 |
| `npm --prefix frontend run build` | 通过 |
| `npm --prefix frontend run test:run -- src/api/__tests__/client.spec.ts src/utils/__tests__/crossTabRefresh.spec.ts src/stores/__tests__/auth.spec.ts` | 通过，7 个测试通过 |
| `npm --prefix frontend run test:run` | 通过，82 个文件、415 个测试通过 |
| `cargo test -p aether-gateway public_support` | 通过，176 个测试通过 |
| `cargo test -p aether-gateway wallet` | 通过，75 个测试通过 |
| `cargo test -p aether-data sqlite_wallet_read_repository_reads_wallet_contract_views` | 通过，1 个测试通过 |

## Error Log

| Error | Resolution |
|-------|------------|
| `ui_workflow.py` 第一次运行缺少必填参数 | 补充 `--platform web`、`--surface`、`--goal` 等参数后重新运行成功 |
| 全量前端测试中 localStorage 方法不存在 | 在 Vitest setup 中安装标准内存 Storage 后通过 |
| SQLite 钱包搜索回归测试缺少导入 | 补充测试模块 import 后通过 |
