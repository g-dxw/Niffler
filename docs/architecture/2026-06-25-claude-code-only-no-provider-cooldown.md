# Claude Code 客户端限制错误不触发供应商冷却

## 目标
- 当上游返回 `No available accounts: this group only allows Claude Code clients` 这类错误时，不把供应商账号记入冷却、近期失败冷却、健康降级或自适应 429 学习。

## 非目标
- 不改变普通 429、529、超时、认证失效、余额不足等真实上游健康错误的处理。
- 不自动修正上游分组配置或客户端伪装配置。

## 行为变化
- 这类错误按“客户端能力不匹配”处理，不写号池冷却。
- 近期失败冷却统计忽略这类错误，避免 60 秒内多次请求后把同一个供应商账号临时跳过。
- 本地健康分不因这类错误下降。
- 自适应限速不把这类错误当成真实 429 学习样本。
- 号池评分不把这类错误写成冷却状态或失败扣分。

## 影响范围
- 仅影响包含 `only allows Claude Code clients`、`restricted to Claude Code clients`、`only Claude Code clients allowed` 等文案的失败记录。
- 其他 `No available accounts` 错误仍按原规则处理。

## 验证方式
- 增加单元测试覆盖：这类错误不会触发号池冷却、近期失败冷却、健康降级、自适应 429 学习和号池评分冷却。
