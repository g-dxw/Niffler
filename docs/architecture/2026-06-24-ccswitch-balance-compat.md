# ccswitch 余额查询兼容接口

## 目标

- 支持 ccswitch 通用脚本通过 `GET /user/balance` 查询 Niffler 用户可用余额。
- 兼容把 OpenAI v1 服务地址当作余额查询基地址的客户端，额外接受 `GET /v1/user/balance`。
- 使用 Niffler 用户 API Key 鉴权，兼容 `Authorization: Bearer <apiKey>`。
- 返回 ccswitch 脚本可直接读取的 `is_active` 和 `balance` 字段。

## 非目标

- 不修改 ccswitch 客户端实现。
- 不替换现有网页登录态接口 `GET /api/wallet/balance`。
- 不新增独立余额计费或写入行为。

## 行为变化

- `GET /user/balance` 和 `GET /v1/user/balance` 被识别为本地公共辅助接口，不再代理到上游。
- 用户 API Key 页生成的 ccswitch 导入脚本会访问 `/user/balance`，不再访问旧的 `/v1/usage`。
- 该接口使用 `key:usage` 鉴权语义，`Authorization: Bearer` 会按 Niffler API Key 解析，并跳过 Provider 与 API Format 限制。
- `total_available_balance` 继续表示钱包余额加套餐当日剩余额度，供需要总可用额度的调用方读取。
- ccswitch 卡片中的 `balance`、`remaining` 和 `quota.remaining` 只表示钱包余额，不再混入套餐额度；套餐当日剩余额度通过 `package_balance` 和 `daily_quota` 单独返回。
- 无限额钱包的 `total_available_balance`、`balance`、`remaining` 和 `quota.remaining` 都返回 `null`，并返回 `unlimited: true`。钱包表中的历史数值不代表可用上限，不能再作为不限额账户的余额展示。
- 用户 API Key 页生成的 ccswitch 查询脚本识别 `unlimited: true`，不返回数值余额，改用额外文本显示“无限额”；有限额账户显示钱包余额，有套餐时用额外文本显示套餐额度。当前 ccswitch 允许 `remaining` 为空并支持 `extra` 文本，因此无需伪造或合并数值。
- 返回字段至少包含 `is_active`、`isValid`、`balance`、`remaining`、`unit`、`wallet_balance`、`package_balance`、`total_available_balance` 和 `unlimited`。
- API Key 无效或被锁定时返回 200，并把 `is_active` 和 `isValid` 设为 `false`，避免 ccswitch 把网络错误和密钥不可用混在一起。

## 影响范围

- 路由分类：新增 `/user/balance`，并兼容 `/v1/user/balance`。
- 本地接口处理：复用现有 ccswitch 用量查询的钱包余额读取逻辑。
- 前端导入：ccswitch 深链里的余额检查脚本使用 `/user/balance`。
- 已导入的旧脚本在后端升级后不再显示错误数值；重新导入后会显示“无限额”文本。
- 鉴权：只接受已有 Niffler API Key 解析结果，不使用网页登录态 Session。

## 验证方式

- `cargo test -p aether-gateway classifies_ccswitch_balance_route_as_public_support_route`
- `cargo test -p aether-gateway classifies_ccswitch_v1_balance_route_as_public_support_route`
- `cargo test -p aether-gateway gateway_handles_ccswitch_user_balance_with_api_key_without_proxying_upstream`
- `npm run test:run -- src/features/api-keys/utils/__tests__/ccswitchImport.spec.ts`
