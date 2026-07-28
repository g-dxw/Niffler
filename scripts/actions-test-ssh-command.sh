#!/bin/bash

set -euo pipefail

DEPLOYER_PATH="${DEPLOYER_PATH:-/opt/niffler-test-release/bin/deploy-test}"
ORIGINAL_COMMAND="${SSH_ORIGINAL_COMMAND:-}"

die() {
    echo "Rejected niffler-test deployment command" >&2
    exit 1
}

if [[ "$ORIGINAL_COMMAND" =~ ^niffler-test\ (receive|deploy)\ ([0-9a-f]{40})$ ]]; then
    action="${BASH_REMATCH[1]}"
    target="${BASH_REMATCH[2]}"
    exec sudo -n "$DEPLOYER_PATH" "$action" --target "$target"
fi

die
