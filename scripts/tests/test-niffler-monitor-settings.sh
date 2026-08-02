#!/bin/bash

set -euo pipefail

SETTINGS_SCRIPT="${1:-$(dirname "$0")/../niffler-monitor-settings.sh}"
TEST_ROOT="$(mktemp -d /tmp/niffler-monitor-settings-test.XXXXXX)"
CONFIG_FILE="$TEST_ROOT/monitor.env"

cleanup() {
    rm -rf -- "$TEST_ROOT"
}
trap cleanup EXIT

cat > "$CONFIG_FILE" <<'EOF'
MONITOR_NODE_NAME=test
MONITOR_DISK_WARNING_PERCENT=80
MONITOR_DISK_CRITICAL_PERCENT=90
MONITOR_FAILURE_THRESHOLD=3
EOF
chmod 0600 "$CONFIG_FILE"

run_settings() {
    NIFFLER_MONITOR_CONFIG_FILE="$CONFIG_FILE" "$SETTINGS_SCRIPT" "$@"
}

run_settings show | grep -qx 'disk_warning=80'
run_settings set disk_warning 85 >/dev/null
[ "$(run_settings get disk_warning)" = "85" ]
run_settings set disk_critical 92 >/dev/null
[ "$(run_settings get disk_critical)" = "92" ]
run_settings set failures 5 >/dev/null
[ "$(run_settings get failures)" = "5" ]

if run_settings set disk_warning 93 >/dev/null 2>&1; then
    echo "warning threshold at or above critical was accepted" >&2
    exit 1
fi
if run_settings set disk_critical 100 >/dev/null 2>&1; then
    echo "out-of-range critical threshold was accepted" >&2
    exit 1
fi
if run_settings set failures 0 >/dev/null 2>&1; then
    echo "out-of-range failure threshold was accepted" >&2
    exit 1
fi

grep -q '^MONITOR_NODE_NAME=test$' "$CONFIG_FILE"
[ "$(stat -c '%a' "$CONFIG_FILE")" = "600" ]

echo "niffler monitor settings tests passed"
