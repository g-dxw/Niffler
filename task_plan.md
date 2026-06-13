# Task Plan: 后台钱包检索、用户时间展示与前端表格体验优化

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
