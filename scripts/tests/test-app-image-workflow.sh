#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKFLOW="$SCRIPT_DIR/../../.github/workflows/app-image.yml"
PROMOTION_WORKFLOW="$SCRIPT_DIR/../../.github/workflows/promotion-policy.yml"

test -f "$WORKFLOW"
grep -Fq "workflow_dispatch:" "$WORKFLOW"
grep -Fq "      - test" "$WORKFLOW"
grep -Fq "cancel-in-progress: \${{ github.ref_name != 'test' }}" "$WORKFLOW"
grep -Fq "if: github.ref_name == 'test'" "$WORKFLOW"
grep -Fq "name: test" "$WORKFLOW"
grep -Fq 'url: ${{ vars.MYLINGWEAVE_PUBLIC_URL }}' "$WORKFLOW"
grep -Fq 'SSH_USER: ${{ vars.MYLINGWEAVE_USER }}' "$WORKFLOW"
grep -Fq 'REMOTE_DIR: ${{ vars.MYLINGWEAVE_REMOTE_DIR }}' "$WORKFLOW"
grep -Fq 'SOURCE_HEALTH_URL: ${{ vars.MYLINGWEAVE_SOURCE_HEALTH_URL }}' "$WORKFLOW"
grep -Fq -- '--run-id "${{ github.run_id }}"' "$WORKFLOW"
grep -Fq -- '--test-deployment' "$WORKFLOW"
grep -Fq -- '--source-health-url "${SOURCE_HEALTH_URL}"' "$WORKFLOW"
grep -Fq -- '--public-health-url "${{ vars.MYLINGWEAVE_PUBLIC_URL }}"' "$WORKFLOW"

test -f "$PROMOTION_WORKFLOW"
grep -Fq "pull_request_target:" "$PROMOTION_WORKFLOW"
grep -Fq "      - main" "$PROMOTION_WORKFLOW"
grep -Fq "name: Promotion policy" "$PROMOTION_WORKFLOW"
grep -Fq "pull-requests: read" "$PROMOTION_WORKFLOW"
grep -Fq 'ref: ${{ github.event.pull_request.base.sha }}' "$PROMOTION_WORKFLOW"
grep -Fq "scripts/check-promotion-source.sh" "$PROMOTION_WORKFLOW"
grep -Fq -- '--head-ref "${{ github.head_ref }}"' "$PROMOTION_WORKFLOW"
grep -Fq -- '--head-sha "${{ github.event.pull_request.head.sha }}"' "$PROMOTION_WORKFLOW"

echo "test deployment and promotion workflow checks passed"
