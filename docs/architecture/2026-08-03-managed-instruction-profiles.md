# 受管理提示词配置设计记录

> 日期：2026-08-03
> 状态：已实现，审查修复中

## 目标

为路由分组增加可关闭、可追踪、版本固定的受管理提示词配置。一次请求只使用实际选中分组的共享核心规则和至多一个专业模块，并统一支持最终上游格式 `openai:responses`、`openai:chat` 和 `claude:messages`。

实现必须满足以下行为：

- 配置绑定到实际选中的路由分组，在全局模型、Provider、Endpoint 或账号切换时保持不变；
- 完整保留客户端经过格式转换、Endpoint `body_rules` 和 Provider Request 路由规则后形成的原有指令；
- 客户端伪造 `<niffler-managed-instructions>` 文本不能阻止或改变服务端注入；
- 现有 Responses 图片生成规则继续位于最终 `instructions` 末尾，且只出现一次；
- 最终请求体和运行记录能够说明实际配置、版本、摘要、注入位置和未注入原因；
- 未选择路由分组，或分组未配置、未启用受管理提示词时，保持当前请求行为。

## 非目标

第一版不提供自定义提示词编辑器，不自动识别题材，不组合多个专业模块，不支持 `replace`，不修改模型响应，也不处理 `openai:responses:compact`。全局模型、Provider Model 和 Endpoint 不能覆盖路由分组配置。此前按全局模型配置的实现尚未发布，本轮直接删除旧入口和旧读取逻辑，不增加双配置优先级或数据迁移。

## 配置与内置注册表

继续使用版本化的 `routing_groups.config_json`，不增加数据库字段：

```json
{
  "managed_instructions": {
    "enabled": true,
    "profile_id": "security_research_v1",
    "merge_mode": "prepend"
  }
}
```

支持的合并模式为：

- `prepend`：始终注入，客户端指令完整保留在后；
- `if_missing`：最终目标字段没有非空客户端指令时才注入。

第一版只内置 `security_research_v1` 和 `adult_fiction_v1` 两个可选配置。`security_research_v1` 合并安全、CTF 与逆向工程规则；`core_v1` 仅作为两个配置共享的内部正文，不出现在管理端配置列表中。普通路由分组不配置受管理提示词，避免无意义注入和额外 Token 消耗。提示词源码放在 `apps/aether-gateway/prompts/managed/`，由网关在构建时嵌入。注册表提供配置 ID、显示名称、说明、核心版本、专业模块版本、最终正文和 SHA-256 摘要。配置 ID 必须逐字符匹配注册表，首尾空格不会被自动删除；未知 ID、带首尾空格的 ID、未知合并模式、缺少字段、错误类型、空源码或无效 UTF-8 都显式失败，不自动改用其他配置。

## 固定正文与 SHA-256 规则

所有协议复用同一个 `embedded_text`，摘要不包含客户端指令、图片规则、JSON 字段、消息对象或 Claude 内容块。

正文生成规则固定如下：

1. 源文件必须为不带 BOM 的 UTF-8；
2. 将 `CRLF` 和单独 `CR` 统一为 `LF`，移除每个源码末尾的全部 `LF`，保留其他字符；
3. 每个配置都在规范化核心文本与专业模块之间放一个空行，结束标签后无换行；
4. 对最终 `embedded_text` 的 UTF-8 字节计算 SHA-256，输出 64 位小写十六进制字符串。

正文格式为：

```text
<niffler-managed-instructions profile="PROFILE_ID">
{core_text}

{domain_text}
</niffler-managed-instructions>
```

## 请求级配置快照与幂等

路由分组选择完成后，`LocalRoutingRequestContext` 已保存分组 ID、版本和完整 `config_json`。`LocalRequestedModelDecisionInput` 使用共享的请求级配置快照，第一次构造最终 Provider 请求时解析该分组配置，后续全局模型、Provider、Endpoint 或账号尝试复用同一结果，不再读取全局模型数据库。没有选中路由分组时固定为空配置；选中分组但配置无效时显式失败。快照同时保存 `managed_instructions` 的实际配置值；后续上下文只要分组 ID、版本或该配置值任一不同，就返回明确的内部请求构造错误。同一分组内切换全局模型属于正常调度。

