#!/bin/bash

set -euo pipefail

CONFIG_FILE="${NIFFLER_MONITOR_CONFIG_FILE:-/etc/niffler-monitor/monitor.env}"
ACTION="${1:-}"
FIELD="${2:-}"
VALUE="${3:-}"

die() {
    echo "ERROR: $*" >&2
    exit 1
}

validate_file() {
    if [ ! -f "$CONFIG_FILE" ] || [ -L "$CONFIG_FILE" ]; then
        die "monitor configuration is missing or unsafe"
    fi
    if [ "$(stat -c '%a' "$CONFIG_FILE")" != "600" ]; then
        die "monitor configuration must use mode 600"
    fi
}

read_value() {
    local key="$1"

    awk -F= -v key="$key" '
        $1 == key {
            sub(/^[^=]*=/, "")
            gsub(/^"|"$/, "")
            print
            exit
        }
    ' "$CONFIG_FILE"
}

field_key() {
    case "$1" in
        disk_warning) printf 'MONITOR_DISK_WARNING_PERCENT' ;;
        disk_critical) printf 'MONITOR_DISK_CRITICAL_PERCENT' ;;
        failures) printf 'MONITOR_FAILURE_THRESHOLD' ;;
        *) die "unknown setting" ;;
    esac
}

validate_result() {
    local warning="$1"
    local critical="$2"
    local failures="$3"

    [[ "$warning" =~ ^[0-9]+$ ]] || die "warning value must be an integer"
    [[ "$critical" =~ ^[0-9]+$ ]] || die "critical value must be an integer"
    [[ "$failures" =~ ^[0-9]+$ ]] || die "failure count must be an integer"
    [ "$warning" -ge 50 ] && [ "$warning" -le 95 ] ||
        die "warning value must be between 50 and 95"
    [ "$critical" -ge 60 ] && [ "$critical" -le 99 ] ||
        die "critical value must be between 60 and 99"
    [ "$failures" -ge 1 ] && [ "$failures" -le 10 ] ||
        die "failure count must be between 1 and 10"
    [ "$warning" -lt "$critical" ] ||
        die "warning value must be lower than critical value"
}

validate_file
warning="$(read_value MONITOR_DISK_WARNING_PERCENT)"
critical="$(read_value MONITOR_DISK_CRITICAL_PERCENT)"
failures="$(read_value MONITOR_FAILURE_THRESHOLD)"

case "$ACTION" in
    show)
        validate_result "$warning" "$critical" "$failures"
        printf 'disk_warning=%s\n' "$warning"
        printf 'disk_critical=%s\n' "$critical"
        printf 'failures=%s\n' "$failures"
        ;;
    get)
        [ -n "$FIELD" ] || die "setting name is required"
        key="$(field_key "$FIELD")"
        read_value "$key"
        ;;
    validate|set)
        [ -n "$FIELD" ] && [ -n "$VALUE" ] || die "setting name and value are required"
        [[ "$VALUE" =~ ^[0-9]+$ ]] || die "setting value must be an integer"
        key="$(field_key "$FIELD")"
        case "$FIELD" in
            disk_warning) warning="$VALUE" ;;
            disk_critical) critical="$VALUE" ;;
            failures) failures="$VALUE" ;;
        esac
        validate_result "$warning" "$critical" "$failures"
        if [ "$ACTION" = "validate" ]; then
            exit 0
        fi
        config_directory="$(dirname "$CONFIG_FILE")"
        temporary_file="$(mktemp "$config_directory/.monitor.env.XXXXXX")"
        trap 'rm -f -- "${temporary_file:-}"' EXIT
        awk -F= -v key="$key" -v value="$VALUE" '
            BEGIN { updated = 0 }
            $1 == key {
                print key "=" value
                updated = 1
                next
            }
            { print }
            END {
                if (!updated) {
                    print key "=" value
                }
            }
        ' "$CONFIG_FILE" > "$temporary_file"
        chmod 0600 "$temporary_file"
        mv "$temporary_file" "$CONFIG_FILE"
        trap - EXIT
        printf '%s=%s\n' "$FIELD" "$VALUE"
        ;;
    *)
        die "usage: $0 show|get|validate|set [setting] [value]"
        ;;
esac
