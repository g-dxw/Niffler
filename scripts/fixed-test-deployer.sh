#!/bin/bash

set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/g-dxw/Niffler.git}"
RELEASE_BRANCH="${RELEASE_BRANCH:-test}"
RELEASE_ROOT="${RELEASE_ROOT:-/opt/niffler-test-release}"
REMOTE_DIR="${REMOTE_DIR:-/opt/niffler-test}"
PUBLIC_HEALTH_URL="${PUBLIC_HEALTH_URL:-https://niffler-test.123.253.224.101.sslip.io/health}"
SOURCE_HEALTH_URL="${SOURCE_HEALTH_URL:-http://127.0.0.1:18084/health}"
DEPLOY_STATE_FILE="${DEPLOY_STATE_FILE:-.niffler-deployed-commit}"
APP_IMAGE_REPOSITORY="${APP_IMAGE_REPOSITORY:-niffler-app}"
BASELINE_COMMIT="${BASELINE_COMMIT:-1a2da6ff7dd3566fad7290aeac71088fa9c33b27}"
HEALTH_TIMEOUT_SECONDS="${HEALTH_TIMEOUT_SECONDS:-180}"
BACKUP_RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-7}"
ACTION="${1:-}"
TARGET_COMMIT=""

usage() {
    cat <<'EOF'
Usage: fixed-test-deployer.sh <receive|deploy> --target <sha>

receive reads an image tar from stdin into the fixed release inbox.
deploy validates and deploys the previously received exact-SHA image.
EOF
}

die() {
    echo "ERROR: $*" >&2
    exit 1
}

shift || true
while [ "$#" -gt 0 ]; do
    case "$1" in
        --target)
            [ "$#" -ge 2 ] || die "missing value for --target"
            TARGET_COMMIT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done

case "$ACTION" in
    receive|deploy) ;;
    *) usage >&2; die "action must be receive or deploy" ;;
