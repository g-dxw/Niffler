# CI 镜像发布流程

## 目标

生产发布不再在服务器上编译 Rust、构建前端或执行 `docker build`。应用镜像由 GitHub Actions 构建，发布时服务器只负责加载镜像并重启容器。

## 非目标

- 不改变 Postgres、Redis 的部署方式。
- 不把 GitHub、GHCR 或服务器密钥写入仓库。
- 不要求服务器登录 GHCR。
- 不要求服务器安装 Rust、Node.js 或前端依赖。

## 行为变化

- 主应用镜像构建不再跟随 `main` 推送自动执行，需要在 GitHub Actions 手动触发 `Build App Image`。
- 当前线上只使用 Linux amd64，因此 `Build App Image` 只构建 amd64 的 `aether-gateway` 和 amd64 镜像文件。
- 从 `main` 手动触发时，CI 会推送 `ghcr.io/ryfinez/niffler:main` 和 `ghcr.io/ryfinez/niffler:sha-xxxxxxx`，并上传 `niffler-app-linux-amd64` 镜像文件。
- 从其他分支手动触发时，CI 只推送 `sha-xxxxxxx` 和手动填写的镜像标签，不覆盖 `main` 镜像。
- `deploy.sh` 不再使用 `Dockerfile.app.local`，也不再计算代码哈希。
- `deploy.sh` 只执行镜像拉取和 `docker compose up -d --no-build`。
- `scripts/deploy-ci-artifact.sh` 会从 CI 下载镜像文件，上传到服务器，执行 `docker load`，再重启指定服务。

## 影响范围

- GitHub Actions 主应用镜像构建流程会产出 GHCR 镜像和 amd64 镜像文件。
- 线上发布优先使用 CI 产出的镜像文件，不依赖服务器访问私有 GHCR。
- Compose 默认镜像统一为 `ghcr.io/ryfinez/niffler`。

## 发布方式

如果服务器可以公开拉取镜像，服务器 `.env` 中设置：

```env
APP_IMAGE=ghcr.io/ryfinez/niffler:main
```

发布时执行：

```bash
./deploy.sh
```

脚本会拉取 `APP_IMAGE` 指向的镜像，然后重启应用容器。需要强制重建容器时执行：

```bash
./deploy.sh --force
```

如果 GHCR 镜像不是公开包，使用 CI 镜像文件发布。以 hd0526 为例：

```bash
APP_SERVICES="frontdoor background" \
APP_IMAGE=niffler-app:latest \
GH_REPO=ryfineZ/Niffler \
./scripts/deploy-ci-artifact.sh --host hd0526 --remote-dir /opt/niffler-app
```

这个脚本会下载最近一次成功的 `Build App Image` 工作流产物，把镜像文件传到服务器，服务器加载成 `niffler-app:latest`，再重启 `frontdoor` 和 `background`。

## 验证方式

- `bash -n deploy.sh`
- `bash -n scripts/deploy-ci-artifact.sh`
- GitHub Actions 的 `Build App Image` 工作流成功。
- 服务器执行发布脚本后，`docker compose ps` 显示应用容器健康。
