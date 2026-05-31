# CI 镜像发布流程

## 目标

生产发布不再在服务器上编译 Rust、构建前端或执行 `docker build`。所有应用镜像都由 GitHub Actions 构建并推送到 GHCR，服务器只负责拉取镜像并重启容器。

## 非目标

- 不改变 Postgres、Redis 的部署方式。
- 不把服务器密钥写入仓库。
- 不要求服务器安装 Rust、Node.js 或前端依赖。

## 行为变化

- 推送到 `main` 后，CI 会构建前端、构建 Linux amd64/arm64 的 `aether-gateway`，再打包成 `ghcr.io/ryfinez/niffler:main` 和 `ghcr.io/ryfinez/niffler:sha-xxxxxxx` 镜像。
- 手动触发 CI 时，可以额外填写一个镜像标签，方便发布指定测试版本。
- `deploy.sh` 不再使用 `Dockerfile.app.local`，也不再计算代码哈希。
- `deploy.sh` 只执行镜像拉取和 `docker compose up -d --no-build`。

## 影响范围

- GitHub Actions 新增主应用镜像构建流程。
- 服务器部署脚本改为拉取预构建镜像。
- Compose 默认镜像统一为 `ghcr.io/ryfinez/niffler`。

## 发布方式

服务器 `.env` 中设置：

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

## 验证方式

- `bash -n deploy.sh`
- GitHub Actions 的 `Build App Image` 工作流成功。
- 服务器执行 `./deploy.sh` 后，`docker compose ps` 显示应用容器健康。
