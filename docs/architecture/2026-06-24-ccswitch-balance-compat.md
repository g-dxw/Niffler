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
- 返回余额口径为 `total_available_balance`，即钱包余额加套餐当日剩余额度；为了兼容 ccswitch 通用模板，同步写入 `balance` 和 `remaining`。
- 无限额钱包的 `total_available_balance` 继续返回 `null` 表示“不限制总可用额度”；但 `balance`、`remaining` 和 `quota.remaining` 必须返回数字，使用钱包余额加套餐当日剩余额度，避免 ccswitch 旧脚本把 `null` 显示为空或查询失败。
- 返回字段至少包含 `is_active`、`isValid`、`balance`、`remaining`、`unit`、`wallet_balance`、`package_balance`、`total_available_balance` 和 `unlimited`。
- API Key 无效或被锁定时返回 200，并把 `is_active` 和 `isValid` 设为 `false`，避免 ccswitch 把网络错误和密钥不可用混在一起。

## 影响范围

- 路由分类：新增 `/user/balance`，并兼容 `/v1/user/balance`。
- 本地接口处理：复用现有 ccswitch 用量查询的钱包余额读取逻辑。
- 前端导入：ccswitch 深链里的余额检查脚本使用 `/user/balance`。
- 鉴权：只接受已有 Niffler API Key 解析结果，不使用网页登录态 Session。

## 验证方式

- `cargo test -p aether-gateway classifies_ccswitch_balance_route_as_public_support_route`
- `cargo test -p aether-gateway classifies_ccswitch_v1_balance_route_as_public_support_route`
- `cargo test -p aether-gateway gateway_handles_ccswitch_user_balance_with_api_key_without_proxying_upstream`
- `npm run test:run -- src/features/api-keys/utils/__tests__/ccswitchImport.spec.ts`
