#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEPLOYER="$SCRIPT_DIR/../fixed-test-deployer.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT
CURRENT="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
TARGET="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
FAKE_BIN="$TEST_ROOT/bin"
REMOTE_DIR="$TEST_ROOT/app"
RELEASE_ROOT="$TEST_ROOT/release"
mkdir -p "$FAKE_BIN" "$REMOTE_DIR"

cat > "$FAKE_BIN/git" <<'EOF'
#!/bin/bash
set -euo pipefail
if [ "${1:-}" = "ls-remote" ]; then
    if [[ "$*" == *"refs/heads/test"* ]]; then printf '%s\trefs/heads/test\n' "$FAKE_TARGET"; else printf '%s\trefs/heads/main\n' "$FAKE_MAIN"; fi
    exit 0
fi
if [ "${1:-}" = "init" ]; then mkdir -p "${3:-}"; exit 0; fi
if [[ "${1:-}" == --git-dir=* ]]; then
    case "${2:-}" in
      rev-parse)
        if [[ "${3:-}" == "$FAKE_TARGET"* && "${FAKE_TREE_MISMATCH:-false}" = "true" ]]; then
            printf 'target-tree\n'
        else
            printf 'shared-tree\n'
        fi
        exit 0
        ;;
      *) exit 0 ;;
    esac
fi
exit 2
EOF

cat > "$FAKE_BIN/docker" <<'EOF'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_DOCKER_LOG"
case "${1:-}" in
  compose)
    case "${2:-}" in
      version|up) exit 0 ;;
      ps) [ "${3:-}" = "-q" ] && printf 'app-container\n'; exit 0 ;;
      exec) printf 'backup'; exit 0 ;;
    esac ;;
  image)
    case "${2:-}" in
      inspect) [[ "$*" == *"--format"* ]] && printf '%s\n' "$FAKE_TARGET"; exit 0 ;;
      tag) exit 0 ;;
    esac ;;
  load) exit 0 ;;
  inspect) printf 'AETHER_DATABASE_DRIVER=postgres\n'; exit 0 ;;
  run) exit 0 ;;
esac
exit 2
EOF

cat > "$FAKE_BIN/curl" <<'EOF'
#!/bin/bash
[ "${FAKE_CURL_FAIL:-false}" != "true" ]
EOF
chmod +x "$FAKE_BIN/git" "$FAKE_BIN/docker" "$FAKE_BIN/curl"
export PATH="$FAKE_BIN:$PATH"
export FAKE_TARGET="$TARGET" FAKE_MAIN="$CURRENT" FAKE_DOCKER_LOG="$TEST_ROOT/docker.log"
export RELEASE_ROOT REMOTE_DIR
: > "$REMOTE_DIR/docker-compose.yml"
printf 'APP_IMAGE=niffler-app:%s\n' "$CURRENT" > "$REMOTE_DIR/.env"
printf '%s\n' "$CURRENT" > "$REMOTE_DIR/.niffler-deployed-commit"

printf 'image tar' | bash "$DEPLOYER" receive --target "$TARGET"
export FAKE_TREE_MISMATCH=true
if bash "$DEPLOYER" deploy --target "$TARGET" > "$TEST_ROOT/tree-mismatch.out" 2>&1; then
    echo "tree mismatch was unexpectedly accepted" >&2
    exit 1
fi
grep -Fq "tree does not exactly match" "$TEST_ROOT/tree-mismatch.out"
export FAKE_TREE_MISMATCH=false
bash "$DEPLOYER" deploy --target "$TARGET" > "$TEST_ROOT/success.out"
grep -Fq "Test deployment verified" "$TEST_ROOT/success.out"
test "$(cat "$REMOTE_DIR/.niffler-deployed-commit")" = "$TARGET"
grep -Fq "compose exec -T postgres" "$FAKE_DOCKER_LOG"
grep -Fq -- "--check-postgres-migration-compatibility" "$FAKE_DOCKER_LOG"

echo "fixed test deployer tests passed"