每个已构造的 `AiExecutionDecision` 在内部 `report_context.managed_instructions` 中记录应用状态。统一后处理再次作用于同一个决策时：

- 配置 ID、摘要和目标字段相同：不再修改正文，记录 `deduplicated: true` 和 `reason: already_applied`；
- 配置 ID、摘要或目标字段不同：显式失败，不叠加、不替换；
- 没有可信内部状态：正常处理，即使客户端正文含相同或不同 XML 标签也照常注入。

可信内部状态已经表明正文完成注入时，统一后处理不得再次执行 Provider Request 路由规则，避免路由正文补丁覆盖或重复修改已经完成的最终请求体；它只核对固定配置并更新去重记录。

失败切换重新构造请求体时，该决策没有已应用状态，因此使用固定配置快照重新注入；这不会在同一个最终上游请求体中产生重复内容。XML 检查只生成 `client_marker_present`，不参与注入判断。

## 最终处理顺序

现有代码先完成格式转换、Endpoint `body_rules`、Codex 特殊字段和图片桥接，再构造 `AiExecutionDecision`，之后才执行 Provider Request 路由规则。为避免路由规则删除受管理内容，统一入口按以下顺序工作：

1. 完成现有 Provider Request 路由规则；
2. 读取请求级固定的路由分组配置；
3. 对带 `image_generation` 工具的 Responses 临时分离现有固定图片桥接；如果路由规则已经将它挤到中间，则移除最后一份完整固定正文；如果路由规则已经覆盖它，则准备重新追加；
4. 按最终 `provider_api_format` 识别客户端指令并注入；
5. 将图片桥接后缀原样接回，确保它仍是绝对末尾；
6. 将执行结果写入 `report_context`，最终 `provider_request_body` 随现有记录链路保存。

图片桥接分离必须同时满足最终请求仍有 `image_generation` 工具和正文完整匹配，不能根据客户端可伪造的标记进行模糊裁剪。没有图片工具时，即使客户端提交相同正文，也按客户端内容完整保留。实现同时移除现有图片桥接函数对原 `instructions` 的 `trim_end()`，避免无意删除客户端结尾字符。

## 三种最终格式

### OpenAI Responses

- `instructions` 缺失、`null` 或空字符串：写入受管理正文；
- 非空字符串：写入“受管理正文 + 一个空行 + `<niffler-client-instructions>` 区块”，区块中的原字符串逐字符保留；
- 其他类型：返回 400 请求构造错误；
- `if_missing` 以分离图片桥接后的原指令是否非空为判断依据；
- 目标字段记录为 `instructions`。

### OpenAI Chat

- `messages` 必须是数组；
- `prepend` 在索引 0 新增 `{ "role": "system", "content": embedded_text }`；
- 原消息对象、扩展字段和相对顺序不变；
- `if_missing` 在没有非空 `system` 或 `developer` 消息时才新增；字符串内容和文本内容块都参与非空判断；
- 目标字段记录为 `messages[0]`。

### Claude Messages

- `system` 缺失、`null` 或空字符串：写入受管理正文字符串；
- 非空字符串：写入“受管理正文 + 一个空行 + 原字符串”；
- 内容块数组：在索引 0 新增 `{ "type": "text", "text": embedded_text }`，原内容块及 `cache_control` 等字段不变；
- 其他类型：返回 400 请求构造错误；
- `if_missing` 以非空字符串或非空文本内容块为判断依据；
- 目标字段按实际结构记录为 `system` 或 `system[0]`。

## 不支持格式与错误语义

功能未启用时任何格式都不修改正文，并记录 `reason: disabled`。功能已启用但最终格式不是三种支持格式时，请求继续执行、不注入，记录 `reason: unsupported_provider_api_format`；这包括 `openai:responses:compact`。运行环境无法解析已声明支持格式的目标字段，或配置本身无效时，显式拒绝请求，不静默跳过。

