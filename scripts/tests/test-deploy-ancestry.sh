#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
CHECK_SCRIPT="$SCRIPT_DIR/../check-deploy-ancestry.sh"
TMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

REPO="$TMP_DIR/repo"
git init -q "$REPO"
git -C "$REPO" config user.name "Deploy Guard Test"
git -C "$REPO" config user.email "deploy-guard@example.com"

commit_empty() {
    local message="$1"
    git -C "$REPO" commit -q --allow-empty -m "$message"
    git -C "$REPO" rev-parse HEAD
}

BASE_COMMIT="$(commit_empty "base")"
MAIN_COMMIT="$(commit_empty "main fix")"
CURRENT_COMMIT="$(commit_empty "current production")"
VALID_TARGET="$(commit_empty "valid target")"

git -C "$REPO" switch -q -c alternate "$MAIN_COMMIT"
ALTERNATE_TARGET="$(commit_empty "target missing current production")"

git -C "$REPO" switch -q -c stale "$BASE_COMMIT"
STALE_TARGET="$(commit_empty "target missing latest main")"

expect_pass() {
    local name="$1"
    shift
    if ! "$@" >"$TMP_DIR/output" 2>&1; then
        echo "FAIL: $name"
        cat "$TMP_DIR/output"
        exit 1
    fi
    echo "PASS: $name"
}

expect_fail() {
    local name="$1"
    shift
    if "$@" >"$TMP_DIR/output" 2>&1; then
        echo "FAIL: $name unexpectedly passed"
        cat "$TMP_DIR/output"
        exit 1
    fi
    echo "PASS: $name"
}

expect_pass \
    "target contains latest main and current production" \
    "$CHECK_SCRIPT" \
    --repo "$REPO" \
    --target "$VALID_TARGET" \
    --required-base "$MAIN_COMMIT" \
    --current "$CURRENT_COMMIT"

expect_fail \
    "target missing latest main is rejected" \
    "$CHECK_SCRIPT" \
    --repo "$REPO" \
    --target "$STALE_TARGET" \
    --required-base "$MAIN_COMMIT" \
    --current "$CURRENT_COMMIT"

expect_fail \
    "target missing current production is rejected" \
    "$CHECK_SCRIPT" \
    --repo "$REPO" \
    --target "$ALTERNATE_TARGET" \
    --required-base "$MAIN_COMMIT" \
    --current "$CURRENT_COMMIT"

expect_pass \
    "explicit rollback bypasses ancestry checks" \
    "$CHECK_SCRIPT" \
    --repo "$REPO" \
    --target "$STALE_TARGET" \
    --required-base "$MAIN_COMMIT" \
    --current "$CURRENT_COMMIT" \
    --allow-rollback

expect_fail \
    "unknown target commit is rejected" \
    "$CHECK_SCRIPT" \
    --repo "$REPO" \
    --target "0000000000000000000000000000000000000000" \
    --required-base "$MAIN_COMMIT"
