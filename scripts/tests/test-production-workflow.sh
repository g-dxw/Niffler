#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOW="$SCRIPT_DIR/../../.github/workflows/deploy-production.yml"

test -f "$WORKFLOW"
grep -Fq "workflow_dispatch:" "$WORKFLOW"
grep -Fq "if: github.ref == 'refs/heads/main'" "$WORKFLOW"
grep -Fq "name: production" "$WORKFLOW"
grep -Fq "cancel-in-progress: false" "$WORKFLOW"
target_input_expression="TARGET_COMMIT: \${{ inputs.commit }}"
exact_commit_check="if [[ \"\${TARGET_COMMIT}\" != \"\${GITHUB_SHA}\" ]]"
fingerprint_expression="SSH_HOST_FINGERPRINT: \${{ secrets.PRODUCTION_SSH_HOST_FINGERPRINT }}"
grep -Fq "$target_input_expression" "$WORKFLOW"
grep -Fq "$exact_commit_check" "$WORKFLOW"
grep -Fq "$fingerprint_expression" "$WORKFLOW"
grep -Fq "scripts/verify-ssh-host-key.sh" "$WORKFLOW"
grep -Fq "StrictHostKeyChecking=yes" "$WORKFLOW"
grep -Fq -- "--restricted-actions" "$WORKFLOW"

action_count=0
while IFS= read -r action_ref; do
    action_count=$((action_count + 1))
    if [[ ! "$action_ref" =~ @[0-9a-f]{40}$ ]]; then
        echo "production workflow action is not pinned to a full commit: $action_ref" >&2
        exit 1
    fi
done < <(
    awk '/^[[:space:]]*uses:/ {print $2}' "$WORKFLOW"
)
if [ "$action_count" -eq 0 ]; then
    echo "production workflow has no pinned action" >&2
    exit 1
fi

echo "production workflow security checks passed"
