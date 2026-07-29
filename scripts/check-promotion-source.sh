#!/bin/bash

set -euo pipefail

GH_BIN="${GH_BIN:-gh}"
REPOSITORY=""
HEAD_REPOSITORY=""
HEAD_OWNER=""
HEAD_REF=""
HEAD_SHA=""

usage() {
    cat <<'EOF'
Usage: scripts/check-promotion-source.sh [options]

Options:
  --repository <owner/repo>       Target repository
  --head-repository <owner/repo>  Pull request source repository
  --head-owner <owner>            Pull request source owner
  --head-ref <branch>             Pull request source branch
  --head-sha <commit>             Pull request source commit
  -h, --help                      Show help
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

while [ "$#" -gt 0 ]; do
    case "$1" in
        --repository)
            require_option_value "$1" "${2:-}"
            REPOSITORY="$2"
            shift 2
            ;;
        --head-repository)
            require_option_value "$1" "${2:-}"
            HEAD_REPOSITORY="$2"
            shift 2
            ;;
        --head-owner)
            require_option_value "$1" "${2:-}"
            HEAD_OWNER="$2"
            shift 2
            ;;
        --head-ref)
            require_option_value "$1" "${2:-}"
            HEAD_REF="$2"
            shift 2
            ;;
        --head-sha)
            require_option_value "$1" "${2:-}"
            HEAD_SHA="$2"
            shift 2
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

if [ -z "$REPOSITORY" ] || [ -z "$HEAD_REPOSITORY" ] || [ -z "$HEAD_OWNER" ] || [ -z "$HEAD_REF" ] || [ -z "$HEAD_SHA" ]; then
    echo "Missing required promotion source arguments." >&2
    usage >&2
    exit 1
fi
if [[ ! "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]; then
    echo "Promotion rejected: head commit must be a lowercase 40-character Git SHA." >&2
    exit 1
fi

if [ "$HEAD_REF" = "test" ]; then
    if [ "$HEAD_REPOSITORY" != "$REPOSITORY" ]; then
        echo "Promotion rejected: main only accepts the upstream repository's test branch." >&2
        exit 1
    fi
    echo "Promotion source verified: test -> main."
    exit 0
fi

if [[ "$HEAD_REF" == hotfix/* ]] && [ "$HEAD_REF" != "hotfix/" ]; then
    if ! command -v "$GH_BIN" >/dev/null 2>&1; then
        echo "Required command not found: $GH_BIN" >&2
        exit 1
    fi

    merged_test_pr_count="$($GH_BIN api \
        --method GET \
        "repos/$REPOSITORY/pulls" \
        -f state=closed \
        -f base=test \
        -f "head=$HEAD_OWNER:$HEAD_REF" \
        --jq "[.[] | select(.merged_at != null and .head.sha == \"$HEAD_SHA\")] | length")"

    if [[ ! "$merged_test_pr_count" =~ ^[0-9]+$ ]]; then
        echo "Promotion rejected: GitHub returned an invalid companion PR count." >&2
        exit 1
    fi
    if [ "$merged_test_pr_count" -eq 0 ]; then
        echo "Promotion rejected: exact commit $HEAD_SHA must first be merged into test and deployed there." >&2
        exit 1
    fi

    echo "Promotion source verified: tested hotfix -> main."
    exit 0
fi

echo "Promotion rejected: main accepts only test or a tested hotfix/* branch." >&2
exit 1
