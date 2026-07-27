# Task Plan: 当前任务记录

## GitHub 受保护生产发布与双人审核（2026-07-27）

### Goal

为 Niffler 建立最小权限的 GitHub `production` 发布环境，使受保护的 `main`
能够调用生产主机上的固定发布入口；增加第二名维护者后，将 `main` 最低批准数
从 0 调整为 1。

### Current Phase

Phase 3：实现并测试受限 SSH 协议、生产工作流和服务器安装流程。

### Phases

- [x] Phase 1：完成 GitHub 与生产主机只读安全审计，确认缺失项和权限边界
- [x] Phase 2：先更新发布设计与运维文档，明确凭证轮换、审批和失败处理
- [ ] Phase 3：实现 GitHub `production` 环境、专用发布凭证和固定发布入口
- [ ] Phase 4：增加第二名维护者，将 `main` 最低批准数调整为 1
- [ ] Phase 5：执行工作流、安全、生产只读和最终权限审计

### Initial Constraints

- 不上传或复用个人 SSH 私钥。
- 不让拉取请求代码直接接触生产凭证。
- 不允许待发布分支替换生产主机固定部署器。
- 不创建无法运行或无法验证的发布工作流。
- 第二名维护者必须由用户提供准确 GitHub 用户名，不能代为猜测。

### Status

in_progress

### Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| 审查 `origin` 时错误使用标签 refspec 与 `--prune`，本地 172 个上游版本标签被清理 | 同步全部 `origin` 引用 | 远端和代码未受影响；立即从 `fawney19/Aether` 重新获取标签，核对上游 172 个、本地匹配 172 个 |
| 首次写入新计划时使用了过期的 `findings.md` 顶部标题作为补丁上下文 | 同时更新三份计划文件 | 读取三个文件当前开头后，改用稳定的一级标题作为插入位置 |
| 新增生产访问测试首次运行提示三个实现脚本不存在 | 先验证安全边界测试确实能阻止缺失实现 | 这是测试先行的预期失败；开始实现受限 SSH、root 包装器和安装脚本 |
| 受限包装器首次成功路径测试将合法上传文件误判为越界 | 使用 `realpath` 直接比较临时目录的逻辑路径 | macOS 会将 `/var` 解析为 `/private/var`；改为分别解析上传目录和文件后比较固定文件名 |
| 隔离容器首次执行安装脚本时提示解释器无权限 | 直接执行尚未设置可执行位的新脚本 | 容器和生产均未修改；为新增运维脚本与测试设置可执行位后重新验证 |
| ShellCheck 官方容器首次调用将脚本路径误当成容器可执行文件 | 沿用普通镜像的参数方式调用带入口点的镜像 | 镜像已安全下载，工作区未修改；显式覆盖入口点后重新运行 |
| ShellCheck 报告测试变量声明警告，并提示已校验提交在 SSH 客户端展开 | 静态检查全部新增和修改脚本 | 分离测试变量声明；提交号展开是固定协议所需且已严格校验，增加局部说明；字面量测试改用命名变量 |
| 生产工作流会在一个扫描指纹匹配后保存同次扫描返回的全部主机密钥 | 逐项复核主机密钥验证与 `known_hosts` 写入关系 | 先新增双密钥回归测试；改为只保存指纹精确匹配的主机密钥 |
| 提交前同步 `origin/main` 遇到 GitHub HTTP/2 framing 错误 | 执行普通 `git fetch origin main` | 本地引用未改变；改用 GitHub API 核对远端准确提交，必要时再以 HTTP/1.1 重试 |
| 首次以 HTTP/1.1 推送安全发布分支收到 GitHub 空响应 | 推送本地安全发布提交 | 先用 GitHub API 检查远端是否实际收到分支；未收到再安全重试 |
| 重试推送时 HTTPS 因 OAuth 令牌缺少 `workflow` 权限被拒绝 | 临时增加 SSH URL 并推送现有分支 | 同一命令的 SSH 目标成功创建远端分支；API 已核对远端与本地提交 SHA 完全一致 |
| PR 首轮 `Release tooling` 在 Linux 上将 `stat -f` 文件系统信息当成文件大小 | 运行受限上传跨平台测试 | GNU 与 macOS 的 `stat` 参数语义不同；所有属性读取改为先尝试 GNU `-c`，再回退 macOS `-f` |
| 最小 Ubuntu 容器缺少测试中写死的 `shasum` | 在 Linux 容器复验跨平台修复 | 测试与生产实现保持一致，先使用 `sha256sum`，仅在不可用时回退 `shasum` |
| Ubuntu 容器以 root 运行单元测试时触发生产模式并查找 `niffler-deploy` | 直接使用容器默认用户复验 GitHub Runner 行为 | 保留 root 无法启用测试覆盖的安全边界；改用容器内普通用户运行测试 |
| 生产专用密钥首次登录被 OpenSSH 拒绝 | 安装器将 `.ssh` 设为 root `0700`、授权文件设为 root `0600` | 临时私钥已删除且 Secrets 未写入；先记录 OpenSSH 读取要求，再改为 root 所有、用户只读不可写并执行真实 sshd 验证 |

