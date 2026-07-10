# GPT-5.6 模型上线配置记录

## 目标

让线上 Niffler 能展示并调度 Codex OAuth 上游实际返回的 GPT-5.6 系列模型，同时记录基础价格。

## 非目标

- 不修改用户套餐规则。
- 不修改 Codex OAuth 凭证解析流程。
- 不为 Codex OAuth 自行补充上游未返回或不可调用的模型名。

## 行为变化

- Codex OAuth 模型展示以 ChatGPT Codex 上游模型获取接口返回结果为准。
- 不对 Codex OAuth 增加 `gpt-5.6` 到 `gpt-5.6-sol` 这类自定义 provider model 映射。
- 不再为 Codex 维护写死预置模型列表；无实时模型列表时不展示 Niffler 自行补充的 Codex 模型。
- 上游模型获取结果未返回 GPT-5.6 系列前，公开模型目录也不启用 GPT-5.6 系列展示。
- 新创建或导入的 Codex OAuth 账号默认开启自动获取模型；重复导入更新已有账号时也会补开自动获取。
- 启用状态的 Codex OAuth 账号持久化后会立即后台获取一次模型，不再只等每日周期任务或服务重启。
- Niffler 的 API Key 安装脚本会为 Codex CLI 自动写入本机 `niffler_model_catalog.json`，并在 `config.toml` 中设置 `model_catalog_json`，避免用户手动维护 `~/.codex/model_catalog.json` 才能看到 GPT-5.6 系列模型。

## 影响范围

- 影响公开模型目录、全局模型目录、Codex 号池调度和模型计费配置。
- Codex OAuth 自动获取模型覆盖所有 Codex provider，包括 Plus、Pro、Team 号池；不影响第三方 custom provider。
- 线上已有 Codex OAuth 账号如果历史上关闭了 `auto_fetch_models`，需要单独做一次数据更新后才会进入周期性自动获取模型任务。本次已更新线上 Plus、Pro、Team 号池的 Codex OAuth key。

## 价格

价格来源：OpenAI GPT-5.6 发布页。

- `gpt-5.6` / `gpt-5.6-sol`：输入 5 USD / 1M tokens，输出 30 USD / 1M tokens。
- `gpt-5.6-terra`：输入 2.5 USD / 1M tokens，输出 15 USD / 1M tokens。
- `gpt-5.6-luna`：输入 1 USD / 1M tokens，输出 6 USD / 1M tokens。

线上沿用现有 GPT-5.4 / GPT-5.5 的长上下文计费规则：超过 272K 输入 token 后，输入价格乘 2，输出价格乘 1.5。

## 验证方式

- 查询 Plus、Pro、Team 三个 Codex 号池的 OAuth key，`auto_fetch_models` 均已开启；启用 key 的模型清单来自上游模型获取结果。
- 使用 Niffler token 请求 `https://niffler.org/v1/models` 时，只返回该 Codex OAuth 账号真实获取到的模型名。
- 对返回的 GPT-5.6 系列模型做一次 `/v1/responses` 真实调用验证。
- 运行 Codex 安装脚本相关单元测试，并用 `codex debug models` 验证生成的模型目录能被 Codex 解析。
