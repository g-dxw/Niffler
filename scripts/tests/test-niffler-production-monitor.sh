#!/bin/bash

set -euo pipefail

MONITOR_SCRIPT="${1:-$(dirname "$0")/../niffler-production-monitor.sh}"
TEST_ROOT="$(mktemp -d /tmp/niffler-monitor-test.XXXXXX)"
FAKE_BIN="$TEST_ROOT/bin"
STATE_DIR="$TEST_ROOT/state"
RUNTIME_DIR="$TEST_ROOT/run"
CONFIG_FILE="$TEST_ROOT/monitor.env"
TELEGRAM_FILE="$TEST_ROOT/telegram.env"
MESSAGE_COUNT_FILE="$TEST_ROOT/message-count"
LAST_MESSAGE_FILE="$TEST_ROOT/last-message"
CONTAINER_HEALTH_FILE="$TEST_ROOT/container-health"
DISK_USAGE_FILE="$TEST_ROOT/disk-usage"

cleanup() {
    rm -rf -- "$TEST_ROOT"
}
trap cleanup EXIT

mkdir -p "$FAKE_BIN"
printf '0\n' > "$MESSAGE_COUNT_FILE"
printf '\n' > "$LAST_MESSAGE_FILE"
printf 'unhealthy\n' > "$CONTAINER_HEALTH_FILE"
printf '42\n' > "$DISK_USAGE_FILE"

cat > "$FAKE_BIN/docker" <<'EOF'
#!/bin/bash
set -euo pipefail
if [ "$1" != "inspect" ]; then
    exit 1
fi
if [ "$(cat "$CONTAINER_HEALTH_FILE")" = "healthy" ]; then
    printf 'true healthy\n'
else
    printf 'true unhealthy\n'
fi
EOF

cat > "$FAKE_BIN/df" <<'EOF'
#!/bin/bash
set -euo pipefail
usage="$(cat "$DISK_USAGE_FILE")"
printf '%s\n' \
    'Filesystem 1024-blocks Used Available Capacity Mounted on' \
    "/dev/test 104857600 1 17825792 ${usage}% /"
EOF

cat > "$FAKE_BIN/curl" <<'EOF'
#!/bin/bash
set -euo pipefail
output_file=""
write_out=0
while [ "$#" -gt 0 ]; do
    case "$1" in
        --output)
            shift
            output_file="$1"
            ;;
        --write-out)
            shift
            write_out=1
            ;;
        --data-urlencode)
            shift
            case "$1" in
                text=*) printf '%s\n' "${1#text=}" > "$LAST_MESSAGE_FILE" ;;
            esac
            ;;
        --config|--request|--connect-timeout|--max-time|--retry)
            shift
            ;;
    esac
    shift
done
printf '{"ok":true}\n' > "$output_file"
count="$(cat "$MESSAGE_COUNT_FILE")"
printf '%s\n' "$((count + 1))" > "$MESSAGE_COUNT_FILE"
if [ "$write_out" -eq 1 ]; then
    printf '200'
fi
EOF

chmod 0755 "$FAKE_BIN/docker" "$FAKE_BIN/df" "$FAKE_BIN/curl"

cat > "$CONFIG_FILE" <<EOF
MONITOR_NODE_NAME=test-node
MONITOR_NODE_DISPLAY_NAME="测试服务器（test-node）"
MONITOR_TELEGRAM_CONFIG_FILE=$TELEGRAM_FILE
MONITOR_DISK_PATH=/
MONITOR_DISK_LABEL=系统盘
MONITOR_DISK_WARNING_PERCENT=80
MONITOR_DISK_CRITICAL_PERCENT=90
MONITOR_FAILURE_THRESHOLD=3
MONITOR_CONTAINERS=test-container
MONITOR_CONTAINER_LABELS=测试服务
MONITOR_HTTP_URL=
MONITOR_REPORT_FILE=$TEST_ROOT/status.txt
EOF
cat > "$TELEGRAM_FILE" <<'EOF'
TELEGRAM_BOT_TOKEN=123456:test_token
TELEGRAM_CHAT_ID=123456
EOF
chmod 0600 "$CONFIG_FILE" "$TELEGRAM_FILE"

export CONTAINER_HEALTH_FILE DISK_USAGE_FILE MESSAGE_COUNT_FILE LAST_MESSAGE_FILE

run_monitor() {
    PATH="$FAKE_BIN:$PATH" \
        NIFFLER_MONITOR_CONFIG_FILE="$CONFIG_FILE" \
        NIFFLER_MONITOR_STATE_DIR="$STATE_DIR" \
        NIFFLER_MONITOR_RUNTIME_DIR="$RUNTIME_DIR" \
        "$MONITOR_SCRIPT" run >/dev/null
}

assert_state() {
    local check_name="$1"
    local expected_state="$2"
    local expected_count="$3"
    local state_file="$STATE_DIR/$check_name.state"

    grep -qx "STATE=$expected_state" "$state_file"
    grep -qx "FAILURE_COUNT=$expected_count" "$state_file"
}

assert_message_count() {
    local expected="$1"

    [ "$(cat "$MESSAGE_COUNT_FILE")" = "$expected" ]
}

run_monitor
assert_state container_test-container pending 1
assert_message_count 0

run_monitor
assert_state container_test-container pending 2
assert_message_count 0

run_monitor
assert_state container_test-container failed 3
assert_message_count 1
grep -q '测试服务连续 3 次检查异常' "$LAST_MESSAGE_FILE"
if grep -q 'health=' "$LAST_MESSAGE_FILE"; then
    echo "technical health details leaked into notification" >&2
    exit 1
fi

run_monitor
assert_state container_test-container failed 3
assert_message_count 1

printf 'healthy\n' > "$CONTAINER_HEALTH_FILE"
run_monitor
assert_state container_test-container ok 0
assert_message_count 2

run_monitor
assert_message_count 2

printf '83\n' > "$DISK_USAGE_FILE"
run_monitor
assert_state disk__ warning 0
assert_message_count 3
grep -q '系统盘空间偏少' "$LAST_MESSAGE_FILE"
grep -q '剩余 17.0 GB' "$LAST_MESSAGE_FILE"

run_monitor
assert_message_count 3

printf '92\n' > "$DISK_USAGE_FILE"
run_monitor
assert_state disk__ critical 0
assert_message_count 4

printf '42\n' > "$DISK_USAGE_FILE"
run_monitor
assert_state disk__ ok 0
assert_message_count 5

report="$(
    PATH="$FAKE_BIN:$PATH" \
        NIFFLER_MONITOR_CONFIG_FILE="$CONFIG_FILE" \
        NIFFLER_MONITOR_STATE_DIR="$STATE_DIR" \
        NIFFLER_MONITOR_RUNTIME_DIR="$RUNTIME_DIR" \
        "$MONITOR_SCRIPT" report
)"
grep -q '测试服务器（test-node）' <<< "$report"
grep -q '系统盘：正常' <<< "$report"
grep -q '测试服务：运行正常' <<< "$report"

echo "niffler production monitor tests passed"
