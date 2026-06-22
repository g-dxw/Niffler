# Codex 修复与 DoDoPay 支付接入

## 目标

- 合入上游 Codex Responses 的两个修复，避免普通工具请求被误改成图片工具请求，减少上游 400 错误导致的请求失败。
- 将 DoDoPay 接入现有支付体系，管理员可以在后台配置服务地址、产品 ID、API Key、Webhook Secret、回调地址、汇率和最低充值金额。
- 支持钱包充值和套餐购买使用 DoDoPay 创建支付订单，并通过 DoDoPay 回调完成订单入账或套餐发放。

## 非目标

- 不在代码仓库中保存线上 DoDoPay 密钥。
- 不在本次改动中自动调用管理员 API Key 创建 DoDoPay 应用。
- 不改变现有易支付配置和订单处理逻辑。
- 不重启 Postgres 或 Redis。

## 行为变化

- Codex 请求只有在明确选择图片生成工具时，才会按图片工具请求处理；普通工具列表里包含图片工具时，不再自动改写第一个工具。
- Codex 请求会移除 Hosted Tool 不该带的 `name` 字段，保留普通函数工具的 `name`。
- 支付配置页增加 DoDoPay 配置入口。
- DoDoPay 配置启用并保存 API Key 与 Webhook Secret 后，用户钱包充值和套餐购买页面会显示“支付宝”和“微信支付”两个 DoDoPay 支付入口。
- DoDoPay 新版 Checkout Sessions 使用 `payment_gateway_configs.merchant_id` 保存 Dodo 产品 ID，使用 `merchant_key_encrypted` 保存 Dodo API Key，使用 `webhook_secret_encrypted` 保存 Dodo Webhook Secret；管理员需要配置一个支持可变金额的 Dodo 产品。
- API Key 只用于创建 Checkout Session；Webhook Secret 只用于 Standard Webhooks 回调验签，不与 API Key 共用。
- 发起 DoDoPay 支付时，后端先创建本地 `pending` 订单，再创建 DoDoPay Checkout Session，最后把 DoDoPay 返回的 `payment_id` 或 `checkout_session_id` 回填到本地订单；这样即使数据库写入失败，也不会把可支付链接发给用户。
- 创建 DoDoPay Checkout Session 时，后端按用户选择写入 `allowed_payment_method_types`：支付宝为 `ali_pay`，微信为 `we_chat_pay`；同时写入带签名 token 的 `cancel_url`，让用户在 DoDoPay 收银台取消或返回时回到本系统。
- DoDoPay Checkout Session 创建失败或本地订单回填失败时，本系统会把仍处于 `pending` 的本地订单标记为 `cancelled`，并且不会向用户返回支付链接。
- 用户取消 DoDoPay 支付后，本系统只接受签名正确且支付服务为 DoDoPay 的本地订单；通过校验后只会把仍处于 `pending` 的订单标记为 `cancelled`，并记录取消时间和来源；已成功、已入账、已失败、已过期或其他支付服务的订单不会被取消回跳覆盖。
- DoDoPay 套餐订单按后台配置的收款币种和汇率计算应付金额；套餐币种不是 USD 且不等于收款币种时会拒绝下单，避免把 CNY 金额发给 USD 收银台这类错误配置。
- DoDoPay 支付成功回调优先按 Standard Webhooks 验证 `webhook-id`、`webhook-timestamp`、`webhook-signature` 和原始请求体，验签通过后按本地订单号、`payment_id` 或 `checkout_session_id` 完成订单入账。
- DoDoPay 回调会校验配置的产品 ID；支持读取官方回调中的 `product_cart[].product_id`。回调带有 `currency` 时，必须与后台配置的收款币种一致，否则拒绝入账。
- DoDoPay 回调金额按本地订单金额校验；当回调只提供含税 `total_amount` 时，会扣除回调中的 `tax` 后再与本地订单金额比较，避免用户已支付但因税费导致本地拒绝入账。
- DoDoPay 回调中的 `channel` 会标准化保存为订单的 `payment_channel`，订单列表和详情显示实际支付方式：`we_chat_pay` 显示为“微信支付”，`ali_pay` 显示为“支付宝支付”；没有渠道时才显示 DoDoPay。

## 已知限制

- 现有 DoDoPay 通知接口仍保留旧 HMAC 回调逻辑，用于兼容历史回调格式；检测到 Standard Webhooks 请求头时，必须使用 Webhook Secret 验签。

## 影响范围

- 请求格式处理：Codex OpenAI Responses / Compact / Image 相关转换。
- 后台管理：支付配置页和支付网关管理接口。
- 用户侧：钱包充值、套餐购买、支付成功回调。
- 数据层：`payment_gateway_configs` 新增 `webhook_secret_encrypted` 字段；继续复用 `payment_orders.status` 和 `payment_orders.payment_channel` 字段。

## 验证方式

- 运行 Codex 请求格式相关单元测试。
- 运行支付路由、DoDoPay Checkout 参数、取消回跳和回调相关测试。
- 运行前端类型检查或构建验证支付配置页。
- 部署后验证前后端健康检查，不重启数据库和 Redis。
