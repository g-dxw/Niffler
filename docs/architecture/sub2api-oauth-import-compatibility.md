# sub2api OAuth 导出文件兼容

## 目标

Niffler 的 Provider“导入授权”入口支持 sub2api 管理后台导出的账号 JSON。导入时展开顶层 `accounts` 数组，将 OpenAI OAuth 凭证转换为 Niffler 现有的批量 OAuth 导入条目。

号池记录名称优先使用以下格式：

1. `账号邮箱 · 空间名称`
2. 没有空间名称时使用 `账号邮箱 · 空间 ID`
3. 没有邮箱时使用 `用户 ID · 空间名称` 或 `用户 ID · 空间 ID`

名称不得添加 `codex_` 前缀，也不使用 sub2api 外层的自定义 `name`。
邮箱和用户 ID 至少需要一个；空间 ID 可能被同一 Team 空间内的多个用户共享，不能单独作为账号身份。

## 非目标

- 不迁移 sub2api 的代理、并发数、优先级、计费倍率、自动暂停和模型映射。
- 不迁移 sub2api 的 WebSocket 模式或隐私状态。
- 不把没有 Refresh Token 的 Personal Access Token 伪装成可自动刷新的 OAuth 凭证。
- 不提供 sub2api 全量备份恢复；本功能只导入当前 Provider 的授权账号。

## 行为变化

- 前端根据 sub2api 的明确类型或完整包装结构识别导出对象，并发送到现有批量导入任务接口，不能仅凭 `exported_at` 或 `proxies` 单个字段判断。
- 后端只接受与当前 Provider 匹配的账号；Codex / ChatGPT Web 仅接受 `platform: "openai"`、`type: "oauth"` 的记录。
- 后端从 `credentials` 中读取 `access_token`、`refresh_token`、邮箱、账号 ID、用户 ID、空间名称和套餐，并使用显式字段映射，不合并任意未知字段。
- 每条无效记录都进入批量失败结果，错误信息不得包含 Token。
- sub2api 包装对象缺少有效的 `accounts` 数组时，批量结果明确报告格式错误。
- 缺少邮箱和用户 ID 的账号进入失败结果；空间 ID 不能单独通过身份校验。
- Personal Access Token 按现有临时授权逻辑保存，不能自动刷新。
- 重复账号继续沿用现有规则：活动账号不覆盖；失效、停用或过期账号允许替换。

## 影响范围

- 前端：OAuth 账号导入格式识别与导入提示。
- 后端：Provider OAuth 批量导入解析、号池记录命名和逐条失败统计。
- 现有 JSON 数组、单账号 JSON、JSON Lines 和逐行 Token 导入行为保持不变。

## 验证方式

- 使用脱敏的 sub2api 包装对象验证前端进入批量任务，而不是单条导入。
- 验证两个共用空间 ID、用户 ID 不同的 Team 账号都能被解析。
- 验证记录名称分别使用“邮箱 · 空间名称”和“邮箱 · 空间 ID”。
- 验证非 OpenAI、非 OAuth、缺少凭证的记录进入失败结果且不泄露 Token。
- 验证导入后的账号保留邮箱、账号 ID、用户 ID、套餐，并标记为不可自动刷新。
- 运行现有 OAuth 单条和批量导入回归测试，确认旧格式不受影响。
