# Findings: 后台钱包检索、用户时间展示与前端表格体验优化

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
