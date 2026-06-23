# PostgreSQL-Only Data Layer Slimming Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove MySQL/MariaDB and SQLite support so the gateway ships and runs as a PostgreSQL-only product.

**Architecture:** Keep the existing repository contracts and PostgreSQL implementations. Delete MySQL/SQLite drivers, repositories, migrations, single-node SQLite deployment paths, and sqlx feature flags, then update startup validation so any non-PostgreSQL URL fails fast with a clear error.

**Tech Stack:** Rust workspace, sqlx PostgreSQL, Axum gateway, Docker Compose, shell installer scripts, Markdown docs.

---

## Scope

This is a breaking product change. After completion, supported persistent storage is PostgreSQL only.

In scope:
- Remove MySQL/MariaDB and SQLite runtime support from `aether-data`.
- Remove MySQL and SQLite migrations, backfills, driver modules, repository implementations, and database lifecycle branches.
- Remove SQLite single-node deployment mode and PostgreSQL-to-single-node migration scripts.
- Remove MySQL compose profile and MySQL CI smoke paths.
- Remove `sqlx` `mysql` and `sqlite` features from the workspace dependency.
- Update tests to use PostgreSQL testkit, in-memory repositories, or pure unit tests.

Out of scope:
- Migrating existing MySQL/SQLite production data into PostgreSQL.
- Keeping a hidden compatibility mode.
- Retaining single-node SQLite as an unsupported option.

## Task 1: Record The Breaking Storage Policy

**Files:**
- Create: `docs/architecture/postgres-only-data-layer.md`
- Modify: `README.md`
- Modify: `docs/architecture/data-schema-inventory.md`

- [ ] **Step 1: Add the architecture note**

Create `docs/architecture/postgres-only-data-layer.md`:

```markdown
# PostgreSQL-Only Data Layer

## Goal

Aether/Niffler supports PostgreSQL as its only persistent SQL database.

## Non-Goals

- No MySQL or MariaDB runtime support.
- No SQLite runtime support.
- No single-node SQLite deployment mode.
- No cross-database copy/export path for production migrations.

## Behavior Changes

- Startup accepts only PostgreSQL connection URLs: `postgres://...` and `postgresql://...`.
- `mysql://`, `mariadb://`, and `sqlite://` URLs fail during configuration validation.
- Database migrations and backfills run only against PostgreSQL.
- Docker Compose starts PostgreSQL and Redis; it no longer defines a MySQL profile.
- Single-node SQLite install and migration scripts are removed.

## Impact

Operators must run PostgreSQL. Existing SQLite/MySQL deployments need an external migration plan before upgrading to this version.

## Validation

