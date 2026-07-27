#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEPLOY_USER="niffler-deploy"
DEPLOY_HOME="/home/$DEPLOY_USER"
RELEASE_BIN="/opt/niffler-release/bin"
SSH_COMMAND_TARGET="$RELEASE_BIN/actions-ssh-command"
DEPLOY_ENTRYPOINT_TARGET="$RELEASE_BIN/deploy-from-actions"
SUDOERS_TARGET="/etc/sudoers.d/niffler-deploy"
PUBLIC_KEY_FILE=""

die() {
    echo "ERROR: $*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage: install-actions-production-access.sh --public-key-file <path>

Installs the restricted GitHub Actions production SSH account and command
entrypoints. Run only as root from a reviewed main checkout.
EOF
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --public-key-file)
            if [ "$#" -lt 2 ] || [ -z "${2:-}" ]; then
                die "--public-key-file requires a path"
            fi
            PUBLIC_KEY_FILE="$2"
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

if [ "$EUID" -ne 0 ]; then
    die "installer must run as root"
fi
if [ -z "$PUBLIC_KEY_FILE" ] || [ ! -f "$PUBLIC_KEY_FILE" ]; then
    die "public key file is required"
fi
for required_script in \
    "$SCRIPT_DIR/actions-production-ssh-command.sh" \
    "$SCRIPT_DIR/actions-production-deploy.sh"; do
    if [ ! -f "$required_script" ]; then
        die "required script is missing: $required_script"
    fi
    bash -n "$required_script"
done
for command_name in useradd usermod id install awk ssh-keygen visudo getent; do
    command -v "$command_name" >/dev/null 2>&1 \
        || die "required command is missing: $command_name"
done

nonempty_key_lines="$(
    awk 'NF && $1 !~ /^#/ {count++} END {print count+0}' "$PUBLIC_KEY_FILE"
)"
if [ "$nonempty_key_lines" != "1" ]; then
    die "public key file must contain exactly one key"
fi
public_key="$(awk 'NF && $1 !~ /^#/ {print; exit}' "$PUBLIC_KEY_FILE")"
if [[ "$public_key" != ssh-ed25519\ * ]]; then
    die "production deploy key must use Ed25519"
fi
ssh-keygen -lf "$PUBLIC_KEY_FILE" -E sha256 >/dev/null \
    || die "public key validation failed"

if ! id "$DEPLOY_USER" >/dev/null 2>&1; then
    useradd \
        --system \
        --user-group \
        --create-home \
        --home-dir "$DEPLOY_HOME" \
        --shell /bin/bash \
        "$DEPLOY_USER"
fi
deploy_passwd_entry="$(getent passwd "$DEPLOY_USER")"
deploy_home="$(printf '%s\n' "$deploy_passwd_entry" | awk -F: '{print $6}')"
deploy_shell="$(printf '%s\n' "$deploy_passwd_entry" | awk -F: '{print $7}')"
if [ "$deploy_home" != "$DEPLOY_HOME" ] || [ "$deploy_shell" != "/bin/bash" ]; then
    die "existing $DEPLOY_USER account has unexpected home or shell"
fi
if id -nG "$DEPLOY_USER" | tr ' ' '\n' | grep -Fxq docker; then
    die "$DEPLOY_USER must not belong to the docker group"
fi
usermod -L "$DEPLOY_USER"

install -d -o "$DEPLOY_USER" -g "$DEPLOY_USER" -m 0700 "$DEPLOY_HOME/uploads"
install -d -o root -g root -m 0700 "$DEPLOY_HOME/.ssh"
install -d -o root -g root -m 0755 "$RELEASE_BIN"
install -o root -g root -m 0755 \
    "$SCRIPT_DIR/actions-production-ssh-command.sh" \
    "$SSH_COMMAND_TARGET"
install -o root -g root -m 0755 \
    "$SCRIPT_DIR/actions-production-deploy.sh" \
    "$DEPLOY_ENTRYPOINT_TARGET"

authorized_keys_tmp="$(mktemp)"
sudoers_tmp="$(mktemp)"
cleanup() {
    rm -f "$authorized_keys_tmp" "$sudoers_tmp"
}
trap cleanup EXIT

printf 'restrict,command="%s" %s\n' "$SSH_COMMAND_TARGET" "$public_key" \
    > "$authorized_keys_tmp"
install -o root -g root -m 0600 \
    "$authorized_keys_tmp" \
    "$DEPLOY_HOME/.ssh/authorized_keys"

printf '%s ALL=(root) NOPASSWD:NOSETENV: %s *\n' \
    "$DEPLOY_USER" \
    "$DEPLOY_ENTRYPOINT_TARGET" \
    > "$sudoers_tmp"
visudo -cf "$sudoers_tmp" >/dev/null
install -o root -g root -m 0440 "$sudoers_tmp" "$SUDOERS_TARGET"
visudo -cf "$SUDOERS_TARGET" >/dev/null

echo "Restricted GitHub Actions production access installed."
