# CC Switch 与一键配置修复记录

## 目标

让用户创建 API Key 后，可以稳定完成两类配置：

- 导入 CC Switch：生成正确的服务地址、余额检查地址和 Codex 默认模型口径。
- 一键配置 CLI：生成可信的公开 API 地址，并减少 Windows/macOS/Linux 脚本兼容问题。

## 非目标

- 不扩展 CC Switch 深链协议；该协议当前只稳定支持单个 `model` 字段，不支持直接传 Codex 模型目录。
- 不改变网关实际转发、扣费、套餐结算规则。
- 不替用户自动选择具体模型；没有填写模型时仍按用户总可用额度展示。

## 行为变化

- CC Switch 导入会单独传入余额检查地址，避免 Codex 端点是 `/v1` 时把余额接口拼成 `/v1/v1/usage`。
- 后端同时接受 `/user/balance` 和 `/v1/user/balance`，兼容客户端刷新用量时继续使用 OpenAI v1 服务地址作为 `baseUrl` 的情况。
- CC Switch 导入如果填写了模型，余额检查会把模型传给 Niffler，Niffler 按该模型查询套餐额度。
- Codex 导入只传 CC Switch 源码确认会读取的顶层 `model`、`endpoint` 和 `apiKey`；未填写模型时默认使用 `gpt-5.6-sol`，CC Switch 生成的 Codex 配置会写入 `model_reasoning_effort = "high"`。该导入不会写入 Codex 的 `model_catalog_json`，所以用户还需要执行页面提供的模型目录命令，Codex App 下拉列表才会显示 GPT-5.6 系列。
- API Key 页面打开 CC Switch 导入弹窗时默认选择 Codex，并自动填入 `gpt-5.6-sol`，减少用户误导入 Claude Code 的情况。
- CC Switch 导入弹窗为 Codex 用户提供独立的“更新 Codex 5.6 模型列表”命令。该命令不重配 API Key，不覆盖 CC Switch 管理的服务地址和 provider 配置，只写入本机 `niffler_model_catalog.json`，并更新 Codex `config.toml` 顶层 `model`、`review_model`、`model_catalog_json`。
- Codex 模型目录命令会优先读取本机 `codex debug models --bundled` 的官方默认模型目录，再追加 Niffler 提供的 GPT-5.6 Sol/Terra/Luna；不能用只包含 Niffler 模型的 JSON 替换整个目录，否则 Codex CLI 会只剩 5.6。
- CC Switch 导入弹窗默认 provider 名称使用 API Key 名称，不再自动加 `Niffler -` 前缀，避免用户重复导入时生成新 provider 名称。
- Codex App 向 `/v1/models?client_version=...` 请求模型目录时，Niffler 直接返回 Codex 可识别的 `{ "models": [...] }` 目录格式，目录包含 Codex 默认模型和 GPT-5.6 Sol/Terra/Luna。这样用户通过 CC Switch 导入后，不需要先手动维护本机模型目录文件，也能从服务端获取 5.6 系列模型。
- 普通 `/v1/models` 不带 `client_version` 时仍返回 OpenAI 标准 `{ "object": "list", "data": [...] }`，继续按账号真实可用模型展示，不受 Codex App 专用目录影响。
- Codex 一键配置和“更新 Codex 5.6 模型列表”命令都支持 macOS、Linux 和 Windows；如果用户设置了 `CODEX_HOME`，脚本优先使用该目录，并在输出中明确提示已使用自定义 Codex 主目录。未设置时分别使用 `~/.codex` 或 `%USERPROFILE%\.codex`。
- 如果用户机器无法执行脚本，页面提供 `/install/codex-model-catalog.json` 作为手动下载模板，并提供 macOS、Linux、Windows 三份可复制 `config.toml` 片段。用户只需要把模板保存到 Codex 主目录下的 `niffler_model_catalog.json`，再按系统复制配置片段并修改路径里的用户名或自定义 `CODEX_HOME`。
- 前端优先向后端获取公开 API 地址，不再只用浏览器当前域名推断。
- 一键配置生成公开地址时，公网默认使用 HTTPS；本机地址仍允许 HTTP。
- Windows 一键配置脚本避免依赖 PowerShell 7 专属的 `ConvertFrom-Json -AsHashtable`。

## 影响范围

- 用户 API Key 页面的一键配置、导入 CC Switch。
- `/v1/usage`、`/user/balance` 和 `/v1/user/balance` 余额检查接口。
- `/v1/models?client_version=...` Codex App 模型目录接口。
- `/api/users/me/public-base-url` 用户侧公开 API 地址接口。

## 验证方式

- 前端单测覆盖 CC Switch 深链接参数。
- 后端路由测试覆盖公开 API 地址接口。
- 后端脚本单测覆盖 PowerShell 兼容写法。
- 手动检查生成的 Codex CC Switch 链接中 `endpoint` 为 `/v1`，`model` 为 `gpt-5.6-sol`，`usageBaseUrl` 为根地址。
- 后端脚本测试覆盖 `/install/codex-models` 和 `/install/codex-models.ps1` 使用 `CODEX_HOME`、写入 `niffler_model_catalog.json`，并包含 GPT-5.6 Sol/Terra/Luna。
- 后端单测覆盖 `/install/codex-model-catalog.json` 使用的 JSON 模板，确认同时包含 Codex 默认模型和 GPT-5.6 Sol/Terra/Luna。
- 后端单测覆盖 `/v1/models?client_version=...` 返回 Codex 目录格式，确认包含 GPT-5.6 Sol/Terra/Luna，且不会触发上游转发。
