#!/bin/bash
# Download the app image built by GitHub Actions and load it on a server.
# The server does not build Rust, frontend assets, or Docker images.

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOW_NAME="${WORKFLOW_NAME:-Build App Image}"
GH_REPO="${GH_REPO:-ryfineZ/Niffler}"
BRANCH="${BRANCH:-main}"
GIT_REMOTE="${GIT_REMOTE:-origin}"
REQUIRED_BASE_BRANCH="${REQUIRED_BASE_BRANCH:-main}"
ARTIFACT_NAME="${ARTIFACT_NAME:-niffler-app-linux-amd64}"
REMOTE_TAR="${REMOTE_TAR:-/tmp/niffler-app-linux-amd64.tar}"
APP_IMAGE="${APP_IMAGE:-niffler-app:latest}"
APP_SERVICES="${APP_SERVICES:-app}"
SSH_OPTS="${SSH_OPTS:-}"
DEPLOY_STATE_FILE="${DEPLOY_STATE_FILE:-.niffler-deployed-commit}"
ANCESTRY_CHECK_SCRIPT="$SCRIPT_DIR/check-deploy-ancestry.sh"
FIXED_DEPLOYER_PATH="/opt/niffler-release/bin/deploy-production"
TEST_DEPLOYMENT=false
PUBLIC_HEALTH_URL=""

DEPLOY_HOST=""
REMOTE_DIR="/opt/niffler-app"
RUN_ID=""
COMMIT_REF=""
ALLOW_LATEST_FOR_LOCAL=false
ALLOW_ROLLBACK=false
RESTRICTED_ACTIONS=false

usage() {
    cat <<'EOF'
Usage: scripts/deploy-ci-artifact.sh --host <ssh-host> [options]

Options:
  --host <ssh-host>        SSH host, for example hd0526
  --remote-dir <path>      Remote compose directory, default /opt/niffler-app
  --run-id <id>            GitHub Actions run id for the artifact to deploy
  --commit <sha>           Git commit SHA; script resolves the successful workflow run for it
  --allow-latest-for-local Allow latest successful run selection. Only for local verification or temporary diagnostics.
  --allow-rollback         Explicitly deploy a commit that does not contain main or the current production commit.
  --restricted-actions     Use the fixed production SSH protocol. Rollback is not available.
  --test-deployment        Deploy the exact current test branch commit with the test policy.
  --public-health-url <url>
                           Public test base URL used for post-deploy health verification.
  -h, --help               Show help

Environment:
  APP_IMAGE                Image tag used by docker compose, default niffler-app:latest
  APP_SERVICES             Compose services to restart, default app
  GH_REPO                  GitHub repo used by gh, default ryfineZ/Niffler
  ARTIFACT_NAME            CI artifact name, default niffler-app-linux-amd64
  WORKFLOW_NAME            GitHub Actions workflow name, default Build App Image
  BRANCH                   Branch used when selecting latest successful run, default main
  GIT_REMOTE               Git remote used for ancestry checks, default origin
  REQUIRED_BASE_BRANCH     Branch every normal production release must contain, default main
  DEPLOY_STATE_FILE        Server file that records the deployed commit
  SSH_OPTS                 Extra ssh/scp options
EOF
}

require_option_value() {
    local option_name="$1"
    local option_value="${2:-}"
    if [ -z "$option_value" ] || [[ "$option_value" == --* ]]; then
        echo "Missing value for $option_name"
        usage
        exit 1
    fi
}

while [ $# -gt 0 ]; do
    case "$1" in
        --host)
            require_option_value "$1" "${2:-}"
            DEPLOY_HOST="${2:-}"
            shift 2
            ;;
        --remote-dir)
            require_option_value "$1" "${2:-}"
            REMOTE_DIR="${2:-}"
            shift 2
            ;;
        --run-id)
            require_option_value "$1" "${2:-}"
            RUN_ID="${2:-}"
            shift 2
            ;;
        --commit)
            require_option_value "$1" "${2:-}"
            COMMIT_REF="${2:-}"
            shift 2
            ;;
        --allow-latest-for-local)
            ALLOW_LATEST_FOR_LOCAL=true
            shift
            ;;
        --allow-rollback)
            ALLOW_ROLLBACK=true
            shift
            ;;
        --restricted-actions)
            RESTRICTED_ACTIONS=true
            shift
            ;;
        --test-deployment)
            TEST_DEPLOYMENT=true
            shift
            ;;
        --public-health-url)
            require_option_value "$1" "${2:-}"
            PUBLIC_HEALTH_URL="${2%/}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1"
            usage
            exit 1
            ;;
    esac
