# Codex 修复与 DoDoPay 支付接入

## 目标

- 合入上游 Codex Responses 的两个修复，避免普通工具请求被误改成图片工具请求，减少上游 400 错误导致的请求失败。
- 将 DoDoPay 接入现有支付体系，管理员可以在后台配置服务地址、App ID、App Secret、回调地址、汇率和最低充值金额。
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
- DoDoPay 配置启用并保存 App ID 与 App Secret 后，用户钱包充值和套餐购买页面只显示“微信支付”一个 DoDoPay 支付入口；支付宝入口暂不开放。
- 当前接入使用 `https://pay.dodododo.org` 的应用协议：`payment_gateway_configs.merchant_id` 保存 DoDoPay App ID，`merchant_key_encrypted` 保存 DoDoPay App Secret。DoDoPay 没有单独的 Webhook Secret，当前配置校验也不要求它。
- 发起 DoDoPay 支付时，后端先创建本地 `pending` 订单，再用 App Secret 签名调用 `POST /api/v1/orders` 创建 DoDoPay 订单，最后把 DoDoPay 返回的 `order_id` 回填到本地订单；这样即使数据库写入失败，也不会把可支付链接发给用户。
- 创建 DoDoPay 订单后，后端调用 `POST /api/v1/orders/{order_id}/channel` 保存支付方式为 `WECHAT`。如果旧页面或直接接口请求传入支付宝，后端会拒绝创建新订单。
- 创建 DoDoPay 钱包充值和套餐购买订单时，后端会把当前 Niffler 用户名作为 `payer_name` 传给 DoDoPay，用于 DoDoPay 后台付款人展示；不传邮箱、联系方式或其他个人信息。
- DoDoPay 订单创建失败、支付方式保存失败或本地订单回填失败时，本系统会把仍处于 `pending` 的本地订单标记为 `cancelled`，并且不会向用户返回支付链接。
- DoDoPay 返回的支付指令包含本系统签名后的 `local_cancel_url`。用户在钱包充值或套餐购买页点击“取消这笔支付”时，本系统先调用 DoDoPay `POST /api/v1/orders/{order_id}/cancel` 取消上游订单，再把本地仍处于 `pending` 的订单标记为 `cancelled`，并记录取消时间和来源。
- 关闭新窗口、页面隐藏或普通返回业务站点不会被自动当成取消，避免用户已经扫码付款但到账确认尚未完成时被误取消。
- DoDoPay 套餐订单按后台配置的收款币种和汇率计算应付金额；套餐币种不是 USD 且不等于收款币种时会拒绝下单，避免把 CNY 金额发给 USD 收银台这类错误配置。
- DoDoPay 支付成功回调用同一个 App Secret 对去掉 `signature` 的 JSON 做 HMAC-SHA256 验签，验签通过后按本地订单号或 DoDoPay `order_id` 完成订单入账。
- DoDoPay 回调会校验配置的 App ID。回调带有 `currency` 时，必须与后台配置的收款币种一致，否则拒绝入账。
- DoDoPay 回调金额按本地订单金额校验；当回调只提供含税 `total_amount` 时，会扣除回调中的 `tax` 后再与本地订单金额比较，避免用户已支付但因税费导致本地拒绝入账。
- DoDoPay 回调中的 `channel` 会标准化保存为订单的 `payment_channel`，订单列表和详情显示实际支付方式：`we_chat_pay` 显示为“微信支付”，历史 `ali_pay` 订单仍显示为“支付宝支付”；没有渠道时才显示 DoDoPay。

## 影响范围

- 请求格式处理：Codex OpenAI Responses / Compact / Image 相关转换。
- 后台管理：支付配置页和支付网关管理接口。
- 用户侧：钱包充值、套餐购买、支付成功回调。
- 数据层：继续复用 `payment_gateway_configs.merchant_id`、`payment_gateway_configs.merchant_key_encrypted`、`payment_orders.status` 和 `payment_orders.payment_channel` 字段。

## 验证方式

- 运行 Codex 请求格式相关单元测试。
- 运行支付路由、DoDoPay Checkout 参数、取消回跳和回调相关测试。
- 运行前端类型检查或构建验证支付配置页。
- 部署后验证前后端健康检查，不重启数据库和 Redis。
