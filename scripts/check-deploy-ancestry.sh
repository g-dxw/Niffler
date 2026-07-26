#!/bin/bash

set -euo pipefail

REPO_ROOT=""
TARGET_COMMIT=""
REQUIRED_BASE_COMMIT=""
CURRENT_DEPLOYED_COMMIT=""
ALLOW_ROLLBACK=false

usage() {
    cat <<'EOF'
Usage: scripts/check-deploy-ancestry.sh [options]

Options:
  --repo <path>             Git repository path
  --target <commit>         Commit that will be deployed
  --required-base <commit>  Commit that normal deployments must contain
  --current <commit>        Current production commit that normal deployments must contain
  --allow-rollback          Explicitly allow a non-descendant deployment
  -h, --help                Show help
EOF
}

require_option_value() {
    local option_name="$1"
    local option_value="${2:-}"
    if [ -z "$option_value" ] || [[ "$option_value" == --* ]]; then
        echo "Missing value for $option_name" >&2
        usage >&2
        exit 1
    fi
}

while [ $# -gt 0 ]; do
    case "$1" in
        --repo)
            require_option_value "$1" "${2:-}"
            REPO_ROOT="${2:-}"
            shift 2
            ;;
        --target)
            require_option_value "$1" "${2:-}"
            TARGET_COMMIT="${2:-}"
            shift 2
            ;;
        --required-base)
            require_option_value "$1" "${2:-}"
            REQUIRED_BASE_COMMIT="${2:-}"
            shift 2
            ;;
        --current)
            require_option_value "$1" "${2:-}"
            CURRENT_DEPLOYED_COMMIT="${2:-}"
            shift 2
            ;;
        --allow-rollback)
            ALLOW_ROLLBACK=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown argument: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

if [ -z "$REPO_ROOT" ] || [ -z "$TARGET_COMMIT" ] || [ -z "$REQUIRED_BASE_COMMIT" ]; then
    echo "Missing required deployment ancestry arguments." >&2
    usage >&2
    exit 1
fi

assert_commit_exists() {
    local label="$1"
    local commit="$2"
    if ! git -C "$REPO_ROOT" cat-file -e "${commit}^{commit}" 2>/dev/null; then
        echo "$label is not available in the local Git repository: $commit" >&2
        exit 1
    fi
}

assert_commit_exists "Target commit" "$TARGET_COMMIT"
assert_commit_exists "Required base commit" "$REQUIRED_BASE_COMMIT"
if [ -n "$CURRENT_DEPLOYED_COMMIT" ]; then
    assert_commit_exists "Current production commit" "$CURRENT_DEPLOYED_COMMIT"
fi

if [ "$ALLOW_ROLLBACK" = true ]; then
    echo "Deployment ancestry checks bypassed for an explicit rollback."
    exit 0
fi

if ! git -C "$REPO_ROOT" merge-base --is-ancestor "$REQUIRED_BASE_COMMIT" "$TARGET_COMMIT"; then
    echo "Deployment rejected: target commit $TARGET_COMMIT does not contain required base $REQUIRED_BASE_COMMIT." >&2
    exit 1
fi

if [ -n "$CURRENT_DEPLOYED_COMMIT" ] \
    && ! git -C "$REPO_ROOT" merge-base --is-ancestor "$CURRENT_DEPLOYED_COMMIT" "$TARGET_COMMIT"; then
    echo "Deployment rejected: target commit $TARGET_COMMIT does not contain current production commit $CURRENT_DEPLOYED_COMMIT." >&2
    exit 1
fi

echo "Deployment ancestry verified for target commit $TARGET_COMMIT."
