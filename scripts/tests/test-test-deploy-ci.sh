#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_SCRIPT="$SCRIPT_DIR/../deploy-ci-artifact.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

TARGET_COMMIT="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
CURRENT_COMMIT="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
FAKE_BIN="$TEST_ROOT/bin"
SSH_LOG="$TEST_ROOT/ssh.log"
SCP_LOG="$TEST_ROOT/scp.log"
mkdir -p "$FAKE_BIN"

cat > "$FAKE_BIN/git" <<'FAKE_GIT'
#!/bin/bash
set -euo pipefail
case "$*" in
    *"rev-parse --show-toplevel")
        printf '%s\n' "$FAKE_REPO_ROOT"
        ;;
    *"rev-parse refs/remotes/origin/test")
        printf '%s\n' "$FAKE_TARGET_COMMIT"
        ;;
    *"fetch --quiet origin refs/heads/test:"*|*"cat-file -e "*|*"merge-base --is-ancestor "*)
        ;;
    *)
        echo "unexpected git command: $*" >&2
        exit 1
        ;;
esac
FAKE_GIT

cat > "$FAKE_BIN/gh" <<'FAKE_GH'
#!/bin/bash
set -euo pipefail
case "${1:-} ${2:-}" in
    "run view")
        printf '%s\t%s\t%s\t%s\n' \
            "$FAKE_TARGET_COMMIT" "in_progress" "__pending__" \
            "${FAKE_RUN_WORKFLOW_NAME:-__missing__}"
        ;;
    "run download")
        download_dir=""
        while [ "$#" -gt 0 ]; do
            if [ "$1" = "--dir" ]; then
                download_dir="$2"
                break
            fi
            shift
        done
        test -n "$download_dir"
        printf '%s' "test-image" > "$download_dir/niffler-app-linux-amd64.tar"
        ;;
    *)
        echo "unexpected gh command: $*" >&2
        exit 1
        ;;
esac
FAKE_GH

cat > "$FAKE_BIN/ssh" <<'FAKE_SSH'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_SSH_LOG"
if [[ "$*" == *"/opt/niffler-test/bin/deploy-test"* ]]; then
    exit 0
fi
printf '%s\n' "$FAKE_CURRENT_COMMIT"
FAKE_SSH

cat > "$FAKE_BIN/scp" <<'FAKE_SCP'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_SCP_LOG"
FAKE_SCP

chmod +x "$FAKE_BIN/git" "$FAKE_BIN/gh" "$FAKE_BIN/ssh" "$FAKE_BIN/scp"

export PATH="$FAKE_BIN:$PATH"
FAKE_REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
export FAKE_REPO_ROOT
export FAKE_TARGET_COMMIT="$TARGET_COMMIT"
export FAKE_CURRENT_COMMIT="$CURRENT_COMMIT"
export FAKE_SSH_LOG="$SSH_LOG"
export FAKE_SCP_LOG="$SCP_LOG"

GH_REPO="ryfineZ/Niffler" \
GITHUB_ACTIONS=true \
GITHUB_RUN_ID="123456" \
GITHUB_SHA="$TARGET_COMMIT" \
GITHUB_WORKFLOW="Build App Image" \
GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/app-image.yml@refs/heads/test" \
GITHUB_REF_NAME="test" \
    bash "$DEPLOY_SCRIPT" \
        --host "deploy@example.test" \
    --remote-dir "/opt/niffler-test" \
    --run-id "123456" \
    --test-deployment \
    --source-health-url "http://127.0.0.1:18084/_gateway/health" \
    --public-health-url "https://test.example.test/"

grep -Fq -- "/opt/niffler-test/bin/deploy-test" "$SSH_LOG"
grep -Fq -- "RELEASE_ROOT=/opt/niffler-test/.release" "$SSH_LOG"
grep -Fq -- "--required-branch test" "$SSH_LOG"
grep -Fq -- "--migration-context-service app" "$SSH_LOG"
grep -Fq -- "--bootstrap-migration-context" "$SSH_LOG"
grep -Fq -- "--allow-non-ancestor-current" "$SSH_LOG"
grep -Fq -- "--service app" "$SSH_LOG"
grep -Fq -- "--source-health-url http://127.0.0.1:18084/_gateway/health" "$SSH_LOG"
grep -Fq -- "--public-health-url https://test.example.test/_gateway/health" "$SSH_LOG"
grep -Fq -- "deploy@example.test:/tmp/niffler-app-linux-amd64.tar" "$SCP_LOG"

