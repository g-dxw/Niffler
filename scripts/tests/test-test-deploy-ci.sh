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
        printf '%s\t%s\t%s\n' "$FAKE_TARGET_COMMIT" "success" "Build App Image"
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
case "$*" in
    *" status")
        printf '%s\n' "$FAKE_CURRENT_COMMIT"
        ;;
    *" upload $FAKE_TARGET_COMMIT")
        upload_file="$FAKE_UPLOAD_FILE"
        dd of="$upload_file" 2>/dev/null
        if command -v sha256sum >/dev/null 2>&1; then
            upload_hash="$(sha256sum "$upload_file" | awk '{print $1}')"
        else
            upload_hash="$(shasum -a 256 "$upload_file" | awk '{print $1}')"
        fi
        printf 'uploaded_sha256=%s\n' "$upload_hash"
        ;;
    *" deploy $FAKE_TARGET_COMMIT")
        ;;
    *)
        echo "unexpected ssh command: $*" >&2
        exit 1
        ;;
esac
FAKE_SSH

cat > "$FAKE_BIN/scp" <<'FAKE_SCP'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_SCP_LOG"
exit 99
FAKE_SCP

chmod +x "$FAKE_BIN/git" "$FAKE_BIN/gh" "$FAKE_BIN/ssh" "$FAKE_BIN/scp"

export PATH="$FAKE_BIN:$PATH"
FAKE_REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
export FAKE_REPO_ROOT
export FAKE_TARGET_COMMIT="$TARGET_COMMIT"
export FAKE_CURRENT_COMMIT="$CURRENT_COMMIT"
export FAKE_SSH_LOG="$SSH_LOG"
export FAKE_SCP_LOG="$SCP_LOG"
export FAKE_UPLOAD_FILE="$TEST_ROOT/uploaded.tar"

GH_REPO="ryfineZ/Niffler" \
    bash "$DEPLOY_SCRIPT" \
        --host "deploy@example.test" \
    --remote-dir "/opt/niffler-test" \
    --run-id "123456" \
    --test-deployment \
    --restricted-actions \
    --source-health-url "http://127.0.0.1:18084/_gateway/health" \
    --public-health-url "https://test.example.test/"

grep -Fq -- "deploy@example.test status" "$SSH_LOG"
grep -Fq -- "deploy@example.test upload $TARGET_COMMIT" "$SSH_LOG"
grep -Fq -- "deploy@example.test deploy $TARGET_COMMIT" "$SSH_LOG"
test "$(cat "$FAKE_UPLOAD_FILE")" = "test-image"
test ! -s "$SCP_LOG"

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