因此，“不支持的最终格式”是可观察的跳过；“支持格式中的非法正文结构或无效配置”是明确错误，两者不冲突。

## 运行记录

沿用现有 `report_context` 和请求记录，不修改数据库结构。`managed_instructions` 固定记录：

```json
{
  "applied": true,
  "routing_group_id": "security-group",
  "routing_group_version": 3,
  "profile_id": "security_research_v1",
  "merge_mode": "prepend",
  "core_version": "core_v1",
  "profile_sha256": "64 位小写摘要",
  "provider_api_format": "openai:chat",
  "target_field": "messages[0]",
  "client_instructions_present": true,
  "deduplicated": false,
  "client_marker_present": false,
  "reason": "applied"
}
```

`reason` 只使用 `applied`、`already_applied`、`client_instructions_present`、`disabled` 和 `unsupported_provider_api_format`。`if_missing` 因客户端指令存在而跳过时，固定为 `applied: false`、`deduplicated: false`、`target_field: null` 和 `reason: client_instructions_present`。

只要分组配置存在，`routing_group_id` 和 `routing_group_version` 就记录本次请求固定使用的分组来源；功能关闭、不支持格式和 `if_missing` 跳过时也保留这两个字段。未选择分组或分组没有 `managed_instructions` 时不增加运行记录字段。

未配置 `managed_instructions` 时不增加运行记录字段，保持默认路径当前行为。普通运行日志不输出完整提示词正文。

## 管理接口与界面

管理端新增只读注册表接口：

```text
GET /api/admin/routing/managed-instruction-profiles
```

它使用现有路由分组管理权限，返回支持的配置、版本、摘要、合并模式、支持格式和组合顺序说明，不返回外部凭据。

路由分组创建、更新和发布版本前统一校验 `config_json.managed_instructions`。界面在分组基本信息与排序方式之间增加一个紧凑区域，提供启用、配置和合并模式三个操作，并显示版本、摘要与顺序预览；该区域始终属于整个分组，不随统一排序或按模型排序切换。注册表加载中禁用选择；加载失败和保存校验失败均显示明确错误。未配置时固定显示空选择和“未配置”，不显示任何配置的版本或摘要；管理员首次启用时才使用注册表第一项作为默认配置。关闭已有配置时保留已选配置但不生效，配置选择仍可操作；若分组保存了已经删除或无法识别的配置 ID，即使功能处于关闭状态也持续显示错误，并允许管理员直接选择现有配置完成修复，不要求先启用功能。全局模型编辑页不再显示或保存该配置。

## 影响范围与性能

修改范围包括网关提示词注册表、最终请求后处理、路由分组管理校验与注册表接口、Responses 图片桥接辅助函数、前端路由分组编辑页、前端类型和中英文文案。数据库、公共用户接口和模型响应结构不变。

分组配置已在请求选择阶段进入内存上下文，最终请求处理不增加数据库读取；后续失败切换只复用内存快照。摘要和正文由进程内静态注册表复用，不在每次尝试中重复读取文件或重新计算。未选择分组或分组未配置时只完成一次空配置解析。

## 验证方式

- 注册表测试：两个配置、正文规范化、固定摘要、未知配置、空源码和专业正文隔离；
- 协议单元测试：Responses、Chat、Claude 的空值、保留、`if_missing`、非法结构、伪造标签、可信去重和冲突；
- 流水测试：使用选中路由分组的配置分别验证 Responses、Chat 和 Claude 最终请求体；同时覆盖 Endpoint `body_rules`、Provider Request 路由、同格式透传、跨格式转换、图片桥接末尾、同组切换模型和不同分组快照冲突；
- 管理端测试：路由分类、注册表响应、分组创建、更新和发布配置校验，并确认全局模型保存不再处理该字段；
- 前端测试：加载、启用、选择、关闭、预览、错误和提交载荷；
- 运行 Rust 格式、相关测试与检查，运行前端类型检查和相关测试，最后执行 UI 阻断检查与差异复核。

真实付费上游对照测试不在本轮本地实现验证中执行，避免未经再次确认消耗外部余额；上线前应在隔离测试模型上另行完成。
