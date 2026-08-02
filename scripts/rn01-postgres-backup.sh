#!/bin/bash

set -euo pipefail

CONFIG_FILE="${NIFFLER_BACKUP_CONFIG_FILE:-/etc/niffler-backup/r2.env}"
BACKUP_DIR="${NIFFLER_BACKUP_DIR:-/var/backups/niffler-postgres}"
STATUS_DIR="${NIFFLER_BACKUP_STATUS_DIR:-/var/lib/niffler-backup}"
LOCK_FILE="${NIFFLER_BACKUP_LOCK_FILE:-/run/lock/niffler-postgres-backup.lock}"
POSTGRES_CONTAINER="${NIFFLER_POSTGRES_CONTAINER:-niffler-postgres}"
POSTGRES_DATABASE="${NIFFLER_POSTGRES_DATABASE:-aether}"
POSTGRES_USER="${NIFFLER_POSTGRES_USER:-postgres}"
MIN_FREE_MARGIN_BYTES="${NIFFLER_BACKUP_FREE_MARGIN_BYTES:-10737418240}"

BACKUP_PATH=""
PARTIAL_PATH=""
CHECKSUM_PATH=""
BACKUP_ID=""
OBJECT_KEY=""
BACKUP_BYTES=""
BACKUP_SHA256=""

die() {
    echo "ERROR: $*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || die "required command is unavailable: $1"
}

write_status() {
    local status="$1"
    local status_file="$STATUS_DIR/status.env"
    local temporary_file="$STATUS_DIR/.status.env.$$"

    install -d -m 0700 "$STATUS_DIR"
    {
        printf 'STATUS=%s\n' "$status"
        printf 'UPDATED_AT=%s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
        printf 'BACKUP_ID=%s\n' "$BACKUP_ID"
        printf 'OBJECT_KEY=%s\n' "$OBJECT_KEY"
        printf 'BACKUP_BYTES=%s\n' "$BACKUP_BYTES"
        printf 'BACKUP_SHA256=%s\n' "$BACKUP_SHA256"
    } > "$temporary_file"
    chmod 0600 "$temporary_file"
    mv "$temporary_file" "$status_file"
}

cleanup_local_files() {
    local file_path

    for file_path in "$PARTIAL_PATH" "$BACKUP_PATH" "$CHECKSUM_PATH"; do
        if [ -n "$file_path" ] && [ -f "$file_path" ] && [ ! -L "$file_path" ]; then
            rm -f -- "$file_path"
        fi
    done
}

finish() {
    local exit_code=$?

    trap - EXIT
    cleanup_local_files
    if [ "$exit_code" -eq 0 ]; then
        write_status success
    else
        write_status failed
    fi
    exit "$exit_code"
}

trap finish EXIT

if [ "$EUID" -ne 0 ]; then
    die "must run as root"
fi
if [[ ! "$MIN_FREE_MARGIN_BYTES" =~ ^[1-9][0-9]*$ ]]; then
    die "free space margin must be a positive integer"
fi

require_command docker
require_command flock
require_command rclone
require_command sha256sum

exec 9> "$LOCK_FILE"
flock -n 9 || die "another backup is already running"

if [ ! -f "$CONFIG_FILE" ] || [ -L "$CONFIG_FILE" ]; then
    die "backup credential file is missing or unsafe"
fi
if [ "$(stat -c '%a' "$CONFIG_FILE")" != "600" ]; then
    die "backup credential file must use mode 600"
fi

set -a
# shellcheck disable=SC1090
source "$CONFIG_FILE"
set +a

: "${R2_BUCKET:?}"
: "${R2_ENDPOINT:?}"
: "${AWS_ACCESS_KEY_ID:?}"
: "${AWS_SECRET_ACCESS_KEY:?}"

export RCLONE_CONFIG_R2_TYPE=s3
export RCLONE_CONFIG_R2_PROVIDER=Cloudflare
export RCLONE_CONFIG_R2_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID"
export RCLONE_CONFIG_R2_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY"
export RCLONE_CONFIG_R2_ENDPOINT="$R2_ENDPOINT"
export RCLONE_CONFIG_R2_REGION=auto
export RCLONE_CONFIG_R2_NO_CHECK_BUCKET=true

install -d -m 0700 "$BACKUP_DIR" "$STATUS_DIR"

docker inspect "$POSTGRES_CONTAINER" >/dev/null 2>&1 \
    || die "PostgreSQL container is unavailable"
docker exec "$POSTGRES_CONTAINER" \
    pg_isready -U "$POSTGRES_USER" -d "$POSTGRES_DATABASE" >/dev/null \
    || die "PostgreSQL is not ready"