done

if [ -z "$DEPLOY_HOST" ]; then
    echo "Missing required option: --host"
    usage
    exit 1
fi

REQUIRED_COMMANDS=(gh ssh)
if [ "$RESTRICTED_ACTIONS" != true ]; then
    REQUIRED_COMMANDS+=(scp)
fi
for command_name in "${REQUIRED_COMMANDS[@]}"; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "Required command not found: $command_name"
        exit 1
    fi
done
SSH_OPTIONS=(-o BatchMode=yes)
if [ -n "$SSH_OPTS" ]; then
    read -r -a EXTRA_SSH_OPTIONS <<< "$SSH_OPTS"
    SSH_OPTIONS+=("${EXTRA_SSH_OPTIONS[@]}")
fi

if [ -n "$RUN_ID" ] && [ -n "$COMMIT_REF" ]; then
    echo "Use only one of --run-id or --commit"
    exit 1
fi
if [ "$RESTRICTED_ACTIONS" = true ] && [ "$ALLOW_ROLLBACK" = true ]; then
    echo "The restricted Actions protocol does not allow --allow-rollback."
    exit 1
fi
if [ "$TEST_DEPLOYMENT" = true ]; then
    if [ "$RESTRICTED_ACTIONS" = true ] || [ "$ALLOW_ROLLBACK" = true ]; then
        echo "Test deployment does not support restricted production mode or rollback overrides."
        exit 1
    fi
    if [[ "$PUBLIC_HEALTH_URL" != https://* ]]; then
        echo "Test deployment requires an https --public-health-url."
        exit 1
    fi
    BRANCH="test"
    REQUIRED_BASE_BRANCH="test"
    APP_SERVICES="app"
    FIXED_DEPLOYER_PATH="$REMOTE_DIR/bin/deploy-test"
fi

if [ -n "$COMMIT_REF" ]; then
    RUN_ID="$(gh run list \
        --repo "$GH_REPO" \
        --workflow "$WORKFLOW_NAME" \
        --commit "$COMMIT_REF" \
        --status success \
        --limit 1 \
        --json databaseId \
        --jq '.[0].databaseId // ""')"
    if [ -z "$RUN_ID" ] || [ "$RUN_ID" = "null" ]; then
        echo "No successful $WORKFLOW_NAME workflow run found for commit $COMMIT_REF"
        echo "Confirm the CI image workflow has completed successfully, or deploy with --run-id."
        exit 1
    fi
fi

if [ -z "$RUN_ID" ]; then
    if [ "$ALLOW_LATEST_FOR_LOCAL" != true ]; then
        echo "Production deployment requires --run-id or --commit."
        echo "Use --allow-latest-for-local only for local verification or temporary diagnostics."
        exit 1
    fi
    RUN_ID="$(gh run list \
        --repo "$GH_REPO" \
        --workflow "$WORKFLOW_NAME" \
        --branch "$BRANCH" \
        --status success \
        --limit 1 \
        --json databaseId \
        --jq '.[0].databaseId')"
fi

if [ -z "$RUN_ID" ] || [ "$RUN_ID" = "null" ]; then
    echo "No successful workflow run found for $WORKFLOW_NAME on $BRANCH"
    exit 1
fi

RUN_METADATA="$(gh run view "$RUN_ID" \
    --repo "$GH_REPO" \
    --json headSha,conclusion,workflowName \
    --jq '[.headSha, .conclusion, .workflowName] | @tsv')"
IFS=$'\t' read -r TARGET_COMMIT RUN_CONCLUSION RUN_WORKFLOW_NAME <<< "$RUN_METADATA"

if [ -z "$TARGET_COMMIT" ] || [ "$RUN_CONCLUSION" != "success" ]; then
    echo "Workflow run $RUN_ID is not a successful completed run."
    exit 1
fi

if [ "$RUN_WORKFLOW_NAME" != "$WORKFLOW_NAME" ]; then
    echo "Workflow run $RUN_ID belongs to '$RUN_WORKFLOW_NAME', expected '$WORKFLOW_NAME'."
    exit 1
fi

if [ ! -x "$ANCESTRY_CHECK_SCRIPT" ]; then
    echo "Deployment ancestry checker is not executable: $ANCESTRY_CHECK_SCRIPT"
    exit 1
fi

REPO_ROOT="$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel)"
git -C "$REPO_ROOT" fetch --quiet "$GIT_REMOTE" \
    "refs/heads/$REQUIRED_BASE_BRANCH:refs/remotes/$GIT_REMOTE/$REQUIRED_BASE_BRANCH"
REQUIRED_BASE_COMMIT="$(git -C "$REPO_ROOT" rev-parse "refs/remotes/$GIT_REMOTE/$REQUIRED_BASE_BRANCH")"

if ! git -C "$REPO_ROOT" cat-file -e "${TARGET_COMMIT}^{commit}" 2>/dev/null; then
    git -C "$REPO_ROOT" fetch --quiet "$GIT_REMOTE" "$TARGET_COMMIT"
fi

if [ "$RESTRICTED_ACTIONS" = true ]; then
    DEPLOYED_STATE="$(ssh "${SSH_OPTIONS[@]}" "$DEPLOY_HOST" status)"
else
    DEPLOYED_STATE="$(ssh "${SSH_OPTIONS[@]}" "$DEPLOY_HOST" bash -s -- \
        "$REMOTE_DIR" "$APP_IMAGE" "$DEPLOY_STATE_FILE" <<'REMOTE_STATE'
set -euo pipefail

REMOTE_DIR="$1"
APP_IMAGE="$2"
DEPLOY_STATE_FILE="$3"
STATE_PATH="$REMOTE_DIR/$DEPLOY_STATE_FILE"

if [ -s "$STATE_PATH" ]; then
    DEPLOYED_COMMIT="$(tr -d '[:space:]' < "$STATE_PATH")"
    if [[ "$DEPLOYED_COMMIT" =~ ^[0-9a-fA-F]{40}$ ]]; then
        printf '%s\n' "$DEPLOYED_COMMIT"
        exit 0
    fi
    printf '%s\n' "__invalid_state__"
    exit 0
fi

if ! docker image inspect "$APP_IMAGE" >/dev/null 2>&1; then
    printf '%s\n' "__none__"
    exit 0
fi

IMAGE_REVISION="$(docker image inspect "$APP_IMAGE" \
    --format '{{ index .Config.Labels "org.opencontainers.image.revision" }}' \
    2>/dev/null || true)"
if [[ "$IMAGE_REVISION" =~ ^[0-9a-fA-F]{40}$ ]]; then
    printf '%s\n' "$IMAGE_REVISION"
    exit 0
fi

IMAGE_TAG_COMMIT="$(docker image inspect "$APP_IMAGE" \
    --format '{{range .RepoTags}}{{println .}}{{end}}' \
    2>/dev/null \
    | awk -F: '$NF ~ /^[0-9a-fA-F]{40}$/ { print $NF; exit }')"
if [[ "$IMAGE_TAG_COMMIT" =~ ^[0-9a-fA-F]{40}$ ]]; then
    printf '%s\n' "$IMAGE_TAG_COMMIT"
    exit 0
fi

printf '%s\n' "__unknown__"
REMOTE_STATE
    )"
fi

CURRENT_DEPLOYED_COMMIT=""
case "$DEPLOYED_STATE" in
    __none__)
        echo ">>> No existing application image found; treating this as the first deployment."
        ;;
    __unknown__|__invalid_state__|"")
        if [ "$ALLOW_ROLLBACK" != true ]; then
            echo "Unable to determine the current production commit on $DEPLOY_HOST."
            echo "Repair $REMOTE_DIR/$DEPLOY_STATE_FILE or use --allow-rollback only for an intentional rollback."
            exit 1
        fi
        echo ">>> Current production commit is unavailable; explicit rollback override is active."
        ;;
    *)
        if [[ ! "$DEPLOYED_STATE" =~ ^[0-9a-fA-F]{40}$ ]]; then
            echo "Invalid current production commit returned by $DEPLOY_HOST: $DEPLOYED_STATE"
            exit 1
        fi
        CURRENT_DEPLOYED_COMMIT="$DEPLOYED_STATE"
        if ! git -C "$REPO_ROOT" cat-file -e "${CURRENT_DEPLOYED_COMMIT}^{commit}" 2>/dev/null; then
            git -C "$REPO_ROOT" fetch --quiet "$GIT_REMOTE" "$CURRENT_DEPLOYED_COMMIT"
        fi
        ;;
