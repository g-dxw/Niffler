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
GH_LOG="$TEST_ROOT/gh.log"
ARTIFACT_FILE="$TEST_ROOT/niffler-app-linux-amd64.tar"
mkdir -p "$FAKE_BIN"
printf '%s' "test-image" > "$ARTIFACT_FILE"

cat > "$FAKE_BIN/git" <<'FAKE_GIT'
#!/bin/bash
set -euo pipefail
case "$*" in
    *"rev-parse --show-toplevel")
        printf '%s\n' "$FAKE_REPO_ROOT"
        ;;
    *"rev-parse refs/remotes/origin/test")
        printf '%s\n' "${FAKE_REQUIRED_BASE_COMMIT:-$FAKE_TARGET_COMMIT}"
        ;;
    *"merge-base --is-ancestor "*)
        if [ "${FAKE_REJECT_ANCESTRY:-false}" = true ]; then
            exit 1
        fi
        ;;
    *"fetch --quiet origin refs/heads/test:"*|*"cat-file -e "*)
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
printf '%s\n' "$*" >> "$FAKE_GH_LOG"
echo "test artifact deployment must not call gh: $*" >&2
exit 1
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
export FAKE_GH_LOG="$GH_LOG"

run_artifact_deploy() {
    env \
        GH_REPO="ryfineZ/Niffler" \
        GITHUB_ACTIONS="${TEST_GITHUB_ACTIONS:-true}" \
        GITHUB_REPOSITORY="${TEST_GITHUB_REPOSITORY:-ryfineZ/Niffler}" \
        GITHUB_RUN_ID="${TEST_GITHUB_RUN_ID:-123456}" \
        GITHUB_SHA="${TEST_GITHUB_SHA:-$TARGET_COMMIT}" \
        GITHUB_WORKFLOW="${TEST_GITHUB_WORKFLOW:-Build App Image}" \
        GITHUB_WORKFLOW_REF="${TEST_GITHUB_WORKFLOW_REF:-ryfineZ/Niffler/.github/workflows/app-image.yml@refs/heads/test}" \
        GITHUB_REF="${TEST_GITHUB_REF:-refs/heads/test}" \
        GITHUB_REF_NAME="${TEST_GITHUB_REF_NAME:-test}" \
        FAKE_REQUIRED_BASE_COMMIT="${FAKE_REQUIRED_BASE_COMMIT:-$TARGET_COMMIT}" \
        FAKE_REJECT_ANCESTRY="${FAKE_REJECT_ANCESTRY:-false}" \
        bash "$DEPLOY_SCRIPT" \
        --host "deploy@example.test" \
        --remote-dir "/opt/niffler-test" \
        --artifact-file "${TEST_ARTIFACT_FILE:-$ARTIFACT_FILE}" \
        --test-deployment \
        --source-health-url "http://127.0.0.1:18084/_gateway/health" \
        --public-health-url "https://test.example.test/" \
        "$@"
}

run_artifact_deploy

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
test ! -s "$GH_LOG"

if TEST_GITHUB_WORKFLOW_REF="ryfineZ/Niffler/.github/workflows/other.yml@refs/heads/test" \
    run_artifact_deploy >"$TEST_ROOT/wrong-workflow-context.out" 2>&1; then
    echo "incorrect workflow context unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "requires the trusted Build App Image workflow context" "$TEST_ROOT/wrong-workflow-context.out"

if TEST_GITHUB_WORKFLOW="Unexpected Workflow" \
    run_artifact_deploy >"$TEST_ROOT/wrong-workflow-name.out" 2>&1; then
    echo "incorrect workflow name unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "requires the trusted Build App Image workflow context" "$TEST_ROOT/wrong-workflow-name.out"

assert_invalid_context() {
    local case_name="$1"
    local variable_name="$2"
    local invalid_value="$3"
    local output_file="$TEST_ROOT/invalid-$case_name-context.out"

    export "$variable_name=$invalid_value"
    if run_artifact_deploy >"$output_file" 2>&1; then
        echo "invalid $case_name workflow context unexpectedly succeeded" >&2
        exit 1
    fi
    unset "$variable_name"
    grep -Fq "requires the trusted Build App Image workflow context" "$output_file"
}

for context_case in \
    "actions|TEST_GITHUB_ACTIONS|false" \
    "repository|TEST_GITHUB_REPOSITORY|other/Niffler" \
    "ref|TEST_GITHUB_REF|refs/heads/main" \
    "ref-name|TEST_GITHUB_REF_NAME|main" \
    "run-id|TEST_GITHUB_RUN_ID|not-a-run-id" \
    "sha|TEST_GITHUB_SHA|not-a-commit"; do
    IFS='|' read -r case_name variable_name invalid_value <<< "$context_case"
    assert_invalid_context "$case_name" "$variable_name" "$invalid_value"
done

if run_artifact_deploy --run-id "123456" >"$TEST_ROOT/mixed-artifact-source.out" 2>&1; then
    echo "mixed artifact source unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "Use --artifact-file without --run-id" "$TEST_ROOT/mixed-artifact-source.out"

if run_artifact_deploy --commit "$TARGET_COMMIT" >"$TEST_ROOT/mixed-artifact-commit.out" 2>&1; then
    echo "artifact and commit sources unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "Use --artifact-file without --run-id" "$TEST_ROOT/mixed-artifact-commit.out"

if run_artifact_deploy --allow-latest-for-local >"$TEST_ROOT/mixed-artifact-latest.out" 2>&1; then
    echo "artifact and latest sources unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "Use --artifact-file without --run-id" "$TEST_ROOT/mixed-artifact-latest.out"

if TEST_ARTIFACT_FILE="relative-artifact.tar" \
    run_artifact_deploy >"$TEST_ROOT/relative-artifact.out" 2>&1; then
    echo "relative artifact path unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "requires an absolute path" "$TEST_ROOT/relative-artifact.out"

if TEST_ARTIFACT_FILE="$TEST_ROOT/missing-artifact.tar" \
    run_artifact_deploy >"$TEST_ROOT/missing-artifact.out" 2>&1; then
    echo "missing artifact unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "artifact is missing or empty" "$TEST_ROOT/missing-artifact.out"

EMPTY_ARTIFACT_FILE="$TEST_ROOT/empty-artifact.tar"
: > "$EMPTY_ARTIFACT_FILE"
if TEST_ARTIFACT_FILE="$EMPTY_ARTIFACT_FILE" \
    run_artifact_deploy >"$TEST_ROOT/empty-artifact.out" 2>&1; then
    echo "empty artifact unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "artifact is missing or empty" "$TEST_ROOT/empty-artifact.out"

: > "$SSH_LOG"
: > "$SCP_LOG"
if FAKE_REQUIRED_BASE_COMMIT="$CURRENT_COMMIT" FAKE_REJECT_ANCESTRY=true \
    run_artifact_deploy >"$TEST_ROOT/stale-test-run.out" 2>&1; then
    echo "stale test workflow artifact unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "does not contain required base $CURRENT_COMMIT" "$TEST_ROOT/stale-test-run.out"
if grep -Fq -- "/opt/niffler-test/bin/deploy-test" "$SSH_LOG"; then
    echo "stale test workflow reached the remote deployer" >&2
    exit 1
fi
test ! -s "$SCP_LOG"

if bash "$DEPLOY_SCRIPT" \
    --host "deploy@example.test" \
    --artifact-file "$ARTIFACT_FILE" >"$TEST_ROOT/non-test-artifact.out" 2>&1; then
    echo "non-test artifact deployment unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "only available with --test-deployment" "$TEST_ROOT/non-test-artifact.out"

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
