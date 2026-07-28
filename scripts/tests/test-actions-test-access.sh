#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SSH_COMMAND="$SCRIPT_DIR/../actions-test-ssh-command.sh"
DEPLOY_ENTRYPOINT="$SCRIPT_DIR/../actions-test-deploy.sh"
INSTALLER="$SCRIPT_DIR/../install-actions-test-access.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

for script_path in "$SSH_COMMAND" "$DEPLOY_ENTRYPOINT" "$INSTALLER"; do
    test -f "$script_path"
    bash -n "$script_path"
done
grep -Fq 'DEPLOY_USER="niffler-test-deploy"' "$INSTALLER"
grep -Fq 'RELEASE_BIN="/opt/niffler-test-release/bin"' "$INSTALLER"
grep -Fq 'restrict,command=' "$INSTALLER"
grep -Fq 'NOPASSWD:NOSETENV:' "$INSTALLER"
if grep -Eq 'usermod[[:space:]].*-aG[[:space:]]+docker' "$INSTALLER"; then
    echo "test deploy user must not be added to the docker group" >&2
    exit 1
fi

CURRENT_USER="$(id -un)"
CURRENT_UID="$(id -u)"
TARGET_COMMIT="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
CURRENT_COMMIT="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
UPLOAD_DIR="$TEST_ROOT/uploads"
INCOMING_DIR="$TEST_ROOT/incoming"
STATE_FILE="$TEST_ROOT/.niffler-deployed-commit"
REMOTE_DIR="$TEST_ROOT/app"
RELEASE_ROOT="$TEST_ROOT/release"
FAKE_BIN="$TEST_ROOT/bin"
SUDO_LOG="$TEST_ROOT/sudo.log"
DEPLOY_LOG="$TEST_ROOT/deploy.log"
mkdir -p "$UPLOAD_DIR" "$INCOMING_DIR" "$REMOTE_DIR" "$FAKE_BIN"
printf '%s\n' "$CURRENT_COMMIT" > "$STATE_FILE"

cat > "$FAKE_BIN/sudo" <<'FAKE_SUDO'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_SUDO_LOG"
if [ "${3:-}" = "status" ]; then
    printf '%s\n' "$FAKE_DEPLOYED_COMMIT"
fi
FAKE_SUDO

cat > "$FAKE_BIN/fixed-deployer" <<'FAKE_DEPLOYER'
#!/bin/bash
set -euo pipefail
printf 'RELEASE_ROOT=%s\n' "${RELEASE_ROOT:-}" > "$FAKE_DEPLOY_LOG"
printf '%s\n' "$*" >> "$FAKE_DEPLOY_LOG"
FAKE_DEPLOYER

chmod +x "$FAKE_BIN/sudo" "$FAKE_BIN/fixed-deployer"

export ACTIONS_TEST_MODE=true
export ACTIONS_DEPLOY_USER="$CURRENT_USER"
export ACTIONS_UPLOAD_DIR="$UPLOAD_DIR"
export ACTIONS_PRIVILEGED_ENTRYPOINT="$TEST_ROOT/deploy-from-actions"
export ACTIONS_MAX_UPLOAD_BYTES=1048576
export FAKE_SUDO_LOG="$SUDO_LOG"
export FAKE_DEPLOYED_COMMIT="$CURRENT_COMMIT"
export PATH="$FAKE_BIN:$PATH"

status_output="$(SSH_ORIGINAL_COMMAND=status bash "$SSH_COMMAND")"
test "$status_output" = "$CURRENT_COMMIT"
grep -Fq -- "-n $TEST_ROOT/deploy-from-actions status" "$SUDO_LOG"

if SSH_ORIGINAL_COMMAND=shell bash "$SSH_COMMAND" >"$TEST_ROOT/unknown.out" 2>&1; then
    echo "unknown test SSH command unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "unsupported test SSH command" "$TEST_ROOT/unknown.out"

upload_output="$(
    printf 'test-image' \
        | SSH_ORIGINAL_COMMAND="upload $TARGET_COMMIT" bash "$SSH_COMMAND"
)"
test -f "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
test "$(stat -c '%a' "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar" 2>/dev/null \
    || stat -f '%Lp' "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar")" = "600"
if command -v sha256sum >/dev/null 2>&1; then
    expected_hash="$(printf 'test-image' | sha256sum | awk '{print $1}')"
else
    expected_hash="$(printf 'test-image' | shasum -a 256 | awk '{print $1}')"
fi
test "$upload_output" = "uploaded_sha256=$expected_hash"

SSH_ORIGINAL_COMMAND="deploy $TARGET_COMMIT" bash "$SSH_COMMAND"
tail -n 1 "$SUDO_LOG" \
    | grep -Fq -- "-n $TEST_ROOT/deploy-from-actions deploy $TARGET_COMMIT"

export ACTIONS_DEPLOY_UID="$CURRENT_UID"
export ACTIONS_INCOMING_DIR="$INCOMING_DIR"
export ACTIONS_STATE_FILE="$STATE_FILE"
export ACTIONS_FIXED_DEPLOYER="$FAKE_BIN/fixed-deployer"
export ACTIONS_REMOTE_DIR="$REMOTE_DIR"
export ACTIONS_RELEASE_ROOT="$RELEASE_ROOT"
export ACTIONS_SOURCE_HEALTH_URL="http://127.0.0.1:18084/_gateway/health"
export ACTIONS_PUBLIC_HEALTH_URL="https://test.example.test/_gateway/health"
export FAKE_DEPLOY_LOG="$DEPLOY_LOG"

wrapper_status="$(SUDO_USER="$CURRENT_USER" bash "$DEPLOY_ENTRYPOINT" status)"
test "$wrapper_status" = "$CURRENT_COMMIT"

printf 'verified-image' > "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
chmod 600 "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
SUDO_USER="$CURRENT_USER" bash "$DEPLOY_ENTRYPOINT" deploy "$TARGET_COMMIT"
test ! -e "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
grep -Fq "RELEASE_ROOT=$RELEASE_ROOT" "$DEPLOY_LOG"
grep -Fq -- "--target $TARGET_COMMIT" "$DEPLOY_LOG"
grep -Fq -- "--remote-dir $REMOTE_DIR" "$DEPLOY_LOG"
grep -Fq -- "--service app" "$DEPLOY_LOG"
grep -Fq -- "--required-branch test" "$DEPLOY_LOG"
grep -Fq -- "--migration-context-service app" "$DEPLOY_LOG"
grep -Fq -- "--bootstrap-migration-context" "$DEPLOY_LOG"
grep -Fq -- "--allow-non-ancestor-current" "$DEPLOY_LOG"
grep -Fq -- "--source-health-url $ACTIONS_SOURCE_HEALTH_URL" "$DEPLOY_LOG"
grep -Fq -- "--public-health-url $ACTIONS_PUBLIC_HEALTH_URL" "$DEPLOY_LOG"

if SUDO_USER=unexpected-user bash "$DEPLOY_ENTRYPOINT" status \
    >"$TEST_ROOT/wrong-user.out" 2>&1; then
    echo "unexpected sudo caller succeeded" >&2
    exit 1
fi
grep -Fq "must be invoked by $CURRENT_USER" "$TEST_ROOT/wrong-user.out"

echo "actions test access tests passed"
