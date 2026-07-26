#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEPLOYER="$SCRIPT_DIR/../fixed-production-deployer.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

CURRENT_COMMIT="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
TARGET_COMMIT="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
OTHER_COMMIT="cccccccccccccccccccccccccccccccccccccccc"
FAKE_BIN="$TEST_ROOT/bin"
REMOTE_DIR="$TEST_ROOT/app"
RELEASE_ROOT="$TEST_ROOT/release"
DOCKER_LOG="$TEST_ROOT/docker.log"
mkdir -p "$FAKE_BIN"

cat > "$FAKE_BIN/git" <<'FAKE_GIT'
#!/bin/bash
set -euo pipefail
if [ "${1:-}" = "ls-remote" ]; then
    printf '%s\trefs/heads/main\n' "$FAKE_MAIN_COMMIT"
    exit 0
fi
if [ "${1:-}" = "init" ] && [ "${2:-}" = "--bare" ]; then
    mkdir -p "$3"
    exit 0
fi
if [[ "${1:-}" == --git-dir=* ]]; then
    shift
    case "${1:-}" in
        fetch|cat-file|merge-base)
            exit 0
            ;;
    esac
fi
echo "unexpected fake git invocation: $*" >&2
exit 2
FAKE_GIT

cat > "$FAKE_BIN/docker" <<'FAKE_DOCKER'
#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" >> "$FAKE_DOCKER_LOG"
case "${1:-}" in
    compose)
        shift
        case "${1:-}" in
            version|up)
                exit 0
                ;;
            ps)
                if [ "${2:-}" = "-q" ] && [ "${3:-}" = "frontdoor" ]; then
                    printf 'frontdoor-container\n'
                fi
                exit 0
                ;;
        esac
        ;;
    image)
        if [ "${2:-}" = "inspect" ]; then
            if [[ "$*" == *"--format"* ]]; then
                printf '%s\n' "$FAKE_TARGET_COMMIT"
            fi
            exit 0
        fi
        if [ "${2:-}" = "tag" ]; then
            exit 0
        fi
        ;;
    load)
        exit 0
        ;;
    inspect)
        if [ "${2:-}" = "frontdoor-container" ]; then
            printf 'AETHER_DATABASE_DRIVER=postgres\n'
            printf 'AETHER_DATABASE_URL=postgres://example.invalid/aether\n'
            exit 0
        fi
        ;;
    run)
        if [ "${FAKE_MIGRATION_COMPATIBLE:-true}" = "true" ]; then
            printf 'PostgreSQL migration compatibility verified; pending migrations: 0\n'
            exit 0
        fi
        echo 'VersionMissing(20260723121000)' >&2
        exit 1
        ;;
esac
echo "unexpected fake docker invocation: $*" >&2
exit 2
FAKE_DOCKER

cat > "$FAKE_BIN/curl" <<'FAKE_CURL'
#!/bin/bash
set -euo pipefail
if [ "${FAKE_CURL_FAIL:-false}" = "true" ]; then
    exit 22
fi
printf '{"status":"ok"}\n'
FAKE_CURL

chmod +x "$FAKE_BIN/git" "$FAKE_BIN/docker" "$FAKE_BIN/curl"

export PATH="$FAKE_BIN:$PATH"
export REPO_URL="https://example.invalid/Niffler.git"
export RELEASE_ROOT
export FAKE_MAIN_COMMIT="$TARGET_COMMIT"
export FAKE_TARGET_COMMIT="$TARGET_COMMIT"
export FAKE_DOCKER_LOG="$DOCKER_LOG"
export FAKE_CURL_FAIL=false
export FAKE_MIGRATION_COMPATIBLE=true

reset_fixture() {
    rm -rf "$REMOTE_DIR" "$RELEASE_ROOT"
    mkdir -p "$REMOTE_DIR"
    : > "$REMOTE_DIR/docker-compose.yml"
    printf 'APP_IMAGE=niffler-app:latest\n' > "$REMOTE_DIR/.env"
    printf '%s\n' "$CURRENT_COMMIT" > "$REMOTE_DIR/.niffler-deployed-commit"
    : > "$REMOTE_DIR/image.tar"
    : > "$DOCKER_LOG"
}

run_deployer() {
    bash "$DEPLOYER" \
        --image-tar "$REMOTE_DIR/image.tar" \
        --target "$1" \
        --remote-dir "$REMOTE_DIR" \
        --source-health-url "http://source.test/health" \
        --public-health-url "https://public.test/health"
}

assert_contains() {
    local file="$1"
    local expected="$2"
    grep -Fq -- "$expected" "$file" || {
        echo "expected '$expected' in $file" >&2
        exit 1
    }
}

assert_not_contains() {
    local file="$1"
    local unexpected="$2"
    if grep -Fq -- "$unexpected" "$file"; then
        echo "did not expect '$unexpected' in $file" >&2
        exit 1
    fi
}

reset_fixture
if run_deployer "$OTHER_COMMIT" >"$TEST_ROOT/not-main.out" 2>&1; then
    echo "non-main deployment unexpectedly succeeded" >&2
    exit 1
fi
assert_contains "$TEST_ROOT/not-main.out" "is not current origin/main"
assert_not_contains "$DOCKER_LOG" "load -i"

reset_fixture
export FAKE_MIGRATION_COMPATIBLE=false
if run_deployer "$TARGET_COMMIT" >"$TEST_ROOT/missing-migration.out" 2>&1; then
    echo "missing-migration deployment unexpectedly succeeded" >&2
    exit 1
fi
assert_contains "$TEST_ROOT/missing-migration.out" "incompatible with the active production PostgreSQL migration history"
assert_not_contains "$DOCKER_LOG" "compose up"
test "$(cat "$REMOTE_DIR/.niffler-deployed-commit")" = "$CURRENT_COMMIT"

reset_fixture
export FAKE_MIGRATION_COMPATIBLE=true
export FAKE_CURL_FAIL=true
if run_deployer "$TARGET_COMMIT" >"$TEST_ROOT/health-failure.out" 2>&1; then
    echo "health-failure deployment unexpectedly succeeded" >&2
    exit 1
fi
assert_contains "$TEST_ROOT/health-failure.out" "restoring niffler-app:rollback-"
test "$(grep -c '^compose up ' "$DOCKER_LOG")" -eq 2
test "$(cat "$REMOTE_DIR/.niffler-deployed-commit")" = "$CURRENT_COMMIT"
assert_contains "$REMOTE_DIR/.env" "APP_IMAGE=niffler-app:rollback-"

reset_fixture
export FAKE_CURL_FAIL=false
run_deployer "$TARGET_COMMIT" >"$TEST_ROOT/success.out" 2>&1
assert_contains "$TEST_ROOT/success.out" "Production deployment verified"
test "$(cat "$REMOTE_DIR/.niffler-deployed-commit")" = "$TARGET_COMMIT"
test ! -e "$REMOTE_DIR/image.tar"
assert_contains "$REMOTE_DIR/.env" "APP_IMAGE=niffler-app:$TARGET_COMMIT"

echo "fixed production deployer tests passed"
