#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SSH_COMMAND="$SCRIPT_DIR/../actions-production-ssh-command.sh"
DEPLOY_ENTRYPOINT="$SCRIPT_DIR/../actions-production-deploy.sh"
INSTALLER="$SCRIPT_DIR/../install-actions-production-access.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

for script_path in "$SSH_COMMAND" "$DEPLOY_ENTRYPOINT" "$INSTALLER"; do
    if [ ! -f "$script_path" ]; then
        echo "required production access script is missing: $script_path" >&2
        exit 1
    fi
    bash -n "$script_path"
done

CURRENT_USER="$(id -un)"
CURRENT_UID="$(id -u)"
TARGET_COMMIT="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
OTHER_COMMIT="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
UPLOAD_DIR="$TEST_ROOT/uploads"
INCOMING_DIR="$TEST_ROOT/incoming"
STATE_FILE="$TEST_ROOT/.niffler-deployed-commit"
FAKE_BIN="$TEST_ROOT/bin"
SUDO_LOG="$TEST_ROOT/sudo.log"
DEPLOY_LOG="$TEST_ROOT/deploy.log"
mkdir -p "$UPLOAD_DIR" "$INCOMING_DIR" "$FAKE_BIN"
printf '%s\n' "$OTHER_COMMIT" > "$STATE_FILE"

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
printf '%s\n' "$*" > "$FAKE_DEPLOY_LOG"
FAKE_DEPLOYER

chmod +x "$FAKE_BIN/sudo" "$FAKE_BIN/fixed-deployer"

export ACTIONS_DEPLOY_USER="$CURRENT_USER"
export ACTIONS_UPLOAD_DIR="$UPLOAD_DIR"
export ACTIONS_PRIVILEGED_ENTRYPOINT="$TEST_ROOT/deploy-from-actions"
export ACTIONS_MAX_UPLOAD_BYTES=1048576
export ACTIONS_TEST_MODE=true
export FAKE_SUDO_LOG="$SUDO_LOG"
export FAKE_DEPLOYED_COMMIT="$OTHER_COMMIT"
export PATH="$FAKE_BIN:$PATH"

status_output="$(
    SSH_ORIGINAL_COMMAND="status" \
        bash "$SSH_COMMAND"
)"
test "$status_output" = "$OTHER_COMMIT"
grep -Fq -- "-n $TEST_ROOT/deploy-from-actions status" "$SUDO_LOG"

if SSH_ORIGINAL_COMMAND="shell" bash "$SSH_COMMAND" >"$TEST_ROOT/unknown.out" 2>&1; then
    echo "unknown SSH command unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "unsupported production SSH command" "$TEST_ROOT/unknown.out"

if SSH_CONNECTION="127.0.0.1 10000 127.0.0.1 22" \
    SSH_ORIGINAL_COMMAND="status" \
        bash "$SSH_COMMAND" >"$TEST_ROOT/ssh-env-override.out" 2>&1; then
    echo "SSH environment unexpectedly enabled test overrides" >&2
    exit 1
fi
grep -Fq "must run as niffler-deploy" "$TEST_ROOT/ssh-env-override.out"

if SSH_ORIGINAL_COMMAND="upload not-a-commit" \
    bash "$SSH_COMMAND" >"$TEST_ROOT/invalid-upload.out" 2>&1; then
    echo "invalid upload commit unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "invalid target commit" "$TEST_ROOT/invalid-upload.out"

upload_output="$(
    printf 'valid-image' \
        | SSH_ORIGINAL_COMMAND="upload $TARGET_COMMIT" bash "$SSH_COMMAND"
)"
test -f "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
test "$(stat -c '%a' "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar" 2>/dev/null \
    || stat -f '%Lp' "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar")" = "600"
if command -v sha256sum >/dev/null 2>&1; then
    expected_upload_hash="$(
        printf 'valid-image' | sha256sum | awk '{print $1}'
    )"
else
    expected_upload_hash="$(
        printf 'valid-image' | shasum -a 256 | awk '{print $1}'
    )"
fi
test "$upload_output" = "uploaded_sha256=$expected_upload_hash"

export ACTIONS_MAX_UPLOAD_BYTES=4
if printf '12345' \
    | SSH_ORIGINAL_COMMAND="upload $OTHER_COMMIT" \
        bash "$SSH_COMMAND" >"$TEST_ROOT/oversize.out" 2>&1; then
    echo "oversize upload unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "uploaded image exceeds" "$TEST_ROOT/oversize.out"
test ! -e "$UPLOAD_DIR/niffler-app-$OTHER_COMMIT.tar"
export ACTIONS_MAX_UPLOAD_BYTES=1048576

SSH_ORIGINAL_COMMAND="deploy $TARGET_COMMIT" bash "$SSH_COMMAND"
tail -n 1 "$SUDO_LOG" \
    | grep -Fq -- "-n $TEST_ROOT/deploy-from-actions deploy $TARGET_COMMIT"

export ACTIONS_DEPLOY_UID="$CURRENT_UID"
export ACTIONS_STATE_FILE="$STATE_FILE"
export ACTIONS_INCOMING_DIR="$INCOMING_DIR"
export ACTIONS_FIXED_DEPLOYER="$FAKE_BIN/fixed-deployer"
export ACTIONS_REMOTE_DIR="$TEST_ROOT/app"
export FAKE_DEPLOY_LOG="$DEPLOY_LOG"
mkdir -p "$ACTIONS_REMOTE_DIR"

wrapper_status="$(
    SUDO_USER="$CURRENT_USER" bash "$DEPLOY_ENTRYPOINT" status
)"
test "$wrapper_status" = "$OTHER_COMMIT"

printf 'verified-image' > "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
chmod 600 "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
SUDO_USER="$CURRENT_USER" bash "$DEPLOY_ENTRYPOINT" deploy "$TARGET_COMMIT"
test ! -e "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
grep -Fq -- "--target $TARGET_COMMIT" "$DEPLOY_LOG"
grep -Fq -- "--remote-dir $ACTIONS_REMOTE_DIR" "$DEPLOY_LOG"
grep -Fq -- "--service frontdoor --service background" "$DEPLOY_LOG"

printf 'wrong-mode' > "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
chmod 644 "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
if SUDO_USER="$CURRENT_USER" \
    bash "$DEPLOY_ENTRYPOINT" deploy "$TARGET_COMMIT" \
        >"$TEST_ROOT/wrong-mode.out" 2>&1; then
    echo "wrong-mode upload unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "must use mode 600" "$TEST_ROOT/wrong-mode.out"
rm -f "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"

ln -s "$TEST_ROOT/missing-image" "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"
if SUDO_USER="$CURRENT_USER" \
    bash "$DEPLOY_ENTRYPOINT" deploy "$TARGET_COMMIT" \
        >"$TEST_ROOT/symlink.out" 2>&1; then
    echo "symlink upload unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "must be a regular non-symlink file" "$TEST_ROOT/symlink.out"
rm -f "$UPLOAD_DIR/niffler-app-$TARGET_COMMIT.tar"

if SUDO_USER="unexpected-user" \
    bash "$DEPLOY_ENTRYPOINT" status >"$TEST_ROOT/wrong-user.out" 2>&1; then
    echo "wrong sudo caller unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "must be invoked by $CURRENT_USER" "$TEST_ROOT/wrong-user.out"

echo "actions production access tests passed"
