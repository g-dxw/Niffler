#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
PUBLIC_KEY_FILE=""
DEPLOY_USER="niffler-deploy"
RELEASE_ROOT="/opt/niffler-test-release"

while [ "$#" -gt 0 ]; do
    case "$1" in
        --public-key-file)
            PUBLIC_KEY_FILE="${2:-}"
            shift 2
            ;;
        *)
            echo "Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

[ "$(id -u)" -eq 0 ] || { echo "Run as root" >&2; exit 1; }
[ -s "$PUBLIC_KEY_FILE" ] || { echo "A public key file is required" >&2; exit 1; }
ssh-keygen -l -f "$PUBLIC_KEY_FILE" >/dev/null

if ! id "$DEPLOY_USER" >/dev/null 2>&1; then
    useradd --create-home --shell /bin/bash "$DEPLOY_USER"
fi
passwd -l "$DEPLOY_USER" >/dev/null

install -d -m 0755 "$RELEASE_ROOT/bin"
install -m 0755 "$SCRIPT_DIR/fixed-test-deployer.sh" "$RELEASE_ROOT/bin/deploy-test"
install -m 0755 "$SCRIPT_DIR/actions-test-ssh-command.sh" /usr/local/sbin/niffler-test-ssh-command

install -d -o "$DEPLOY_USER" -g "$DEPLOY_USER" -m 0700 "/home/$DEPLOY_USER/.ssh"
key_line="$(tr -d '\r\n' < "$PUBLIC_KEY_FILE")"
printf 'restrict,command="/usr/local/sbin/niffler-test-ssh-command" %s\n' "$key_line" \
    > "/home/$DEPLOY_USER/.ssh/authorized_keys"
chown "$DEPLOY_USER:$DEPLOY_USER" "/home/$DEPLOY_USER/.ssh/authorized_keys"
chmod 0600 "/home/$DEPLOY_USER/.ssh/authorized_keys"

printf '%s ALL=(root) NOPASSWD: %s/bin/deploy-test *\n' "$DEPLOY_USER" "$RELEASE_ROOT" \
    > /etc/sudoers.d/niffler-test-deploy
chmod 0440 /etc/sudoers.d/niffler-test-deploy
visudo -cf /etc/sudoers.d/niffler-test-deploy

echo "Restricted niffler-test Actions access installed for $DEPLOY_USER"
