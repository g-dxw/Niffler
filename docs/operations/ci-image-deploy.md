# CI 镜像发布流程

## 目标

生产发布不再在服务器上编译 Rust、构建前端或执行 `docker build`。应用镜像由 GitHub Actions 构建，发布时服务器只负责加载镜像并重启容器。

生产发布必须保证待上线提交就是受保护 `main` 的准确提交，并且包含当前线上提交，防止从落后的功能分支发布时静默删除已经上线的功能。

是否允许上线、迁移是否兼容、容器是否健康以及失败后是否回退，都由生产主机
`/opt/niffler-release/bin/deploy-production` 的固定部署器判断。固定部署器位于应用
目录之外，不能由待发布分支替换。

## 非目标

- 不改变 Postgres、Redis 的部署方式。
- 不把 GitHub、GHCR 或服务器密钥写入仓库。
- 不要求服务器登录 GHCR。
- 不要求服务器安装 Rust、Node.js 或前端依赖。
- 不阻止管理员执行明确回滚；回滚必须显式使用 `--allow-rollback`。
- 不把个人 SSH 私钥保存到 GitHub Actions。

## 行为变化

- 主应用镜像构建不再跟随 `main` 推送自动执行，需要在 GitHub Actions 手动触发 `Build App Image`。
- 当前线上只使用 Linux amd64，因此 `Build App Image` 只构建 amd64 的 `aether-gateway`。
- `Build App Image` 只产出 `niffler-app-linux-amd64` 镜像文件，不再推送 GHCR 镜像，避免重复构建和上传。
- `Build App Image` 使用 Node 22 运行时的官方 GitHub Actions，避免 CI 运行时升级影响生产镜像产物。
- `Build App Image` 在封装镜像前运行发布脚本语法检查和提交继承规则测试；检查失败时不生成生产镜像。
- `deploy.sh` 不再使用 `Dockerfile.app.local`，也不再计算代码哈希。
- `deploy.sh` 只执行镜像拉取和 `docker compose up -d --no-build`。
- `scripts/deploy-ci-artifact.sh` 只负责从 CI 下载准确镜像并上传；生产变更由服务器固定部署器执行。
- 生产执行 `scripts/deploy-ci-artifact.sh` 必须显式传入 `--run-id` 或 `--commit`，不能默认部署“最新成功产物”。
- 使用 `--commit` 时，脚本会按提交号查找对应的成功 `Build App Image` 工作流；如果没有找到成功产物，脚本必须停止，不能退回到默认分支的最新产物。
- 脚本会读取 CI 运行对应的准确提交，确认工作流名称和结果，并从 GitHub 同步最新 `main`。
- 正常发布要求待上线提交等于远端 `main`，并同时包含当前线上提交；任一条件不满足时停止。
- 当前线上提交优先从服务器的 `.niffler-deployed-commit` 读取；旧部署没有该文件时，从当前镜像的版本标签或提交标签识别。
- 首次部署没有现有应用镜像时，只检查待上线提交是否包含最新 `main`。
- 明确回滚必须额外传入 `--allow-rollback`，跳过提交继承检查；不能将该参数用于普通发布。
- 固定部署器从公开 Git 远端自行同步 `main`，不信任客户端传入的主线提交号。
- 镜像提供只读 PostgreSQL 迁移兼容检查入口。固定部署器使用当前
  `frontdoor` 容器的实际环境和网络运行目标镜像，由目标镜像直接检查生产
  `_sqlx_migrations`；数据库可以是外部 PostgreSQL，不要求同一 Compose 项目中
  存在名为 `postgres` 的服务。
- 生产迁移记录为脏状态，或目标镜像缺少任一已执行版本时，必须在重启容器前停止。
- 固定部署器保留当前镜像标签，使用准确提交标签重建服务并通过 `docker compose up --wait` 等待健康。
- 新服务未按时健康、源站健康请求失败或公开健康请求失败时，固定部署器自动恢复旧镜像和旧服务。
- 只有新服务全部健康后，服务器才写入 `.niffler-deployed-commit`，记录本次实际上线的完整提交号。
- CI 镜像写入 `org.opencontainers.image.revision` 标签，便于服务器在状态文件缺失时恢复当前线上提交。
- `--allow-latest-for-local` 只允许本地验证或临时排查使用，不能作为生产发布命令。
- 所有合并请求都运行 Rust、前端和发布工具检查，`main` 只接受这三组检查通过的
  合并请求。

## 主线保护

`main` 的必过检查为：

- `check`：Rust 格式、静态检查、测试和 PostgreSQL 数据库冒烟测试的汇总结果；
- `Frontend`：前端生产依赖审计、类型检查、测试和构建；
- `Release tooling`：发布脚本语法和固定部署器行为测试。

这些工作流不使用路径过滤，因此任意合并请求都会返回明确结果，不会因“未触发”
而让必过检查一直等待。`main` 禁止强制推送和删除，所有对话必须解决后才能合并。
当前只允许普通合并提交，避免压缩合并或变基合并破坏生产链整合所需的祖先关系。