esac
[[ "$TARGET_COMMIT" =~ ^[0-9a-f]{40}$ ]] || die "target must be a lowercase 40-character SHA"
[[ "$BASELINE_COMMIT" =~ ^[0-9a-f]{40}$ ]] || die "BASELINE_COMMIT must be a lowercase 40-character SHA"
[[ "$REMOTE_DIR" == /* && "$REMOTE_DIR" != "/" ]] || die "REMOTE_DIR must be an explicit application path"
[[ "$RELEASE_ROOT" == /* && "$RELEASE_ROOT" != "/" ]] || die "RELEASE_ROOT must be an explicit release path"

INCOMING_DIR="$RELEASE_ROOT/incoming"
STATE_DIR="$RELEASE_ROOT/state"
IMAGE_TAR="$INCOMING_DIR/niffler-app-$TARGET_COMMIT.tar"

mkdir -p "$INCOMING_DIR" "$STATE_DIR"

if [ "$ACTION" = "receive" ]; then
    partial="$IMAGE_TAR.partial"
    umask 077
    cat > "$partial"
    [ -s "$partial" ] || die "received image artifact is empty"
    mv "$partial" "$IMAGE_TAR"
    sha256sum "$IMAGE_TAR" | awk '{print "Received image artifact sha256=" $1}'
    exit 0
fi

[ -f "$REMOTE_DIR/docker-compose.yml" ] || die "compose file not found"
[ -s "$IMAGE_TAR" ] || die "received image artifact not found"
for command_name in git docker curl awk grep mktemp install cp mv date sed tr chmod sha256sum find; do
    command -v "$command_name" >/dev/null 2>&1 || die "required command not found: $command_name"
done

if docker compose version >/dev/null 2>&1; then
    DC=(docker compose)
else
    die "docker compose is not installed"
fi

RELEASE_REPO="$RELEASE_ROOT/git/Niffler.git"
mkdir -p "$RELEASE_ROOT/git"
if [ ! -d "$RELEASE_REPO" ]; then
    git init --bare "$RELEASE_REPO" >/dev/null
fi

REMOTE_TEST_COMMIT="$(git ls-remote "$REPO_URL" "refs/heads/$RELEASE_BRANCH" | awk 'NR == 1 {print $1}')"
REMOTE_MAIN_COMMIT="$(git ls-remote "$REPO_URL" refs/heads/main | awk 'NR == 1 {print $1}')"
[ "$TARGET_COMMIT" = "$REMOTE_TEST_COMMIT" ] || die "target is not current origin/$RELEASE_BRANCH"
[[ "$REMOTE_MAIN_COMMIT" =~ ^[0-9a-f]{40}$ ]] || die "unable to resolve origin/main"

git --git-dir="$RELEASE_REPO" fetch --quiet --force "$REPO_URL" \
    "+refs/heads/main:refs/heads/main" \
    "+refs/heads/$RELEASE_BRANCH:refs/heads/$RELEASE_BRANCH"
git --git-dir="$RELEASE_REPO" merge-base --is-ancestor "$REMOTE_MAIN_COMMIT" "$TARGET_COMMIT" \
    || die "test target does not contain current origin/main"
git --git-dir="$RELEASE_REPO" merge-base --is-ancestor "$BASELINE_COMMIT" "$TARGET_COMMIT" \
    || die "test target does not contain the original test deployment"
MAIN_TREE="$(git --git-dir="$RELEASE_REPO" rev-parse "${REMOTE_MAIN_COMMIT}^{tree}")"
TARGET_TREE="$(git --git-dir="$RELEASE_REPO" rev-parse "${TARGET_COMMIT}^{tree}")"
[ "$MAIN_TREE" = "$TARGET_TREE" ] || die "test target tree does not exactly match current origin/main"

STATE_PATH="$REMOTE_DIR/$DEPLOY_STATE_FILE"
CURRENT_COMMIT=""
if [ -s "$STATE_PATH" ]; then
    CURRENT_COMMIT="$(tr -d '[:space:]' < "$STATE_PATH")"
    [[ "$CURRENT_COMMIT" =~ ^[0-9a-f]{40}$ ]] || die "invalid deployed commit state"
    git --git-dir="$RELEASE_REPO" cat-file -e "${CURRENT_COMMIT}^{commit}" 2>/dev/null \
        || die "current deployed commit is unavailable from the release repository"
    git --git-dir="$RELEASE_REPO" merge-base --is-ancestor "$CURRENT_COMMIT" "$TARGET_COMMIT" \
        || die "target does not contain current deployed commit"
fi

TIMESTAMP="$(date -u +%Y%m%dT%H%M%SZ)"
TARGET_IMAGE="$APP_IMAGE_REPOSITORY:$TARGET_COMMIT"
ROLLBACK_IMAGE=""
if [ -n "$CURRENT_COMMIT" ] && docker image inspect "$APP_IMAGE_REPOSITORY:$CURRENT_COMMIT" >/dev/null 2>&1; then
    ROLLBACK_IMAGE="$APP_IMAGE_REPOSITORY:rollback-${CURRENT_COMMIT:0:12}-$TIMESTAMP"
    docker image tag "$APP_IMAGE_REPOSITORY:$CURRENT_COMMIT" "$ROLLBACK_IMAGE"
fi

docker load -i "$IMAGE_TAR"
docker image inspect "$TARGET_IMAGE" >/dev/null 2>&1 || die "artifact does not contain $TARGET_IMAGE"
IMAGE_REVISION="$(docker image inspect "$TARGET_IMAGE" --format '{{ index .Config.Labels "org.opencontainers.image.revision" }}')"
[ "$IMAGE_REVISION" = "$TARGET_COMMIT" ] || die "image revision does not match target"

cd "$REMOTE_DIR"
APP_CONTAINER="$("${DC[@]}" ps -q app)"
[ -n "$APP_CONTAINER" ] || die "current app container is not running"

TMP_DIR="$(mktemp -d "$STATE_DIR/preflight.XXXXXX")"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT
CURRENT_ENV_FILE="$TMP_DIR/app.env"
docker inspect "$APP_CONTAINER" --format '{{range .Config.Env}}{{println .}}{{end}}' > "$CURRENT_ENV_FILE"
chmod 0600 "$CURRENT_ENV_FILE"
docker run --rm \
    --network "container:$APP_CONTAINER" \
    --env-file "$CURRENT_ENV_FILE" \
    --entrypoint /usr/local/bin/aether-gateway \
    "$TARGET_IMAGE" \
    --check-postgres-migration-compatibility \
    || die "target image is incompatible with current PostgreSQL migrations"

BACKUP_DIR="$REMOTE_DIR/backups/postgres"
mkdir -p "$BACKUP_DIR"
BACKUP_PATH="$BACKUP_DIR/aether-before-${TARGET_COMMIT:0:12}-$TIMESTAMP.dump"
"${DC[@]}" exec -T postgres sh -c 'exec pg_dump -U "$POSTGRES_USER" -d "$POSTGRES_DB" -Fc' > "$BACKUP_PATH"
[ -s "$BACKUP_PATH" ] || die "PostgreSQL backup is empty"
find "$BACKUP_DIR" -type f -name 'aether-*.dump' -mtime "+$BACKUP_RETENTION_DAYS" -delete

ENV_PATH="$REMOTE_DIR/.env"
ENV_BACKUP="$STATE_DIR/env-$TIMESTAMP"
cp "$ENV_PATH" "$ENV_BACKUP"
set_app_image() {
    local image_ref="$1"
    if grep -q '^APP_IMAGE=' "$ENV_PATH"; then
        sed -i.bak "s|^APP_IMAGE=.*|APP_IMAGE=$image_ref|" "$ENV_PATH"
        rm -f "$ENV_PATH.bak"
    else
        printf '\nAPP_IMAGE=%s\n' "$image_ref" >> "$ENV_PATH"
    fi
}

rollback_app() {
    if [ -z "$ROLLBACK_IMAGE" ]; then
        echo "No rollback image is available; database backup: $BACKUP_PATH" >&2
        return 1
    fi
    echo "Deployment failed; restoring $ROLLBACK_IMAGE. Database was not rolled back." >&2
    set_app_image "$ROLLBACK_IMAGE"
    "${DC[@]}" up -d --no-build --force-recreate --wait --wait-timeout "$HEALTH_TIMEOUT_SECONDS" app || true
    return 1
}

set_app_image "$TARGET_IMAGE"
if ! "${DC[@]}" up -d --no-build --force-recreate --wait --wait-timeout "$HEALTH_TIMEOUT_SECONDS" app; then
    rollback_app
    exit 1
fi
if ! curl -fsS --max-time 15 "$SOURCE_HEALTH_URL" >/dev/null; then
    rollback_app
    exit 1
fi
if ! curl -fsS --max-time 20 "$PUBLIC_HEALTH_URL" >/dev/null; then
    rollback_app
    exit 1
fi

docker image tag "$TARGET_IMAGE" "$APP_IMAGE_REPOSITORY:latest"
STATE_TMP="$STATE_DIR/deployed-commit.tmp"
printf '%s\n' "$TARGET_COMMIT" > "$STATE_TMP"
install -m 0644 "$STATE_TMP" "$STATE_PATH"
rm -f "$STATE_TMP" "$IMAGE_TAR"
"${DC[@]}" ps
echo "Test deployment verified: $TARGET_COMMIT backup=$BACKUP_PATH"
