# 管理端 Codex 号池窗口用量查询优化

## 目标

- 修复号池管理打开 Codex 账号列表时，Postgres 为计算 5H/周用量触发全表并行扫描和共享内存申请失败的问题。
- 保持 5H/周用量展示口径不变：仍以结算快照里的基础成本优先，结算状态以结算快照优先、主表状态兜底。

## 非目标

- 不改变号池调度逻辑。
- 不改变账号额度刷新、额度耗尽判断和页面状态定义。
- 不清理生产数据，不重启数据库，不调整数据库全局参数。

## 行为变化

- 管理端号池账号列表计算 Codex 5H/周用量时，不再读取 `usage_billing_facts` 视图做全表并行连接。
- 查询按每个账号的每个窗口单独聚合，先用 `provider_api_key_id + created_at` 缩小 `usage` 记录范围，再连接 `usage_settlement_snapshots` 判断最终结算状态和基础成本。
- 新增 Postgres 索引 `idx_usage_provider_api_key_created_at`，服务这类小窗口统计查询。

## 影响范围

- 管理端号池管理页的 Codex 账号列表。
- Postgres 使用记录窗口统计查询。
- 新建库和现有库的 Postgres 索引结构。

## 验证方式

- SQL 单元断言确认窗口统计不再从 `usage_billing_facts` 读取。
- 生产只读 `EXPLAIN` 对比确认执行计划不再全表并行扫描 `usage` 和 `usage_settlement_snapshots`，也不再出现 `Parallel Hash`。
- 打开号池管理页确认 `/api/admin/pool/{provider}/keys?page_size=50` 返回正常且耗时下降。
