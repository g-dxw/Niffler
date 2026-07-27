#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
VERIFY_SCRIPT="$SCRIPT_DIR/../verify-ssh-host-key.sh"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

if [ ! -f "$VERIFY_SCRIPT" ]; then
    echo "SSH host key verifier is missing: $VERIFY_SCRIPT" >&2
    exit 1
fi
bash -n "$VERIFY_SCRIPT"

ssh-keygen -q -t ed25519 -N "" -f "$TEST_ROOT/expected"
ssh-keygen -q -t ed25519 -N "" -f "$TEST_ROOT/untrusted"
EXPECTED_FINGERPRINT="$(
    ssh-keygen -lf "$TEST_ROOT/expected.pub" -E sha256 | awk '{print $2}'
)"

FAKE_BIN="$TEST_ROOT/bin"
mkdir -p "$FAKE_BIN"
cat > "$FAKE_BIN/ssh-keyscan" <<'FAKE_KEYSCAN'
#!/bin/bash
set -euo pipefail
printf 'example.test %s %s\n' \
    "$(awk '{print $1}' "$EXPECTED_PUBLIC_KEY")" \
    "$(awk '{print $2}' "$EXPECTED_PUBLIC_KEY")"
printf 'example.test %s %s\n' \
    "$(awk '{print $1}' "$UNTRUSTED_PUBLIC_KEY")" \
    "$(awk '{print $2}' "$UNTRUSTED_PUBLIC_KEY")"
FAKE_KEYSCAN
chmod +x "$FAKE_BIN/ssh-keyscan"

export PATH="$FAKE_BIN:$PATH"
export EXPECTED_PUBLIC_KEY="$TEST_ROOT/expected.pub"
export UNTRUSTED_PUBLIC_KEY="$TEST_ROOT/untrusted.pub"

KNOWN_HOSTS="$TEST_ROOT/known_hosts"
"$VERIFY_SCRIPT" \
    --host example.test \
    --port 22889 \
    --fingerprint "$EXPECTED_FINGERPRINT" \
    --output "$KNOWN_HOSTS"

test "$(wc -l < "$KNOWN_HOSTS" | tr -d '[:space:]')" = "1"
grep -Fq "$(awk '{print $2}' "$EXPECTED_PUBLIC_KEY")" "$KNOWN_HOSTS"
if grep -Fq "$(awk '{print $2}' "$UNTRUSTED_PUBLIC_KEY")" "$KNOWN_HOSTS"; then
    echo "untrusted host key was written to known_hosts" >&2
    exit 1
fi
test "$(stat -f '%Lp' "$KNOWN_HOSTS" 2>/dev/null \
    || stat -c '%a' "$KNOWN_HOSTS")" = "600"

if "$VERIFY_SCRIPT" \
    --host example.test \
    --port 22889 \
    --fingerprint "SHA256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" \
    --output "$TEST_ROOT/mismatch-known-hosts" \
        >"$TEST_ROOT/mismatch.out" 2>&1; then
    echo "mismatched host fingerprint unexpectedly succeeded" >&2
    exit 1
fi
grep -Fq "no scanned SSH host key matched" "$TEST_ROOT/mismatch.out"
test ! -e "$TEST_ROOT/mismatch-known-hosts"

echo "SSH host key verification tests passed"