esac

ANCESTRY_ARGS=(
    --repo "$REPO_ROOT"
    --target "$TARGET_COMMIT"
    --required-base "$REQUIRED_BASE_COMMIT"
)
if [ -n "$CURRENT_DEPLOYED_COMMIT" ] && [ "$TEST_DEPLOYMENT" != true ]; then
    ANCESTRY_ARGS+=(--current "$CURRENT_DEPLOYED_COMMIT")
fi
if [ "$ALLOW_ROLLBACK" = true ]; then
    ANCESTRY_ARGS+=(--allow-rollback)
fi

"$ANCESTRY_CHECK_SCRIPT" "${ANCESTRY_ARGS[@]}"
echo ">>> Release commit verified: $TARGET_COMMIT"

TMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo ">>> Downloading CI image artifact from run $RUN_ID..."
gh run download "$RUN_ID" --repo "$GH_REPO" --name "$ARTIFACT_NAME" --dir "$TMP_DIR"

IMAGE_TAR="$TMP_DIR/niffler-app-linux-amd64.tar"
if [ ! -f "$IMAGE_TAR" ]; then
    echo "Artifact did not contain expected file: niffler-app-linux-amd64.tar"
    find "$TMP_DIR" -maxdepth 2 -type f -print
    exit 1
fi

