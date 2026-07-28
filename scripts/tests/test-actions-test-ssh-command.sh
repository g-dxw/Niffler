#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WRAPPER="$SCRIPT_DIR/../actions-test-ssh-command.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT
mkdir -p "$TEST_ROOT/bin"

cat > "$TEST_ROOT/bin/sudo" <<'EOF'
#!/bin/bash
printf '%s\n' "$*" > "$FAKE_SUDO_LOG"
EOF
chmod +x "$TEST_ROOT/bin/sudo"
export PATH="$TEST_ROOT/bin:$PATH"
export FAKE_SUDO_LOG="$TEST_ROOT/sudo.log"
export DEPLOYER_PATH="/fixed/deployer"
sha="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"

SSH_ORIGINAL_COMMAND="niffler-test receive $sha" bash "$WRAPPER"
grep -Fqx -- "-n /fixed/deployer receive --target $sha" "$FAKE_SUDO_LOG"

SSH_ORIGINAL_COMMAND="niffler-test deploy $sha" bash "$WRAPPER"
grep -Fqx -- "-n /fixed/deployer deploy --target $sha" "$FAKE_SUDO_LOG"

if SSH_ORIGINAL_COMMAND="bash -i" bash "$WRAPPER" >/dev/null 2>&1; then
    echo "interactive shell was unexpectedly allowed" >&2
    exit 1
fi

echo "actions test SSH command tests passed"
