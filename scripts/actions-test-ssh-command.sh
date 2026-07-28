#!/bin/bash

set -euo pipefail

TEST_MODE=false
if [ "${ACTIONS_TEST_MODE:-false}" = true ] && [ -z "${SSH_CONNECTION:-}" ]; then
    TEST_MODE=true
fi

if [ "$TEST_MODE" = true ]; then
    DEPLOY_USER="${ACTIONS_DEPLOY_USER:?}"
    UPLOAD_DIR="${ACTIONS_UPLOAD_DIR:?}"
    PRIVILEGED_ENTRYPOINT="${ACTIONS_PRIVILEGED_ENTRYPOINT:?}"
    MAX_UPLOAD_BYTES="${ACTIONS_MAX_UPLOAD_BYTES:-4294967296}"
else
    DEPLOY_USER="niffler-test-deploy"
    UPLOAD_DIR="/home/$DEPLOY_USER/uploads"
    PRIVILEGED_ENTRYPOINT="/opt/niffler-test-release/bin/deploy-from-actions"
    MAX_UPLOAD_BYTES="4294967296"
fi
ORIGINAL_COMMAND="${SSH_ORIGINAL_COMMAND:-}"

die() {
    echo "ERROR: $*" >&2
    exit 1
}

file_size() {
    stat -c '%s' "$1" 2>/dev/null || stat -f '%z' "$1"
}

file_sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

validate_target_commit() {
    if [[ ! "$1" =~ ^[0-9a-f]{40}$ ]]; then
        die "invalid target commit"
    fi
}

if [ "$(id -un)" != "$DEPLOY_USER" ]; then
    die "must run as $DEPLOY_USER"
fi
if [[ ! "$MAX_UPLOAD_BYTES" =~ ^[1-9][0-9]*$ ]]; then
    die "invalid upload size limit"
fi

read -r -a command_parts <<< "$ORIGINAL_COMMAND"
command_name="${command_parts[0]:-}"

case "$command_name" in
    status)
        if [ "${#command_parts[@]}" -ne 1 ]; then
            die "status does not accept arguments"
        fi
        exec sudo -n "$PRIVILEGED_ENTRYPOINT" status
        ;;
    upload)
        if [ "${#command_parts[@]}" -ne 2 ]; then
            die "upload requires one target commit"
        fi
        target_commit="${command_parts[1]}"
        validate_target_commit "$target_commit"
        mkdir -p "$UPLOAD_DIR"
        chmod 0700 "$UPLOAD_DIR"
        umask 077
        temporary_upload="$(mktemp "$UPLOAD_DIR/.niffler-app-$target_commit.XXXXXX")"
        cleanup_upload() {
            rm -f "$temporary_upload"
        }
        trap cleanup_upload EXIT

        block_size=1048576
        block_count=$((MAX_UPLOAD_BYTES / block_size + 2))
        dd bs="$block_size" count="$block_count" of="$temporary_upload" 2>/dev/null
        upload_size="$(file_size "$temporary_upload")"
        if [ "$upload_size" -le 0 ]; then
            die "uploaded image is empty"
        fi
        if [ "$upload_size" -gt "$MAX_UPLOAD_BYTES" ]; then
            die "uploaded image exceeds $MAX_UPLOAD_BYTES bytes"
        fi

        final_upload="$UPLOAD_DIR/niffler-app-$target_commit.tar"
        chmod 0600 "$temporary_upload"
        mv -f "$temporary_upload" "$final_upload"
        trap - EXIT
        printf 'uploaded_sha256=%s\n' "$(file_sha256 "$final_upload")"
        ;;
    deploy)
        if [ "${#command_parts[@]}" -ne 2 ]; then
            die "deploy requires one target commit"
        fi
        target_commit="${command_parts[1]}"
        validate_target_commit "$target_commit"
        exec sudo -n "$PRIVILEGED_ENTRYPOINT" deploy "$target_commit"
        ;;
    *)
        die "unsupported test SSH command"
        ;;
esac
