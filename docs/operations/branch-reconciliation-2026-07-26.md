# 2026-07-26 分支整合与生产主线恢复

## 目标

将当前生产提交 `ca07f004f89314fc206806307b76ac50f52bc050` 与
`origin/main` 的 `24a2e5c932ce4cc8051f2e13c120278ba8a1b8cf`
整合进同一条 `main` 历史，确保已上线修复、数据库迁移和 PR #6 的界面改动都不会丢失。

整合完成后，生产只能从受保护的 `main` 发布。所有临时分支必须在提交可达性
和归档标签验证完成后才能删除。

## 非目标

- 不将历史回退重新当成功能合入。
- 不整体合并含有另一条上游历史的 Grok OAuth 分支。
- 不在核心主线恢复 PR 中扩展 Grok OAuth 业务功能。
- 不修改生产数据库中的既有迁移记录或业务数据。

## 整合前状态

| 对象 | 提交 | 处理结论 |
|------|------|----------|
| `origin/main` | `24a2e5c932ce4cc8051f2e13c120278ba8a1b8cf` | 必须完整合入 |
| 当前生产 `origin/codex/fix-codex-lite-tools` | `ca07f004f89314fc206806307b76ac50f52bc050` | 作为整合起点，必须完整合入 |
| `origin/codex/fix-image-heartbeat-timeout` | `fcc5a165ca617b02e8e4daadb962e6edcab76ce3` | 已进入当前生产链 |
| `origin/codex/fix-sub2api-import` | `eb44f662d47acbaebb2adcbafe5ce999a582e3bb` | 已进入当前生产链 |
| `origin/codex/integrate-production-main` | `2a30b77201624f38eecdf2cbbf7d9abc88e4d829` | 已被两条待整合主链覆盖 |
| `origin/codex/restore-image-studio` | `e44df8caf60a0b79312496d8c403949ad87f0803` | 已进入当前生产链 |
| `origin/codex/restore-main-and-guard` | `d8c5fe83e2a3a4f4b0564d59c450eb339868269a` | 已进入当前生产链 |
| `origin/hotfix/ci-rust-checks-after-5xx` | `936b090a81ae08c610d8aeb8e820b3294e5cb3c6` | 已被两条待整合主链覆盖 |
| `origin/rollback/deploy-8a6709c1` | `f63be41ba986fe4765b11f9d144c9d1e3e2b2063` | 独有提交已由 `44354c0ef` 的自动版本更新实现替代，只归档 |
| 本地历史回退 | `0257a5fcdd720e1b09fc60a807a3b6da504691d9` | 有意删除功能的回退，只归档 |
| 本地 Grok OAuth 分支 | `d9cdff9004e1caecc5a3fa6965846dda22569e25` | 只移植 `9087b7133`、`4ad96eae0`、`384f685ca`，另开 PR |

## 归档标签

以下带说明标签已经推送到 `origin`，并通过解引用后的提交号复核：

- `archive/20260726-main-before-reconcile`
- `archive/20260726-production-before-reconcile`
- `archive/20260726-rollback-deploy-8a6709c1`
- `archive/20260726-grok-oauth-provider`
- `archive/20260726-rollback-commit-0257a5fc`

## 行为变化

1. 使用正常合并提交同时保留生产链和 `main` 的历史，不改写已经部署的提交。
2. 合并冲突按文件和业务含义处理，必须保留生产数据库已执行的迁移文件。
3. 发布校验改由固定入口执行，待发布分支不能决定自己是否允许上线。
4. 普通生产发布只接受受保护 `main` 的准确提交，并要求目标包含当前生产提交。
5. 发布前只读核对生产迁移历史；新容器健康前不更新部署状态，失败时恢复旧镜像。
6. Grok OAuth 的三个业务提交在核心主线稳定后单独适配和验证。
7. 所有合并请求都必须产生 Rust、前端和发布工具三组检查结果，避免路径过滤导致
   必过检查一直等待。
8. 当前仓库只有一名具有写权限的成员，暂时不能要求另一人批准，否则仓库所有者
   自己创建的合并请求将无法合并。现阶段先强制合并请求和必过检查；增加第二名
   审核者后，再将最少批准数改为 1。

## 影响范围

- Git 历史与分支管理。
- GitHub Actions 生产镜像和生产发布工作流。
- `main` 的合并请求与必过检查规则。
- 生产部署脚本、健康检查、失败回退与部署状态文件。
- 数据库迁移兼容性预检。
- 后续 Grok OAuth 功能适配。

## 验证方式

### 提交完整性

```bash
git merge-base --is-ancestor ca07f004f89314fc206806307b76ac50f52bc050 HEAD
git merge-base --is-ancestor 24a2e5c932ce4cc8051f2e13c120278ba8a1b8cf HEAD
git rev-list --count <branch> --not HEAD
```

除文档中明确记录的“已被替代、历史回退、Grok 待适配”外，每个分支的
独有提交数必须为 0。

### 代码与构建

```bash
git diff --check
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test -p aether-data --lib
cargo test -p aether-model-fetch --lib
npm --prefix frontend run type-check
npm --prefix frontend run test:run
npm --prefix frontend run build
npm --prefix frontend audit --omit=dev
bash scripts/tests/test-deploy-ancestry.sh
```

合并 `main` 后发现其锁文件仍指向存在已知漏洞的旧补丁版本。本次只在现有
语义版本范围内更新锁文件，不升级主版本。生产依赖审计必须为 0；开发工具
依赖的剩余告警单独记录，不得误报为已经进入生产镜像。

### 生产发布

- 镜像、GitHub Actions 运行和 `origin/main` 的提交号一致。
- 镜像迁移清单覆盖生产 `_sqlx_migrations`。
- `frontdoor` 与 `background` 都通过容器健康检查。
- 公开首页、`/_gateway/health` 和一条真实 API 请求成功。
- 至少间隔 10 秒完成两轮检查。
- 健康失败时恢复旧镜像，且不写入新的 `.niffler-deployed-commit`。

### 主线保护

- `main` 禁止直接推送，变更必须通过合并请求。
- 必过检查为 `check`、`Frontend` 和 `Release tooling`。
- 合并前必须解决所有审查对话，禁止强制推送和删除 `main`。
- 只允许普通合并提交，禁止在本次整合中使用压缩合并或变基合并，确保当前生产
  `ca07f004f` 仍可从最终 `main` 到达。
- 当前最少批准数为 0，这是单成员仓库的临时限制，不代表省略代码验证。
- 增加第二名有写权限的审核者后，将最少批准数改为 1。

## 停止条件

- 任一关键祖先检查失败。
- 发现未归类的独有提交。
- 目标镜像缺少生产已执行迁移。
- 必过测试、审核或生产健康检查失败。
- 归档标签不能从远端恢复准确提交。

满足任一停止条件时，禁止继续发布或删除分支。

## 核心合并结果

核心整合提交为 `c8faac0cc29f0dfa50d3b6d6beda850a06b4ef62`，其两个父提交准确为
当前生产 `ca07f004f` 和旧 `main` `24a2e5c`。两个祖先检查均已通过。

合并后 8 个正常远端分支相对该提交的独有提交数均为 0。剩余例外为：

- `f63be41ba`：旧版固定 Codex 客户端版本，已由提交内的 `44354c0ef` 替代；
- `0257a5fcd`：有意删除功能的历史回退；
- Grok 分支的 718 个提交来自另一条上游合并历史，真实业务工作仍为
  `9087b7133`、`4ad96eae0`、`384f685ca`。
