# CI 镜像发布流程

## 目标

生产发布不再在服务器上编译 Rust、构建前端或执行 `docker build`。应用镜像由 GitHub Actions 构建，发布时服务器只负责加载镜像并重启容器。

生产发布必须保证待上线提交同时包含最新 `main` 和当前线上提交，防止从落后的功能分支发布时静默删除已经上线的功能。

## 非目标

- 不改变 Postgres、Redis 的部署方式。
- 不把 GitHub、GHCR 或服务器密钥写入仓库。
- 不要求服务器登录 GHCR。
- 不要求服务器安装 Rust、Node.js 或前端依赖。
- 不阻止管理员执行明确回滚；回滚必须显式使用 `--allow-rollback`。

## 行为变化

- 主应用镜像构建不再跟随 `main` 推送自动执行，需要在 GitHub Actions 手动触发 `Build App Image`。
- 当前线上只使用 Linux amd64，因此 `Build App Image` 只构建 amd64 的 `aether-gateway`。
- `Build App Image` 只产出 `niffler-app-linux-amd64` 镜像文件，不再推送 GHCR 镜像，避免重复构建和上传。
- `Build App Image` 使用 Node 22 运行时的官方 GitHub Actions，避免 CI 运行时升级影响生产镜像产物。
- `Build App Image` 在封装镜像前运行发布脚本语法检查和提交继承规则测试；检查失败时不生成生产镜像。
- `deploy.sh` 不再使用 `Dockerfile.app.local`，也不再计算代码哈希。
- `deploy.sh` 只执行镜像拉取和 `docker compose up -d --no-build`。
- `scripts/deploy-ci-artifact.sh` 会从 CI 下载镜像文件，上传到服务器，执行 `docker load`，再重启指定服务。
- 生产执行 `scripts/deploy-ci-artifact.sh` 必须显式传入 `--run-id` 或 `--commit`，不能默认部署“最新成功产物”。
- 使用 `--commit` 时，脚本会按提交号查找对应的成功 `Build App Image` 工作流；如果没有找到成功产物，脚本必须停止，不能退回到默认分支的最新产物。
- 脚本会读取 CI 运行对应的准确提交，确认工作流名称和结果，并从 GitHub 同步最新 `main`。
- 正常发布要求待上线提交同时包含最新 `main` 和当前线上提交；任一条件不满足时，在下载和上传镜像前停止。
- 当前线上提交优先从服务器的 `.niffler-deployed-commit` 读取；旧部署没有该文件时，从当前镜像的版本标签或提交标签识别。
- 首次部署没有现有应用镜像时，只检查待上线提交是否包含最新 `main`。
- 明确回滚必须额外传入 `--allow-rollback`，跳过提交继承检查；不能将该参数用于普通发布。
- 发布成功后，服务器写入 `.niffler-deployed-commit`，记录本次实际上线的完整提交号。
- CI 镜像写入 `org.opencontainers.image.revision` 标签，便于服务器在状态文件缺失时恢复当前线上提交。
- `--allow-latest-for-local` 只允许本地验证或临时排查使用，不能作为生产发布命令。

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

这个脚本会下载指定提交对应的 `Build App Image` 工作流产物，把镜像文件传到服务器，服务器加载成 `niffler-app:latest`，再重启 `frontdoor` 和 `background`。Postgres 和 Redis 不需要重启。

如果指定提交没有成功的 `Build App Image` 工作流，或者该提交没有同时包含最新 `main` 和当前线上提交，脚本会直接报错并停止。需要先完成分支合并并重新触发 CI。

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
- `bash scripts/tests/test-deploy-ancestry.sh`
- 不传 `--run-id`、`--commit` 和 `--allow-latest-for-local` 时，`scripts/deploy-ci-artifact.sh` 必须拒绝执行。
- 普通发布缺少最新 `main` 或当前线上提交时必须在下载镜像前停止。
- `--allow-rollback` 只跳过提交继承检查，不跳过工作流名称、成功状态和镜像产物检查。
- `Build App Image` 的 `Verify deployment tooling` 任务通过后才运行最终镜像封装。
- GitHub Actions 的 `Build App Image` 工作流成功，且日志不再出现官方 action 运行在 Node 20 的弃用警告。
- 服务器执行发布脚本后，`docker compose ps` 显示应用容器健康。