- `cargo test -p aether-data --no-default-features`
- `cargo test -p aether-gateway --no-default-features`
- `cargo fmt --check`
- `git diff --check`
```

- [ ] **Step 2: Update the README deployment wording**

Edit `README.md`:
- Remove the single-node SQLite quick-start lines.
- Replace database examples with PostgreSQL only:

```markdown
- `DATABASE_URL`：PostgreSQL 连接串，例如 `postgresql://postgres:aether@postgres:5432/aether`。
```

- [ ] **Step 3: Update schema inventory**

Edit `docs/architecture/data-schema-inventory.md` so it describes only:

```markdown
`sqlx` migrations remain under `crates/aether-data/migrations/postgres`.
```

Remove table rows and prose that say MySQL or SQLite migrations remain supported.

- [ ] **Step 4: Verify docs mention the new policy**

Run:

```bash
rg -n "MySQL|mysql|MariaDB|mariadb|SQLite|sqlite|single-node" README.md docs/architecture
```

Expected: remaining matches are either historical notes outside the active architecture docs or explicit statements that the old backends were removed.

## Task 2: Remove Non-Postgres Cargo Features And Public Exports

**Files:**
- Modify: `Cargo.toml`
- Modify: `crates/aether-data/src/lib.rs`
- Modify: `crates/aether-data/tests/public_entrypoints.rs`

- [ ] **Step 1: Shrink sqlx features**

Change workspace `sqlx` dependency in `Cargo.toml` from:

```toml
sqlx = { version = "0.8", default-features = false, features = ["postgres", "mysql", "sqlite", "runtime-tokio-rustls", "chrono"] }
```

to:

```toml
sqlx = { version = "0.8", default-features = false, features = ["postgres", "runtime-tokio-rustls", "chrono"] }
```

- [ ] **Step 2: Remove SQLite default export**

In `crates/aether-data/src/lib.rs`, replace:

```rust
pub use database::{DatabaseDriver, SqlDatabaseConfig, SqlPoolConfig, DEFAULT_SQLITE_DATABASE_URL};
```

with:

```rust
pub use database::{DatabaseDriver, SqlDatabaseConfig, SqlPoolConfig};
```

- [ ] **Step 3: Update public entrypoint test**

In `crates/aether-data/tests/public_entrypoints.rs`, remove `mysql` and `sqlite` from the expected public module list and remove any imports that mention `driver::mysql::MySqlPool`.

- [ ] **Step 4: Verify feature removal starts surfacing compile errors**

Run:

```bash
cargo check -p aether-data --no-default-features
```

Expected: FAIL at references to MySQL/SQLite types. These compile errors drive the next tasks.

## Task 3: Collapse Database Configuration To PostgreSQL

**Files:**
- Modify: `crates/aether-data/src/database.rs`
- Modify: `crates/aether-data/src/config.rs`
- Modify: `apps/aether-gateway/src/main.rs`
- Modify: `apps/aether-gateway/src/data/config.rs`

- [ ] **Step 1: Restrict `DatabaseDriver`**

In `crates/aether-data/src/database.rs`, reduce the enum to:

```rust
pub enum DatabaseDriver {
    Postgres,
}
```

Update parser helpers so only `postgres` and `postgresql` are accepted:

```rust
pub fn from_database_url(url: &str) -> Option<Self> {
    let scheme = url.split_once(':')?.0.to_ascii_lowercase();
    match scheme.as_str() {
        "postgres" | "postgresql" => Some(Self::Postgres),
        _ => None,
    }
}
```

Update `FromStr` error text to:

```rust
"unsupported database driver '{other}'; expected postgres"
```

- [ ] **Step 2: Remove SQLite default helper**

Delete `DEFAULT_SQLITE_DATABASE_URL` and `SqlDatabaseConfig::sqlite_default()`.

- [ ] **Step 3: Keep URL-driver mismatch validation**

Keep `SqlDatabaseConfig::validate()` but make unsupported URL schemes fail:

```rust
let Some(url_driver) = DatabaseDriver::from_database_url(&self.url) else {
    return Err(DataLayerError::InvalidConfiguration(
        "database url must use postgres:// or postgresql://".to_string(),
    ));
};
if url_driver != self.driver {
    return Err(DataLayerError::InvalidConfiguration(format!(
        "database driver '{}' does not match url scheme '{}'",
        self.driver, url_driver
    )));
}
```

- [ ] **Step 4: Remove SQLite fallback from gateway CLI**

In `apps/aether-gateway/src/main.rs`, delete the import of `DEFAULT_SQLITE_DATABASE_URL` and remove the branch that turns `AETHER_DATABASE_DRIVER=sqlite` into a default URL.

Non-Postgres driver errors should read:

```rust
"AETHER_DATABASE_DRIVER only supports postgres"
```

- [ ] **Step 5: Update configuration tests**

Replace tests that expect SQLite/MySQL driver parsing with tests that assert rejection:

```rust
assert!("sqlite".parse::<DatabaseDriver>().is_err());
assert!("mysql".parse::<DatabaseDriver>().is_err());
assert_eq!(
    DatabaseDriver::from_database_url("postgresql://localhost/aether"),
    Some(DatabaseDriver::Postgres)
);
assert_eq!(DatabaseDriver::from_database_url("sqlite://./data/aether.db"), None);
```

- [ ] **Step 6: Verify configuration unit tests**

Run:

```bash
cargo test -p aether-data database --no-default-features
cargo test -p aether-gateway config --no-default-features
```

Expected: PASS after all non-Postgres expectations are removed.

## Task 4: Remove MySQL/SQLite Backend Composition

**Files:**
- Modify: `crates/aether-data/src/backend/mod.rs`
- Modify: `crates/aether-data/src/backend/read.rs`
- Modify: `crates/aether-data/src/backend/write.rs`
- Modify: `crates/aether-data/src/backend/maintenance.rs`
- Modify: `crates/aether-data/src/backend/stats.rs`
- Modify: `crates/aether-data/src/backend/wallet.rs`
- Modify: `crates/aether-data/src/backend/system.rs`
- Delete: `crates/aether-data/src/backend/mysql.rs`
- Delete: `crates/aether-data/src/backend/sqlite.rs`
- Delete: `crates/aether-data/src/backend/stats/mysql.rs`
- Delete: `crates/aether-data/src/backend/stats/sqlite.rs`

- [ ] **Step 1: Remove backend modules and fields**

In `backend/mod.rs`, delete:

```rust
mod mysql;
mod sqlite;
pub use mysql::MysqlBackend;
pub use sqlite::SqliteBackend;
```

Change `DataBackends` to keep only:

```rust
postgres: Option<PostgresBackend>,
```

Change `SqlBackendRef` to:

```rust
enum SqlBackendRef<'a> {
    Postgres(&'a PostgresBackend),
}
```

- [ ] **Step 2: Build only Postgres backend**

In `DataBackends::from_config`, remove MySQL and SQLite construction. Build repositories with:

```rust
let read = DataReadRepositories::from_postgres(postgres.as_ref());
let write = DataWriteRepositories::from_postgres(postgres.as_ref());
```

- [ ] **Step 3: Simplify read/write repository factories**

In `read.rs` and `write.rs`, remove `mysql` and `sqlite` parameters from `from_backends`, or replace it with `from_postgres`.

Each repository field should be assigned only from `PostgresBackend`, for example:

```rust
content_moderation_evidence: postgres
    .map(PostgresBackend::content_moderation_evidence_read_repository),