if [ "$RESTRICTED_ACTIONS" = true ]; then
    if command -v sha256sum >/dev/null 2>&1; then
        LOCAL_IMAGE_SHA256="$(sha256sum "$IMAGE_TAR" | awk '{print $1}')"
    else
        LOCAL_IMAGE_SHA256="$(shasum -a 256 "$IMAGE_TAR" | awk '{print $1}')"
    fi
    echo ">>> Uploading image tar through the restricted production protocol..."
    UPLOAD_RESULT="$(
        # TARGET_COMMIT is a validated lowercase 40-character SHA.
        # shellcheck disable=SC2029
        ssh "${SSH_OPTIONS[@]}" "$DEPLOY_HOST" \
            upload "$TARGET_COMMIT" < "$IMAGE_TAR"
    )"
    if [ "$UPLOAD_RESULT" != "uploaded_sha256=$LOCAL_IMAGE_SHA256" ]; then
        echo "Uploaded artifact SHA-256 does not match the local artifact." >&2
        exit 1
    fi
    echo ">>> Handing the verified artifact to the restricted production deployer..."
    # TARGET_COMMIT is a validated lowercase 40-character SHA.
    # shellcheck disable=SC2029
    ssh "${SSH_OPTIONS[@]}" "$DEPLOY_HOST" deploy "$TARGET_COMMIT"
else
    echo ">>> Uploading image tar to $DEPLOY_HOST:$REMOTE_TAR..."
    scp "${SSH_OPTIONS[@]}" "$IMAGE_TAR" "$DEPLOY_HOST:$REMOTE_TAR"

    echo ">>> Handing the verified artifact to the fixed production deployer..."
    read -r -a LOCAL_SERVICES <<< "$APP_SERVICES"
    REMOTE_DEPLOY_ARGS=(
        --image-tar "$REMOTE_TAR"
        --target "$TARGET_COMMIT"
        --remote-dir "$REMOTE_DIR"
        --state-file "$DEPLOY_STATE_FILE"
    )
    for service in "${LOCAL_SERVICES[@]}"; do
        REMOTE_DEPLOY_ARGS+=(--service "$service")
    done
    if [ "$ALLOW_ROLLBACK" = true ]; then
        REMOTE_DEPLOY_ARGS+=(--allow-rollback)
    fi
    REMOTE_COMMAND=("$FIXED_DEPLOYER_PATH" "${REMOTE_DEPLOY_ARGS[@]}")
    if [ "$TEST_DEPLOYMENT" = true ]; then
        REMOTE_DEPLOY_ARGS+=(
            --required-branch test
            --migration-context-service app
            --allow-non-ancestor-current
            --source-health-url http://127.0.0.1:18084/_gateway/health
            --public-health-url "$PUBLIC_HEALTH_URL/_gateway/health"
        )
        REMOTE_COMMAND=(
            env "RELEASE_ROOT=$REMOTE_DIR/.release"
            "$FIXED_DEPLOYER_PATH" "${REMOTE_DEPLOY_ARGS[@]}"
        )
    fi

    ssh "${SSH_OPTIONS[@]}" "$DEPLOY_HOST" bash -s -- \
        "${REMOTE_COMMAND[@]}" <<'REMOTE_SCRIPT'
set -euo pipefail
exec "$@"
REMOTE_SCRIPT
fi

echo ">>> Done."