if GH_REPO="ryfineZ/Niffler" \
    FAKE_RUN_WORKFLOW_NAME="Build App Image" \
    GITHUB_ACTIONS=true \
    GITHUB_RUN_ID="654321" \
    GITHUB_SHA="$TARGET_COMMIT" \
    GITHUB_WORKFLOW="Build App Image" \
    GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/app-image.yml@refs/heads/test" \
    GITHUB_REF_NAME="test" \
        bash "$DEPLOY_SCRIPT" \
            --host "deploy@example.test" \
            --remote-dir "/opt/niffler-test" \
            --run-id "123456" \
            --test-deployment \
            --source-health-url "http://127.0.0.1:18084/_gateway/health" \
            --public-health-url "https://test.example.test" \
            >"$TEST_ROOT/unrelated-run.out" 2>&1; then
    echo "unrelated active workflow run unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "is not a successful completed run" "$TEST_ROOT/unrelated-run.out"

if GH_REPO="ryfineZ/Niffler" \
    FAKE_RUN_WORKFLOW_NAME="Build App Image" \
    GITHUB_ACTIONS=true \
    GITHUB_RUN_ID="123456" \
    GITHUB_SHA="$TARGET_COMMIT" \
    GITHUB_WORKFLOW="Build App Image" \
    GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/app-image.yml@refs/heads/test" \
    GITHUB_REF_NAME="main" \
        bash "$DEPLOY_SCRIPT" \
            --host "deploy@example.test" \
            --remote-dir "/opt/niffler-test" \
            --run-id "123456" \
            --test-deployment \
            --source-health-url "http://127.0.0.1:18084/_gateway/health" \
            --public-health-url "https://test.example.test" \
            >"$TEST_ROOT/non-test-branch.out" 2>&1; then
    echo "active workflow run outside test unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "is not a successful completed run" "$TEST_ROOT/non-test-branch.out"

if GH_REPO="ryfineZ/Niffler" \
    FAKE_RUN_WORKFLOW_NAME="Unexpected Workflow" \
    GITHUB_ACTIONS=true \
    GITHUB_RUN_ID="123456" \
    GITHUB_SHA="$TARGET_COMMIT" \
    GITHUB_WORKFLOW="Build App Image" \
    GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/app-image.yml@refs/heads/test" \
    GITHUB_REF_NAME="test" \
        bash "$DEPLOY_SCRIPT" \
            --host "deploy@example.test" \
            --remote-dir "/opt/niffler-test" \
            --run-id "123456" \
            --test-deployment \
            --source-health-url "http://127.0.0.1:18084/_gateway/health" \
            --public-health-url "https://test.example.test" \
            >"$TEST_ROOT/wrong-workflow-name.out" 2>&1; then
    echo "incorrect workflow name unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "belongs to 'Unexpected Workflow'" "$TEST_ROOT/wrong-workflow-name.out"

if GH_REPO="ryfineZ/Niffler" \
    FAKE_RUN_WORKFLOW_NAME="Build App Image" \
    GITHUB_ACTIONS=true \
    GITHUB_RUN_ID="123456" \
    GITHUB_SHA="$TARGET_COMMIT" \
    GITHUB_WORKFLOW="Build App Image" \
    GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/other.yml@refs/heads/test" \
    GITHUB_REF_NAME="test" \
        bash "$DEPLOY_SCRIPT" \
            --host "deploy@example.test" \
            --remote-dir "/opt/niffler-test" \
            --run-id "123456" \
            --test-deployment \
            --source-health-url "http://127.0.0.1:18084/_gateway/health" \
            --public-health-url "https://test.example.test" \
            >"$TEST_ROOT/wrong-workflow-ref.out" 2>&1; then
    echo "incorrect workflow ref unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "is not a successful completed run" "$TEST_ROOT/wrong-workflow-ref.out"

if bash "$DEPLOY_SCRIPT" \
    --host "deploy@example.test" \
    --remote-dir "/opt/niffler-test" \
    --run-id "123456" \
    --test-deployment \
    --public-health-url "http://test.example.test" >"$TEST_ROOT/http-url.out" 2>&1; then
    echo "insecure public test URL unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "requires an https --public-health-url" "$TEST_ROOT/http-url.out"

if bash "$DEPLOY_SCRIPT" \
    --host "deploy@example.test" \
    --remote-dir "/opt/not-niffler-test" \
    --run-id "123456" \
    --test-deployment \
    --source-health-url "http://127.0.0.1:18084/_gateway/health" \
    --public-health-url "https://test.example.test" >"$TEST_ROOT/remote-dir.out" 2>&1; then
    echo "unexpected test deployment remote directory succeeded" >&2
    exit 1
fi
grep -Fq "requires --remote-dir /opt/niffler-test" "$TEST_ROOT/remote-dir.out"

if bash "$DEPLOY_SCRIPT" \
    --host "deploy@example.test" \
    --remote-dir "/opt/niffler-test" \
    --run-id "123456" \
    --test-deployment \
    --source-health-url "https://source.example.test/_gateway/health" \
    --public-health-url "https://test.example.test" >"$TEST_ROOT/source-url.out" 2>&1; then
    echo "non-local source test URL unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "requires a local --source-health-url" "$TEST_ROOT/source-url.out"

echo "test CI deployment tests passed"
