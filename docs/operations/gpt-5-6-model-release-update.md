# GPT-5.6 模型上线配置记录

## 目标

让线上 Niffler 能展示并调度 Codex OAuth 上游实际返回的 GPT-5.6 系列模型，同时记录基础价格。

## 非目标

- 不把 `gpt-5.6-luna` 设为默认模型；它可以展示在模型列表中，真实调用是否可用以上游为准。
- 不修改 Codex OAuth 凭证解析流程。
- 不开放尚未确认发布的其他模型名；已发布但上游模型列表接口暂未返回的 Codex 模型，可作为临时补充。

## 行为变化

- Codex OAuth 模型展示以 ChatGPT Codex 上游模型获取接口返回结果为准，并临时补充已发布的 `gpt-5.6-sol`、`gpt-5.6-terra` 和 `gpt-5.6-luna`。
- 不对 Codex OAuth 增加 `gpt-5.6` 到 `gpt-5.6-sol` 这类自定义 provider model 映射。
- 不再为 Codex 维护写死预置模型列表；无实时模型列表时不展示 Niffler 自行补充的 Codex 模型。
- 已确认上游返回 GPT-5.6 系列后，Codex 号池 provider key 的模型限制需要同步加入这些模型，否则 `/v1/models` 和真实调用仍会被 key 级限制挡住。
- 新创建或导入的 Codex OAuth 账号默认开启自动获取模型；重复导入更新已有账号时也会补开自动获取。
- 启用状态的 Codex OAuth 账号持久化后会立即后台获取一次模型，不再只等每日周期任务或服务重启。
- Codex 号池自动获取模型时会保留已发布的 GPT-5.6 系列，避免上游临时返回旧列表时把 provider key 的模型白名单覆盖回 GPT-5.5。
- 管理端“获取上游模型”弹窗对 Codex provider 使用同一口径：在真实上游返回结果基础上补充已发布的 GPT-5.6 系列，方便管理员直接导入模型。
- Niffler 的 API Key 安装脚本会为 Codex CLI 自动写入本机 `niffler_model_catalog.json`，并在 `config.toml` 中设置 `model_catalog_json`，避免用户手动维护 `~/.codex/model_catalog.json` 才能看到 GPT-5.6 系列模型。
- CC Switch 导入链路不写 Codex 本机 `model_catalog_json`，只通过深链传单个默认模型；Codex 导入默认模型已改为 `gpt-5.6-sol`。
- CC Switch 用户可以在 API Key 页面复制独立命令更新本机 Codex 模型目录：macOS/Linux 使用 `/install/codex-models`，Windows 使用 `/install/codex-models.ps1`。该命令只更新模型目录和 Codex 顶层默认模型，不改 API Key、provider、服务地址。
- Codex 模型目录必须保留本机 Codex 自带默认模型，再追加 Niffler 提供的 GPT-5.6 Sol/Terra/Luna；不能写成只包含 GPT-5.6，否则 Codex CLI 的模型选择会丢失 GPT-5.5、GPT-5.4 等默认模型。
- 独立命令优先识别 `CODEX_HOME` 自定义 Codex 主目录；未设置时使用系统默认目录。脚本输出会说明实际写入目录，避免用户自定义目录时误写默认目录。
- 页面提供 `/install/codex-model-catalog.json` 下载模板，作为脚本无法执行时的手动备用方案。

## 影响范围

- 影响公开模型目录、全局模型目录、Codex 号池调度和模型计费配置。
- Codex OAuth 自动获取模型覆盖所有 Codex provider，包括 Plus、Pro、Team 号池；不影响第三方 custom provider。
- 线上已有 Codex OAuth 账号如果历史上关闭了 `auto_fetch_models`，需要单独做一次数据更新后才会进入周期性自动获取模型任务。本次已更新线上 Plus、Pro、Team 号池的 Codex OAuth key。
- 线上 Plus、Pro、Team 号池里带模型白名单的 Codex provider key 需要加入 GPT-5.6 系列；`gpt-5.6-luna` 可以展示，调用是否成功以上游为准。

## 价格

价格来源：OpenAI GPT-5.6 发布页。

- `gpt-5.6` / `gpt-5.6-sol`：输入 5 USD / 1M tokens，输出 30 USD / 1M tokens。
- `gpt-5.6-terra`：输入 2.5 USD / 1M tokens，输出 15 USD / 1M tokens。
- `gpt-5.6-luna`：输入 1 USD / 1M tokens，输出 6 USD / 1M tokens。

线上沿用现有 GPT-5.4 / GPT-5.5 的长上下文计费规则：超过 272K 输入 token 后，输入价格乘 2，输出价格乘 1.5。

2026-07-10 已查证线上 `rn01 / niffler-postgres / aether`：`gpt-5.6-sol`、`gpt-5.6-terra`、`gpt-5.6-luna` 三个全局模型均已启用并写入上述价格；每个模型已有 3 条启用的 provider model 关联。

## 验证方式

- 查询 Plus、Pro、Team 三个 Codex 号池的 OAuth key，`auto_fetch_models` 均已开启；启用 key 的模型清单来自上游模型获取结果。
- 使用 Niffler token 请求 `https://niffler.org/v1/models` 时，只返回该 Codex OAuth 账号真实获取到的模型名。
- 对返回的 GPT-5.6 系列模型做一次 `/v1/responses` 真实调用验证。
- 运行 Codex 安装脚本相关单元测试，并用 `codex debug models` 验证生成的模型目录能被 Codex 解析。
- 线上更新后，用 Niffler token 请求 `https://niffler.org/v1/models` 应返回 GPT-5.6 系列，并用 `gpt-5.6-sol` 完成一次 `/v1/responses` 调用。
- 运行 `cargo test -p aether-gateway codex_model_fetch_preserves_released_gpt_56_models`，确认 Codex 自动获取模型即使只拿到 GPT-5.5，也会保留已开放的 GPT-5.6 模型。
- 运行 `cargo test -p aether-gateway provider_query_codex_models_include_released_gpt_56_overrides`，确认管理端获取上游模型弹窗也会补充已开放的 GPT-5.6 模型。
- 运行 `npm run test:run -- src/features/api-keys/utils/__tests__/ccswitchImport.spec.ts`，确认 CC Switch Codex 导入默认使用 `gpt-5.6-sol`。
- 运行 Codex 模型目录脚本测试，确认公开脚本支持 `CODEX_HOME`，包含 `gpt-5.6-sol`、`gpt-5.6-terra` 和 `gpt-5.6-luna`。
- 运行 `cargo test -p aether-gateway codex_model_catalog_download_json_contains_codex_defaults_and_released_models`，确认下载模板同时包含 Codex 默认模型和已发布的 GPT-5.6 模型。
- 运行线上只读 SQL，确认 GPT-5.6 三个全局模型价格和 provider model 关联已存在。