database_bytes="$(
    docker exec "$POSTGRES_CONTAINER" \
        psql -X -U "$POSTGRES_USER" -d "$POSTGRES_DATABASE" -Atc \
        'SELECT pg_database_size(current_database());'
)"
available_bytes="$(df --output=avail -B1 "$BACKUP_DIR" | tail -n 1 | tr -d ' ')"
required_bytes=$((database_bytes + MIN_FREE_MARGIN_BYTES))
if [ "$available_bytes" -lt "$required_bytes" ]; then
    die "insufficient disk space: available=$available_bytes required=$required_bytes"
fi

BACKUP_ID="$(date -u '+%Y%m%dT%H%M%SZ')"
year="$(date -u '+%Y')"
month="$(date -u '+%m')"
BACKUP_PATH="$BACKUP_DIR/aether-$BACKUP_ID.dump"
PARTIAL_PATH="$BACKUP_PATH.partial"
CHECKSUM_PATH="$BACKUP_PATH.sha256"
OBJECT_KEY="postgres/aether/daily/$year/$month/aether-$BACKUP_ID.dump"

if [ -e "$BACKUP_PATH" ] || [ -e "$PARTIAL_PATH" ] || [ -e "$CHECKSUM_PATH" ]; then
    die "backup path already exists"
fi

write_status running
echo "Starting PostgreSQL backup $BACKUP_ID"

docker exec "$POSTGRES_CONTAINER" nice -n 10 ionice -c 2 -n 7 pg_dump \
    -U "$POSTGRES_USER" \
    -d "$POSTGRES_DATABASE" \
    --format=custom \
    --compress=6 \
    --no-owner \
    --no-privileges \
    > "$PARTIAL_PATH"

if [ ! -s "$PARTIAL_PATH" ]; then
    die "pg_dump produced an empty backup"
fi

docker exec -i "$POSTGRES_CONTAINER" pg_restore --list \
    < "$PARTIAL_PATH" > /dev/null

mv "$PARTIAL_PATH" "$BACKUP_PATH"
chmod 0600 "$BACKUP_PATH"
BACKUP_BYTES="$(stat -c '%s' "$BACKUP_PATH")"
BACKUP_SHA256="$(sha256sum "$BACKUP_PATH" | awk '{print $1}')"
printf '%s  %s\n' "$BACKUP_SHA256" "$(basename "$BACKUP_PATH")" > "$CHECKSUM_PATH"
chmod 0600 "$CHECKSUM_PATH"

rclone copyto "$BACKUP_PATH" "r2:$R2_BUCKET/$OBJECT_KEY" \
    --retries 3 \
    --low-level-retries 10
rclone copyto "$CHECKSUM_PATH" "r2:$R2_BUCKET/$OBJECT_KEY.sha256" \
    --retries 3 \
    --low-level-retries 10

remote_bytes="$(
    rclone lsl "r2:$R2_BUCKET/$OBJECT_KEY" | awk 'NR == 1 {print $1}'
)"
remote_sha256="$(
    rclone cat "r2:$R2_BUCKET/$OBJECT_KEY.sha256" | awk 'NR == 1 {print $1}'
)"
if [ "$remote_bytes" != "$BACKUP_BYTES" ]; then
    die "uploaded backup size mismatch"
fi
if [ "$remote_sha256" != "$BACKUP_SHA256" ]; then
    die "uploaded backup checksum mismatch"
fi

day_of_week="$(date -u '+%u')"
day_of_month="$(date -u '+%d')"
if [ "$day_of_week" = "7" ]; then
    weekly_key="postgres/aether/weekly/$year/$month/aether-$BACKUP_ID.dump"
    rclone copyto "r2:$R2_BUCKET/$OBJECT_KEY" "r2:$R2_BUCKET/$weekly_key"
    rclone copyto "r2:$R2_BUCKET/$OBJECT_KEY.sha256" \
        "r2:$R2_BUCKET/$weekly_key.sha256"
fi
if [ "$day_of_month" = "01" ]; then
    monthly_key="postgres/aether/monthly/$year/$month/aether-$BACKUP_ID.dump"
    rclone copyto "r2:$R2_BUCKET/$OBJECT_KEY" "r2:$R2_BUCKET/$monthly_key"
    rclone copyto "r2:$R2_BUCKET/$OBJECT_KEY.sha256" \
        "r2:$R2_BUCKET/$monthly_key.sha256"
fi

rclone delete "r2:$R2_BUCKET/postgres/aether/daily" \
    --min-age 8d \
    --include '*.dump' \
    --include '*.dump.sha256'
rclone delete "r2:$R2_BUCKET/postgres/aether/weekly" \
    --min-age 29d \
    --include '*.dump' \
    --include '*.dump.sha256'
rclone delete "r2:$R2_BUCKET/postgres/aether/monthly" \
    --min-age 190d \
    --include '*.dump' \
    --include '*.dump.sha256'

echo "Backup uploaded and verified: $OBJECT_KEY ($BACKUP_BYTES bytes)"