当前仓库只有一名具有写权限的成员，若要求至少一名他人批准，仓库所有者自己创建
的合并请求将无法合并。因此现阶段最少批准数设为 0，仍强制合并请求和全部必过
检查。增加第二名审核者后，必须将最少批准数改为 1。

## 影响范围

- GitHub Actions 主应用镜像构建流程只产出 amd64 镜像文件。
- 线上发布使用 CI 产出的镜像文件，不依赖服务器访问私有 GHCR。
- 服务器 `.env` 中的 `APP_IMAGE` 应设置为 `niffler-app:latest`，由 `docker load` 后的本地镜像提供。
- 服务器应用目录新增 `.niffler-deployed-commit` 状态文件，不改变数据库、Redis 或业务数据。

## 发布方式

使用 CI 镜像文件发布。以 hd0526 为例：

```bash
APP_SERVICES="frontdoor background" \
APP_IMAGE=niffler-app:latest \
GH_REPO=ryfineZ/Niffler \
./scripts/deploy-ci-artifact.sh \
  --host hd0526 \
  --remote-dir /opt/niffler-app \
  --commit <git-commit-sha>
```

这个脚本会下载指定提交对应的 `Build App Image` 工作流产物并传到服务器，
随后调用固定部署器。固定部署器重新确认远端 `main`、当前生产提交、镜像标签，
并让目标镜像使用当前 `frontdoor` 的实际数据库环境完成只读迁移兼容检查后，才
重启 `frontdoor` 和 `background`。Postgres 和 Redis 不需要重启。

如果指定提交没有成功的 `Build App Image` 工作流、不是远端 `main`、没有包含
当前线上提交或缺少生产已执行迁移，发布会直接停止。需要先完成分支合并并重新触发 CI。

如果已经知道 GitHub Actions run id，也可以使用：

```bash
APP_SERVICES="frontdoor background" \
APP_IMAGE=niffler-app:latest \
GH_REPO=ryfineZ/Niffler \
./scripts/deploy-ci-artifact.sh \
  --host hd0526 \
  --remote-dir /opt/niffler-app \
  --run-id <github-actions-run-id>
```

本地验证或临时排查时，才可以显式选择最新成功产物：

```bash
./scripts/deploy-ci-artifact.sh \
  --host <test-host> \
  --allow-latest-for-local
```

只有明确回滚到旧版本时才能使用：

```bash
APP_SERVICES="frontdoor background" \
APP_IMAGE=niffler-app:latest \
GH_REPO=ryfineZ/Niffler \
./scripts/deploy-ci-artifact.sh \
  --host hd0526 \
  --remote-dir /opt/niffler-app \
  --run-id <github-actions-run-id> \
  --allow-rollback
```

回滚仍要求指定的 CI 工作流成功且镜像产物存在，只跳过“必须包含最新 `main` 和当前线上提交”的检查。

## 验证方式

- `bash -n deploy.sh`
- `bash -n scripts/deploy-ci-artifact.sh`
- `bash -n scripts/fixed-production-deployer.sh`
- `bash scripts/tests/test-deploy-ancestry.sh`
- `bash scripts/tests/test-fixed-production-deployer.sh`
- 不传 `--run-id`、`--commit` 和 `--allow-latest-for-local` 时，`scripts/deploy-ci-artifact.sh` 必须拒绝执行。
- 普通发布缺少最新 `main` 或当前线上提交时必须在下载镜像前停止。
- `--allow-rollback` 只跳过提交继承检查，不跳过工作流名称、成功状态和镜像产物检查。
- `Build App Image` 的 `Verify deployment tooling` 任务通过后才运行最终镜像封装。
- GitHub Actions 的 `Build App Image` 工作流成功，且日志不再出现官方 action 运行在 Node 20 的弃用警告。
- 正常发布目标不等于远端 `main` 时，固定部署器必须拒绝执行。
- 数据库部署在 Compose 项目之外时，固定部署器仍必须通过目标镜像完成只读迁移
  兼容检查，不能依赖 `docker compose exec postgres`。
- 目标镜像缺少生产已执行迁移时，固定部署器必须在重建容器前拒绝执行。
- 新容器健康失败时，固定部署器必须恢复旧镜像，且部署状态文件保持旧提交。
- 服务器执行发布脚本后，`docker compose ps` 显示应用容器健康，状态文件、
  镜像标签和 `origin/main` 为同一提交。

## 固定部署器安装

固定部署器更新与普通应用发布分开执行。检查脚本和测试通过后，由管理员显式安装：

```bash
ssh hd0526 'sudo install -d -m 0755 /opt/niffler-release/bin /opt/niffler-release/git'
scp scripts/fixed-production-deployer.sh hd0526:/tmp/deploy-production
ssh hd0526 'sudo install -m 0755 /tmp/deploy-production /opt/niffler-release/bin/deploy-production && rm -f /tmp/deploy-production'
```

普通发布不得覆盖该文件。以后修改固定部署器时，必须先通过独立审核和测试，再执行
一次显式安装。

当前仓库没有 Actions 专用部署密钥或自托管运行器，因此生产发布暂由已授权管理员
从本机调用固定部署器。配置专用发布凭证后，再将同一固定入口接入受保护的 GitHub
`production` 环境；不得上传个人 SSH 私钥。
