# GPT-5.6 模型上线配置记录

## 目标

让线上 Niffler 能展示并调度 GPT-5.6 系列模型，同时记录官方基础价格。

## 非目标

- 不修改用户套餐规则。
- 不修改 Codex OAuth 凭证解析流程。
- 不证明所有上游账号都实际拥有 GPT-5.6 调用权限。

## 行为变化

- 全局模型新增 `gpt-5.6`、`gpt-5.6-sol`、`gpt-5.6-terra`、`gpt-5.6-luna`。
- Pro 号池新增同名 provider model。
- `gpt-5.6` 按 `gpt-5.6-sol` 价格配置，并在 provider model 映射中指向 `gpt-5.6-sol`。
- Codex 预置模型列表新增 GPT-5.6 系列，供无实时模型列表时使用。
- 新创建或导入的 Codex OAuth 账号默认开启自动获取模型；重复导入更新已有账号时也会补开自动获取。
- 启用状态的 Codex OAuth 账号持久化后会立即后台获取一次模型，不再只等每日周期任务或服务重启。

## 影响范围

- 影响公开模型目录、全局模型目录、Pro 号池调度和模型计费配置。
- Codex OAuth 自动获取模型覆盖所有 Codex provider，包括 Plus、Pro、Team 号池；不影响第三方 custom provider。
- 线上已有 Codex OAuth 账号如果历史上关闭了 `auto_fetch_models`，需要单独做一次数据更新后才会进入周期性自动获取模型任务。本次已更新线上 Plus、Pro、Team 号池的 Codex OAuth key。

## 价格

价格来源：OpenAI GPT-5.6 发布页。

- `gpt-5.6` / `gpt-5.6-sol`：输入 5 USD / 1M tokens，输出 30 USD / 1M tokens。
- `gpt-5.6-terra`：输入 2.5 USD / 1M tokens，输出 15 USD / 1M tokens。
- `gpt-5.6-luna`：输入 1 USD / 1M tokens，输出 6 USD / 1M tokens。

线上沿用现有 GPT-5.4 / GPT-5.5 的长上下文计费规则：超过 272K 输入 token 后，输入价格乘 2，输出价格乘 1.5。

## 验证方式

- 查询 `global_models` 能看到 4 个 GPT-5.6 模型启用。
- 查询 Pro 号池 `models` 能看到 4 个 GPT-5.6 provider model 启用。
- 查询 `niffler_model_base_prices` 能看到 4 个 GPT-5.6 基础价格。
- 查询 Plus、Pro、Team 三个 Codex 号池的 OAuth key，`auto_fetch_models` 均已开启；所有启用 key 已完成一次启动阶段模型获取。
- `https://niffler.org/api/public/models` 能返回 GPT-5.6 系列模型。
- `https://niffler.org/api/public/global-models` 能返回 GPT-5.6 系列模型。
