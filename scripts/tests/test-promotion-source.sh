#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CHECKER="$SCRIPT_DIR/../check-promotion-source.sh"
TMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

cat > "$TMP_DIR/gh" <<'EOF'
#!/bin/bash
set -euo pipefail
printf '%s\n' "${MOCK_COMPANION_PR_COUNT:-0}"
EOF
chmod +x "$TMP_DIR/gh"

run_pass() {
    local description="$1"
    shift
    if ! "$@" >"$TMP_DIR/stdout" 2>"$TMP_DIR/stderr"; then
        echo "Expected success: $description" >&2
        cat "$TMP_DIR/stderr" >&2
        exit 1
    fi
}

run_fail() {
    local description="$1"
    shift
    if "$@" >"$TMP_DIR/stdout" 2>"$TMP_DIR/stderr"; then
        echo "Expected failure: $description" >&2
        cat "$TMP_DIR/stdout" >&2
        exit 1
    fi
}

COMMON_ARGS=(
    --repository ryfineZ/Niffler
    --head-owner ryfineZ
    --head-sha aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
)

run_pass "upstream test promotes to main" \
    "$CHECKER" "${COMMON_ARGS[@]}" \
    --head-repository ryfineZ/Niffler \
    --head-ref test

run_fail "fork test cannot impersonate the upstream integration branch" \
    "$CHECKER" "${COMMON_ARGS[@]}" \
    --head-repository g-dxw/Niffler \
    --head-ref test

run_fail "ordinary feature branch cannot target main" \
    "$CHECKER" "${COMMON_ARGS[@]}" \
    --head-repository g-dxw/Niffler \
    --head-ref feature/direct-main

run_fail "hotfix without merged test PR is rejected" \
    env GH_BIN="$TMP_DIR/gh" MOCK_COMPANION_PR_COUNT=0 \
    "$CHECKER" \
    --repository ryfineZ/Niffler \
    --head-repository g-dxw/Niffler \
    --head-owner g-dxw \
    --head-ref hotfix/production-failure \
    --head-sha bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

run_pass "hotfix merged into test can target main" \
    env GH_BIN="$TMP_DIR/gh" MOCK_COMPANION_PR_COUNT=1 \
    "$CHECKER" \
    --repository ryfineZ/Niffler \
    --head-repository g-dxw/Niffler \
    --head-owner g-dxw \
    --head-ref hotfix/production-failure \
    --head-sha bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

run_fail "empty hotfix branch name is rejected" \
    env GH_BIN="$TMP_DIR/gh" MOCK_COMPANION_PR_COUNT=1 \
    "$CHECKER" \
    --repository ryfineZ/Niffler \
    --head-repository g-dxw/Niffler \
    --head-owner g-dxw \
    --head-ref hotfix/ \
    --head-sha bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb

run_fail "invalid head commit is rejected" \
    "$CHECKER" \
    --repository ryfineZ/Niffler \
    --head-repository ryfineZ/Niffler \
    --head-owner ryfineZ \
    --head-ref test \
    --head-sha not-a-commit

echo "promotion source checks passed"