## Codex 上游配额窗口与真实展示（2026-07-23）

### Goal

让 Codex 账号配额显示完全以当前上游返回为准，覆盖 5H、7D、1M 以及未来出现的其他窗口；不再把窗口位置强行解释成“周”和“5H”，也不因上游暂时不返回某个窗口而沿用旧数据。

### Current Phase

已完成：通用窗口解析、快照展示、调度判断、用量统计和测试验证

### Phases

- [x] Phase 1：确认上游 `/wham/usage` 与响应头的窗口字段、现有本地持久化和历史兼容边界
- [x] Phase 2：先更新设计记录，再实现后端通用窗口解析和快照汇总
- [x] Phase 3：移除管理端号池和前端对 `weekly/5h` 的展示硬编码
- [x] Phase 4：更新耗尽判断、窗口用量统计和重置处理，支持新增/消失窗口
- [x] Phase 5：补充 5H、7D、1M、单窗口、窗口消失和未知窗口测试
- [x] Phase 6：运行 Rust/前端相关验证，复核与当前未提交图片改动的边界
- [x] Phase 7：修复最终审查发现的失败状态判断、CRLF 事件流和 SQL 数值溢出问题

### Implementation Status

complete

### Decisions

| Decision | Rationale |
|----------|-----------|
| 窗口事实以 `limit_window_seconds`、`window_minutes`、`reset_at` 和 `used_percent` 为准 | 窗口位置 `primary/secondary` 不是业务名称，不能据此推断周/月 |
| 标准窗口代码使用 `5h`、`weekly`、`1m`；其它窗口使用基于真实时长的稳定代码 | 7D 沿用已有标识，避免同义代码和历史用量迁移；界面仍显示 `7D` |
| 额度窗口消失就从当前快照移除，不从历史快照补回 | 防止过期额度被误显示为当前额度 |

### Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| planning-with-files 恢复脚本路径不存在 | 调用旧 Claude 插件路径 | 以项目现有 `task_plan.md`、`findings.md`、`progress.md` 和工作区为准继续 |

## 本地未提交改动审查与上线（2026-07-15）

### Goal

完整审查当前工作区 24 个未提交文件，修复审查发现的问题，验证 Codex 模型目录版本管理、生图验收脚本和相关文档后，提交、推送并按现有生产流程部署。

### Current Phase

Review Phase 6

### Phases

- [x] Review Phase 1：核对项目规则、变更范围和已上线内容，识别重复或过期改动
- [x] Review Phase 2：逐项审查模型版本管理、测试脚本、文档和任务记录
- [x] Review Phase 3：先更新设计记录，再修复审查问题并补充测试
- [x] Review Phase 4：运行格式、静态检查和单元测试；真实上游模型目录验证安排在生产部署后执行
- [x] Review Phase 5：按文件清单暂存并创建约定式提交，推送远端
- [x] Review Phase 6：等待生产镜像构建，部署 hd0526 并完成两次健康检查和模型目录验证

### Status

complete

### Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| `planning-with-files` 恢复脚本读到旧会话中与当前工作区不一致的描述 | 恢复未同步上下文 | 以当前 `git status`、完整差异和现有计划文件为准，旧描述不作为本次审查依据 |
| 新增空模型目录回归测试后按预期失败：成功数为 1，预期为 0 | 验证 HTTP 200 空目录不会清空权限 | 将空目录改为失败结果并保留原有权限和缓存，随后重新运行测试 |
| 将提交接到 `eb44f662` 后，生图架构文档出现一处内容冲突 | 保留已上线的同步图片保活说明，同时合并 Codex App 请求头结论 | 手工合并两组验证要求并完成 rebase，代码文件无冲突 |
| 首次触发镜像工作流时，GitHub CLI 使用了错误的默认仓库 | 直接运行 `gh workflow run app-image.yml` | 显式指定 `-R ryfineZ/Niffler` 后成功触发生产镜像构建 |
| 首次生产模型验证脚本存在 shell 引号错误，随后又使用了错误的管理鉴权头 | 调用本机管理接口验证三个号池 | 改用编码后的 Python 脚本，并按代码定义使用 `x-aether-admin-*` 请求头；Plus、Pro、Team 均验证成功 |
| hd0526 没有安装 `rg` | 筛选最近 15 分钟容器错误日志 | 改用 `grep -Eai`，两个容器均未发现 `panic`、`fatal` 或 `error` |

---

## sub2api JSON 导入兼容性（2026-07-14）

### Goal

核对用户提供的 sub2api 导出文件与 Niffler 导入协议，定位“无法解析输入内容”的确切原因并给出修复范围；本轮先诊断，不修改代码。

### Current Phase

Implement Phase 5

### Phases

- [x] Import Phase 1：验证 JSON 语法、编码和顶层结构
- [x] Import Phase 2：检查 Niffler 前端解析与后端导入协议
- [x] Import Phase 3：对照 sub2api 导出结构，确认缺失兼容分支和安全边界
- [x] Import Phase 4：汇总根因、影响范围和修复方案

### Status

complete

### Implementation

- [x] Implement Phase 1：确认显示名称规则与兼容边界
- [x] Implement Phase 2：新增设计记录
- [x] Implement Phase 3：补充前后端失败测试
- [x] Implement Phase 4：实现前端识别、后端展开与名称生成
- [x] Implement Phase 5：运行回归测试和 UI review（功能测试通过；UI 静态审查仅命中弹窗既有样式问题）

**Implementation Status:** complete

### Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| 读取 `token_import.rs` 时少写了 `dispatch/` 路径 | 检查 Access Token 单独导入行为 | 改用 `apps/aether-gateway/src/handlers/admin/provider/oauth/dispatch/token_import.rs` |
| 一条 `jq` 表达式因逗号优先级错误而索引数组 | 汇总平台、套餐和授权模式 | 拆成独立表达式重新查询 |
| UI 静态审查命中弹窗既有交互密度和配色问题 | 按紧凑工具界面规则复核本次前端改动 | 本次没有改可见界面；记录既有问题，不扩大本次兼容性修复范围 |

---

## 线上生图超时事故（2026-07-14）

### Goal

确认线上 Niffler 生图请求超时的具体环节和影响范围，修复后提交并部署生产环境。

### Current Phase

Incident Phase 5

### Phases

- [x] Incident Phase 1：核对客户端现象和线上网关日志
- [x] Incident Phase 2：核对请求记录、提供商端点与出站链路
- [x] Incident Phase 3：检查超时、取消和账号切换的实现
- [x] Incident Phase 4：得出结论并给出修复范围
- [x] Incident Phase 5：开启生产保活、提交代码、构建部署并完成真实生图验证

### Known Facts

- Codex App 已加载生图工具，请求已到达 Niffler 的 `/v1/images/generations`。
- 线上请求被路由到 Pro 号池的 `gpt-image-2`，多个不同账号都出现 60 秒、120 秒无上游响应。
- 同时段文本 Responses 请求大多仍能完成，问题集中在图片生成链路。
- 更正：进一步核对 `extra_data.image_progress` 后，大多数失败请求已在 0.3–2.8 秒内收到上游响应头，且收到 8–9 个 `keepalive` 事件；并非“上游完全无响应”。
- 连接统一在约 124.6 秒被取消，与 Cloudflare 默认 120 秒 Proxy Read Timeout 一致。
- 代码已有 15 秒 JSON 空白心跳实现，但 `should_enable_openai_image_sync_json_heartbeat` 固定返回 `false`，生产请求的心跳数为 0。
- 生产配置已开启同步生图保活；代码默认值改为开启，同时保留管理员显式关闭能力。
- 提交 `fcc5a165` 已部署至 hd0526，前台和后台容器均健康。
- 部署后真实生图请求 HTTP 200，0.87 秒收到首字节，21.56 秒完成，最终 JSON 中的图片可解码为有效 PNG。

### Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| hd0526 未安装 `rg` | 用 `rg` 过滤容器日志 | 改用 `grep -E` |
| Postgres `json` 不支持 `?` 操作符 | 直接检查 `request_headers` 键 | 转为 `jsonb` 后查询；该取消记录未保存请求头 |

---

# 暂停的原任务：后台钱包检索、用户时间展示与前端表格体验优化

## Goal

先复核并上线本次后台钱包检索和用户创建时间展示改动；随后系统检查前端后台页面在手机端、表格内容展示和列宽调整上的问题，并完成可验证的优化。

## Current Phase

Phase 1

## Phases

### Phase 1: 上线前复核
- [x] 复核钱包管理按用户名、邮箱、用户 ID 检索的前后端链路
- [x] 复核用户管理创建时间显示到具体时间
- [x] 运行前端类型检查、后端编译检查和 diff 空白检查
- [x] 确认本地未提交改动中哪些会进入本次上线
- [x] 修复 Vitest 在 Node 25 下的 localStorage 测试环境问题
- [x] 补充钱包用户搜索回归测试
- **Status:** completed

### Phase 2: 上线
- [ ] 提交并推送本地改动
- [ ] 等待 CI 构建生产镜像
- [ ] 按现有生产发布流程上线
- [ ] 验证后台服务健康和关键页面/API
- [ ] 记录本次上线包含的变更、影响范围和风险
- **Status:** in_progress

### Phase 3: 前端体验盘点
- [ ] 盘点后台主要页面的手机端布局问题
- [ ] 盘点表格内容看不全的问题
- [ ] 盘点表格列宽不能拖动调整的问题
- [ ] 明确优先级，避免一次性无边界改动影响生产
- **Status:** pending

### Phase 4: 前端体验优化
- [ ] 抽象或复用表格横向滚动、列宽调整和移动端展示能力
- [ ] 优先修后台高频页面
- [ ] 补齐空态、加载、错误和移动端布局验证
- **Status:** pending

### Phase 5: 最终验证与上线
- [ ] 运行前端类型检查和必要后端检查
- [ ] 执行 UI review，达到 `PASS ui-review gate`
- [ ] 上线并验证生产页面
- **Status:** pending

## Decisions Made

| Decision | Rationale |
|----------|-----------|
| 先上线前复核当前改动，再做全局前端优化 | 当前钱包检索和时间展示是明确需求，先避免拖延上线 |
| 前端优化按后台高频页面分批推进 | 全量后台页面一次性大改风险高，容易影响生产 |
| 表格优化优先做通用能力 | 避免每个页面分别打补丁，降低后续维护成本 |
| 本次上线包含当前工作区所有已完成改动 | 工作区已有多项前序需求处于未提交状态，生产发布流程要求以提交对应的 CI 镜像发布 |

## Known Facts

- 本地工作区存在大量前序任务的未提交改动，本次上线前必须确认发布范围。
- 本次已新增 `user_search` 后台查询参数，覆盖钱包、流水、退款和充值订单读取链路。
- 用户管理已有 `formatDateTime`，本次只把创建时间显示从日期切到日期时间。
- UI 方向按 Niffler 后台既有设计系统处理，不重做视觉风格。
- 生产发布文档要求通过 GitHub Actions 的 `Build App Image` 产物部署，不能直接在生产服务器编译。

## Errors Encountered

| Error | Attempt | Resolution |
|-------|---------|------------|
| `ui_workflow.py` 第一次运行缺少 `--platform`、`--surface`、`--goal` | 调用脚本时只传了 mode/path | 已按脚本要求补齐平台、界面类型、目标、代码路径和审查档位重新运行 |
| `npm --prefix frontend run test:run` 首次失败，`localStorage.clear/getItem is not a function` | Node 25 提供了不完整的实验性 `globalThis.localStorage`，jsdom 没覆盖 | 新增 Vitest setup，在测试环境缺少标准 Storage 方法时安装内存版 Storage |
| 新增 SQLite 钱包搜索回归测试首次编译失败，缺少查询结构体导入 | 测试模块 import 没包含新增查询类型 | 补充 `AdminWalletLedgerQuery` 和 `AdminWalletRefundRequestListQuery` 导入后测试通过 |