```

- [ ] **Step 4: Simplify maintenance dispatch**

In `backend/maintenance.rs`, remove every `Self::Mysql` and `Self::Sqlite` branch. Migration, backfill, pool summary, wallet aggregation, stats aggregation, and system config operations should dispatch only to Postgres.

- [ ] **Step 5: Remove mixed SQL constants**

In `backend/wallet.rs` and `backend/system.rs`, delete MySQL/SQLite SQL constants and methods. Keep Postgres code paths only.

- [ ] **Step 6: Verify backend compile**

Run:

```bash
cargo check -p aether-data --no-default-features
```

Expected: remaining errors point into repository modules, lifecycle modules, or gateway tests, not backend composition.

## Task 5: Delete MySQL/SQLite Repository Implementations

**Files:**
- Modify every `crates/aether-data/src/repository/*/mod.rs` that declares `mod mysql;` or `mod sqlite;`
- Delete every `crates/aether-data/src/repository/*/mysql.rs`
- Delete every `crates/aether-data/src/repository/*/sqlite.rs`

- [ ] **Step 1: Remove module declarations**

For each repository module, remove lines like:

```rust
mod mysql;
mod sqlite;
pub use mysql::Mysql...Repository;
pub use sqlite::Sqlite...Repository;
```

Keep memory and Postgres exports where they already exist.

- [ ] **Step 2: Delete MySQL/SQLite repository files**

Run:

```bash
find crates/aether-data/src/repository -name mysql.rs -delete
find crates/aether-data/src/repository -name sqlite.rs -delete
```

- [ ] **Step 3: Remove direct MySQL/SQLite repository tests**

Search:

```bash
rg -n "Mysql|mysql|Sqlite|sqlite" crates/aether-data/src/repository
```

Expected: no MySQL/SQLite repository implementation references remain. Matches in historical comments should be deleted or rewritten.

- [ ] **Step 4: Verify repository compile**

Run:

```bash
cargo check -p aether-data --no-default-features
```

Expected: remaining errors point to lifecycle, driver, migration, export, or application tests.

## Task 6: Remove MySQL/SQLite Drivers, Migrations, Backfills, And Export Paths

**Files:**
- Modify: `crates/aether-data/src/driver/mod.rs`
- Delete: `crates/aether-data/src/driver/mysql/`
- Delete: `crates/aether-data/src/driver/sqlite/`
- Modify: `crates/aether-data/src/lifecycle/migrate/mod.rs`
- Delete: `crates/aether-data/src/lifecycle/migrate/mysql.rs`
- Delete: `crates/aether-data/src/lifecycle/migrate/sqlite.rs`
- Modify: `crates/aether-data/src/lifecycle/backfill.rs`
- Modify: `crates/aether-data/src/lifecycle/export.rs`
- Delete: `crates/aether-data/migrations/mysql/`
- Delete: `crates/aether-data/migrations/sqlite/`

- [ ] **Step 1: Remove driver modules**

In `driver/mod.rs`, remove:

```rust
pub mod mysql;
pub mod sqlite;
```

Delete both driver directories.

- [ ] **Step 2: Keep only Postgres migrations**

In `lifecycle/migrate/mod.rs`, remove MySQL and SQLite migration functions and exports. Keep `run_migrations`, `pending_migrations`, and `prepare_database_for_startup` for PostgreSQL.

- [ ] **Step 3: Remove MySQL/SQLite backfill branches**

In `lifecycle/backfill.rs`, remove functions named `run_mysql_backfills`, `run_sqlite_backfills`, `pending_mysql_backfills`, and `pending_sqlite_backfills`. Keep PostgreSQL backfill lock behavior.

- [ ] **Step 4: Remove cross-database export/copy**

In `lifecycle/export.rs`, keep only PostgreSQL export/import behavior that is still used. Remove source/target dispatch for MySQL and SQLite.

If the `aether-gateway copy` command only exists for PostgreSQL-to-SQLite single-node migration, delete the command entrypoint from `apps/aether-gateway/src/main.rs` in Task 7.

- [ ] **Step 5: Delete migration directories**

Run:

```bash
rm -rf crates/aether-data/migrations/mysql crates/aether-data/migrations/sqlite
```

- [ ] **Step 6: Verify data crate compile**

Run:

```bash
cargo check -p aether-data --no-default-features
```

Expected: PASS for `aether-data`, or only errors in public tests that still reference deleted drivers.

## Task 7: Remove SQLite Single-Node And MySQL Deployment Paths

**Files:**
- Modify: `docker-compose.yml`
- Delete: `docker-compose.single-node.yml`
- Modify: `install.sh`
- Delete: `scripts/migrate-pg-to-single-node.sh`
- Delete: `scripts/migrate-pg-compose-to-single-node.sh`
- Modify: `.env.example`
- Modify: `.github/workflows/rust-ci.yml`

- [ ] **Step 1: Remove MySQL compose profile**

In `docker-compose.yml`, delete the `mysql` service and the `mysql_data` volume.

- [ ] **Step 2: Delete single-node compose**

Delete:

```bash
rm docker-compose.single-node.yml
```

- [ ] **Step 3: Remove installer single-node modes**

In `install.sh`, remove branches that generate:

```env
AETHER_DATABASE_DRIVER=sqlite
AETHER_DATABASE_URL=sqlite://...
DATABASE_URL=sqlite://...
AETHER_RUNTIME_BACKEND=memory
AETHER_GATEWAY_DEPLOYMENT_TOPOLOGY=single-node
```

Keep cluster/multi-node PostgreSQL installation paths.

- [ ] **Step 4: Remove PG-to-single-node migration scripts**

Delete:

```bash
rm scripts/migrate-pg-to-single-node.sh
rm scripts/migrate-pg-compose-to-single-node.sh
```

- [ ] **Step 5: Remove MySQL CI smoke job**

In `.github/workflows/rust-ci.yml`, delete `data_db_smoke_mysql` and remove it from downstream `needs` or failure checks.

- [ ] **Step 6: Verify deployment references**

Run:

```bash
rg -n "mysql|mariadb|sqlite|single-node|compose-single-node" docker-compose.yml install.sh scripts .github/workflows README.md .env.example docs
```

Expected: no active deployment path remains for MySQL, MariaDB, SQLite, or single-node SQLite.

## Task 8: Update Gateway Tests Away From SQLite

**Files:**
- Modify: `apps/aether-gateway/src/main.rs`
- Modify: `apps/aether-gateway/src/data/tests.rs`
- Modify: `apps/aether-gateway/src/usage/reporting/mod.rs`
- Modify: `apps/aether-gateway/src/niffler_billing_reservation.rs`
- Modify: `apps/aether-gateway/src/maintenance/runtime/niffler_billing_reservation_expiry.rs`

- [ ] **Step 1: Replace SQLite in-memory data tests**

For tests that only need a data state, use in-memory repositories or `GatewayDataState::disabled()` instead of:

```rust
SqlDatabaseConfig::new(DatabaseDriver::Sqlite, "sqlite::memory:", pool)
```

- [ ] **Step 2: Move true SQL integration tests to PostgreSQL testkit**

For tests that require SQL behavior, use `aether-testkit` Postgres helpers, matching existing patterns under `crates/aether-testkit/src/postgres.rs`.

- [ ] **Step 3: Update gateway CLI config tests**

Remove tests that say SQLite is valid for single-node. Add tests that reject SQLite:

```rust
let error = parse_database_config_from_env_like_values(
    Some("sqlite"),
    Some("sqlite://./data/aether.db"),
).unwrap_err();
assert!(error.to_string().contains("only supports postgres"));
```

- [ ] **Step 4: Verify gateway compile**

Run:

```bash
cargo check -p aether-gateway --no-default-features
```

Expected: PASS or errors only in docs/tests outside gateway.

## Task 9: Workspace-Wide Cleanup

**Files:**
- Modify files found by the commands below.

- [ ] **Step 1: Remove remaining backend names**

Run:

```bash
rg -n "Mysql|mysql|MariaDB|mariadb|Sqlite|sqlite|SQLite|single-node|compose-single-node" .
```

For source code and active docs, remove or rewrite every match. Historical docs can stay only if they explicitly state the path was removed.

- [ ] **Step 2: Remove generated or schema driver folders**

Delete MySQL/SQLite schema driver sources if they exist:

```bash
rm -rf crates/aether-data/schema/drivers/mysql
rm -rf crates/aether-data/schema/drivers/sqlite
rm -rf crates/aether-data/schema/generated/mysql
rm -rf crates/aether-data/schema/generated/sqlite
```

- [ ] **Step 3: Update schema tooling**

In `crates/aether-data-schema`, remove MySQL/SQLite emitters and tests if that crate exists only to maintain multi-driver schemas. Keep PostgreSQL schema generation if still used.

- [ ] **Step 4: Verify no sqlx feature regressed**

Run:

```bash
cargo tree -e features -i sqlx | rg "mysql|sqlite"
```

Expected: no `mysql` or `sqlite` sqlx feature remains enabled.

## Task 10: Final Verification

**Files:**
- No new files expected.

- [ ] **Step 1: Format**

Run:

```bash
cargo fmt --check
```

Expected: PASS.

- [ ] **Step 2: Rust tests**

Run:

```bash
cargo test -p aether-data --no-default-features
cargo test -p aether-gateway --no-default-features
cargo test --workspace --no-default-features
```

Expected: PASS.

- [ ] **Step 3: Frontend checks if scripts are available**

Run:

```bash
cd frontend && npm test -- --run
```

Expected: PASS, or document if frontend test scripts are not installed in this workspace.

- [ ] **Step 4: Diff hygiene**

Run:

```bash
git diff --check
```

Expected: PASS.

- [ ] **Step 5: Build-size evidence**

Run before and after implementation on the same machine:

```bash
cargo build -p aether-gateway --release --no-default-features
ls -lh target/release/aether-gateway
cargo tree -e features -i sqlx
```

Expected: release binary no longer includes sqlx MySQL/SQLite features; binary size should decrease. Record the before/after numbers in the final report.

## Risks

- Existing single-node SQLite users cannot upgrade without a separate migration path.
- Existing MySQL/MariaDB users lose support.
- Some tests currently use SQLite in-memory as a cheap integration database; replacing them with Postgres testkit may make tests slower.
- `aether-data-schema` may still assume multi-driver output; removing those emitters can require extra cleanup beyond `aether-data`.

## Rollback

Rollback is a source-level revert of this branch. There is no database rollback because the plan removes code paths and deployment templates, not live operator data.

## Recommended Execution Order

1. Commit Task 1 as documentation-only breaking-change record.
2. Commit Tasks 2-4 together once `aether-data` compiles.
3. Commit Tasks 5-6 after repository/lifecycle cleanup compiles.
4. Commit Tasks 7-8 after deployment and gateway tests compile.
5. Commit Tasks 9-10 after full verification.
