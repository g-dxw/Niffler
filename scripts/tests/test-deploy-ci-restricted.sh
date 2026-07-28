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
    *"rev-parse refs/remotes/"*)
        printf '%s\n' "$FAKE_TARGET_COMMIT"
        ;;
    *"fetch --quiet "*|*"cat-file -e "*|*"merge-base --is-ancestor "*)
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
    "run list")
        printf '%s\n' "123456"
        ;;
    "run view")
        printf '%s\t%s\t%s\t%s\n' \
            "$FAKE_TARGET_COMMIT" "completed" "success" "Build App Image"
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
        printf '%s' "verified-image" > "$download_dir/niffler-app-linux-amd64.tar"
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
        if [ "${FAKE_HASH_MISMATCH:-false}" = true ]; then
            printf '%s\n' "uploaded_sha256=cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
        else
            printf 'uploaded_sha256=%s\n' \
                "$(shasum -a 256 "$upload_file" | awk '{print $1}')"
        fi
        ;;
    *" deploy $FAKE_TARGET_COMMIT")
        printf '%s\n' "deployed"
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
SSH_OPTS="-p 22889 -o BatchMode=yes" \
    bash "$DEPLOY_SCRIPT" \
        --host "niffler-deploy@example.test" \
        --commit "$TARGET_COMMIT" \
        --restricted-actions

grep -Fq -- "niffler-deploy@example.test status" "$SSH_LOG"
grep -Fq -- "niffler-deploy@example.test upload $TARGET_COMMIT" "$SSH_LOG"
grep -Fq -- "niffler-deploy@example.test deploy $TARGET_COMMIT" "$SSH_LOG"
test "$(cat "$FAKE_UPLOAD_FILE")" = "verified-image"
test ! -s "$SCP_LOG"

if GH_REPO="ryfineZ/Niffler" \
    SSH_OPTS="-p 22889 -o BatchMode=yes" \
    FAKE_HASH_MISMATCH=true \
        bash "$DEPLOY_SCRIPT" \
            --host "niffler-deploy@example.test" \
            --commit "$TARGET_COMMIT" \
            --restricted-actions >"$TEST_ROOT/hash-mismatch.out" 2>&1; then
    echo "hash mismatch unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "Uploaded artifact SHA-256 does not match" "$TEST_ROOT/hash-mismatch.out"

if bash "$DEPLOY_SCRIPT" \
    --host "niffler-deploy@example.test" \
    --commit "$TARGET_COMMIT" \
    --restricted-actions \
    --allow-rollback >"$TEST_ROOT/rollback.out" 2>&1; then
    echo "restricted rollback unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "does not allow --allow-rollback" "$TEST_ROOT/rollback.out"

echo "restricted CI deployment tests passed"
