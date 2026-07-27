#!/bin/bash

set -euo pipefail

TEST_MODE=false
if [ "${ACTIONS_TEST_MODE:-false}" = true ] && [ "$EUID" -ne 0 ]; then
    TEST_MODE=true
fi

if [ "$TEST_MODE" = true ]; then
    DEPLOY_USER="${ACTIONS_DEPLOY_USER:?}"
    DEPLOY_UID="${ACTIONS_DEPLOY_UID:?}"
    UPLOAD_DIR="${ACTIONS_UPLOAD_DIR:?}"
    INCOMING_DIR="${ACTIONS_INCOMING_DIR:?}"
    STATE_FILE="${ACTIONS_STATE_FILE:?}"
    FIXED_DEPLOYER="${ACTIONS_FIXED_DEPLOYER:?}"
    REMOTE_DIR="${ACTIONS_REMOTE_DIR:?}"
    MAX_UPLOAD_BYTES="${ACTIONS_MAX_UPLOAD_BYTES:-4294967296}"
else
    DEPLOY_USER="niffler-deploy"
    DEPLOY_UID="$(id -u "$DEPLOY_USER")"
    UPLOAD_DIR="/home/$DEPLOY_USER/uploads"
    INCOMING_DIR="/opt/niffler-release/incoming"
    STATE_FILE="/opt/niffler-app/.niffler-deployed-commit"
    FIXED_DEPLOYER="/opt/niffler-release/bin/deploy-production"
    REMOTE_DIR="/opt/niffler-app"
    MAX_UPLOAD_BYTES="4294967296"
fi

die() {
    echo "ERROR: $*" >&2
    exit 1
}

file_size() {
    local file_path="$1"
    stat -c '%s' "$file_path" 2>/dev/null || stat -f '%z' "$file_path"
}

file_mode() {
    local file_path="$1"
    stat -c '%a' "$file_path" 2>/dev/null || stat -f '%Lp' "$file_path"
}

file_uid() {
    local file_path="$1"
    stat -c '%u' "$file_path" 2>/dev/null || stat -f '%u' "$file_path"
}

validate_target_commit() {
    local target_commit="$1"
    if [[ ! "$target_commit" =~ ^[0-9a-f]{40}$ ]]; then
        die "invalid target commit"
    fi
}

if [ "$TEST_MODE" != true ] && [ "$EUID" -ne 0 ]; then
    die "must run as root"
fi
if [ "${SUDO_USER:-}" != "$DEPLOY_USER" ]; then
    die "must be invoked by $DEPLOY_USER"
fi
if [[ ! "$MAX_UPLOAD_BYTES" =~ ^[1-9][0-9]*$ ]]; then
    die "invalid upload size limit"
fi

command_name="${1:-}"
case "$command_name" in
    status)
        if [ "$#" -ne 1 ]; then
            die "status does not accept arguments"
        fi
        if [ ! -s "$STATE_FILE" ]; then
            die "deployed commit state is missing"
        fi
        deployed_commit="$(tr -d '[:space:]' < "$STATE_FILE")"
        validate_target_commit "$deployed_commit"
        printf '%s\n' "$deployed_commit"
        ;;
    deploy)
        if [ "$#" -ne 2 ]; then
            die "deploy requires one target commit"
        fi
        target_commit="$2"
        validate_target_commit "$target_commit"
        source_image="$UPLOAD_DIR/niffler-app-$target_commit.tar"
        if [ ! -f "$source_image" ] || [ -L "$source_image" ]; then
            die "uploaded image must be a regular non-symlink file"
        fi
        resolved_upload_dir="$(realpath "$UPLOAD_DIR")"
        resolved_source="$(realpath "$source_image")"
        expected_source="$resolved_upload_dir/$(basename "$source_image")"
        if [ "$resolved_source" != "$expected_source" ]; then
            die "uploaded image path is outside the dedicated upload location"
        fi
        if [ "$(file_uid "$source_image")" != "$DEPLOY_UID" ]; then
            die "uploaded image has the wrong owner"
        fi
        if [ "$(file_mode "$source_image")" != "600" ]; then
            die "uploaded image must use mode 600"
        fi
        upload_size="$(file_size "$source_image")"
        if [ "$upload_size" -le 0 ] || [ "$upload_size" -gt "$MAX_UPLOAD_BYTES" ]; then
            die "uploaded image size is outside the allowed range"
        fi
        if [ ! -x "$FIXED_DEPLOYER" ]; then
            die "fixed production deployer is unavailable"
        fi

        install -d -m 0700 "$INCOMING_DIR"
        staged_image="$INCOMING_DIR/niffler-app-$target_commit.tar"
        rm -f "$staged_image"
        install -m 0600 "$source_image" "$staged_image"
        rm -f "$source_image"
        cleanup_staged_image() {
            rm -f "$staged_image"
        }
        trap cleanup_staged_image EXIT

        "$FIXED_DEPLOYER" \
            --image-tar "$staged_image" \
            --target "$target_commit" \
            --remote-dir "$REMOTE_DIR" \
            --service frontdoor \
            --service background
        ;;
    *)
        die "unsupported privileged production command"
        ;;
esac
