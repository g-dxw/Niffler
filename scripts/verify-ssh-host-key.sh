#!/bin/bash

set -euo pipefail

SSH_HOST=""
SSH_PORT=""
EXPECTED_FINGERPRINT=""
OUTPUT_PATH=""

die() {
    echo "ERROR: $*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: verify-ssh-host-key.sh --host <host> --port <port> \
  --fingerprint <SHA256:fingerprint> --output <known-hosts-path>

Scans an SSH server and writes only host keys whose SHA-256 fingerprint
exactly matches the expected value.
EOF
}

require_option_value() {
    local option_name="$1"
    local option_value="${2:-}"
    if [ -z "$option_value" ] || [[ "$option_value" == --* ]]; then
        die "$option_name requires a value"
    fi
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --host)
            require_option_value "$1" "${2:-}"
            SSH_HOST="$2"
            shift 2
            ;;
        --port)
            require_option_value "$1" "${2:-}"
            SSH_PORT="$2"
            shift 2
            ;;
        --fingerprint)
            require_option_value "$1" "${2:-}"
            EXPECTED_FINGERPRINT="$2"
            shift 2
            ;;
        --output)
            require_option_value "$1" "${2:-}"
            OUTPUT_PATH="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown argument: $1"
            ;;
    esac
done

if [ -z "$SSH_HOST" ] || [[ "$SSH_HOST" == -* ]] \
    || [[ "$SSH_HOST" =~ [[:space:]] ]]; then
    die "SSH host is invalid"
fi
if [[ ! "$SSH_PORT" =~ ^[0-9]{1,5}$ ]] \
    || ((10#$SSH_PORT < 1 || 10#$SSH_PORT > 65535)); then
    die "SSH port is invalid"
fi
if [[ ! "$EXPECTED_FINGERPRINT" =~ ^SHA256:[A-Za-z0-9+/]{43}$ ]]; then
    die "SSH host fingerprint is invalid"
fi
if [[ "$OUTPUT_PATH" != /* ]] || [ "$OUTPUT_PATH" = "/" ]; then
    die "output must be an explicit absolute file path"
fi
for command_name in ssh-keyscan ssh-keygen install mktemp awk; do
    command -v "$command_name" >/dev/null 2>&1 \
        || die "required command is missing: $command_name"
done

scan_file="$(mktemp)"
public_key_file="$(mktemp)"
verified_host_keys="$(mktemp)"
cleanup() {
    rm -f "$scan_file" "$public_key_file" "$verified_host_keys"
}
trap cleanup EXIT

ssh-keyscan -p "$SSH_PORT" -H "$SSH_HOST" > "$scan_file"
if [ ! -s "$scan_file" ]; then
    die "ssh-keyscan returned no host keys"
fi

while read -r host_field key_type key_data _; do
    if [ -z "$host_field" ] || [[ "$host_field" == \#* ]]; then
        continue
    fi
    if [[ ! "$key_type" =~ ^[A-Za-z0-9@._+-]+$ ]] \
        || [[ ! "$key_data" =~ ^[A-Za-z0-9+/=]+$ ]]; then
        continue
    fi
    printf '%s %s\n' "$key_type" "$key_data" > "$public_key_file"
    scanned_fingerprint="$(
        ssh-keygen -lf "$public_key_file" -E sha256 2>/dev/null \
            | awk 'NR == 1 {print $2}'
    )"
    if [ "$scanned_fingerprint" = "$EXPECTED_FINGERPRINT" ]; then
        printf '%s %s %s\n' "$host_field" "$key_type" "$key_data" \
            >> "$verified_host_keys"
    fi
done < "$scan_file"

if [ ! -s "$verified_host_keys" ]; then
    die "no scanned SSH host key matched the expected fingerprint"
fi
install -m 0600 "$verified_host_keys" "$OUTPUT_PATH"

echo "SSH host key fingerprint verified."
