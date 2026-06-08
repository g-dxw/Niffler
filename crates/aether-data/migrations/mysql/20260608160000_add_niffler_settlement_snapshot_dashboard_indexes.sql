SET @niffler_settlement_snapshots_created_at_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'niffler_settlement_snapshots'
          AND index_name = 'idx_niffler_settlement_snapshots_created_at'
    ) = 0,
    'CREATE INDEX idx_niffler_settlement_snapshots_created_at ON niffler_settlement_snapshots (created_at_unix_ms, request_id)',
    'DO 0'
);

PREPARE niffler_settlement_snapshots_created_at_index_stmt FROM @niffler_settlement_snapshots_created_at_index_sql;
EXECUTE niffler_settlement_snapshots_created_at_index_stmt;
DEALLOCATE PREPARE niffler_settlement_snapshots_created_at_index_stmt;

SET @niffler_settlement_snapshots_key_time_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'niffler_settlement_snapshots'
          AND index_name = 'idx_niffler_settlement_snapshots_key_time'
    ) = 0,
    'CREATE INDEX idx_niffler_settlement_snapshots_key_time ON niffler_settlement_snapshots (api_key_id, created_at_unix_ms)',
    'DO 0'
);

PREPARE niffler_settlement_snapshots_key_time_index_stmt FROM @niffler_settlement_snapshots_key_time_index_sql;
EXECUTE niffler_settlement_snapshots_key_time_index_stmt;
DEALLOCATE PREPARE niffler_settlement_snapshots_key_time_index_stmt;

SET @niffler_settlement_snapshots_plan_time_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'niffler_settlement_snapshots'
          AND index_name = 'idx_niffler_settlement_snapshots_plan_time'
    ) = 0,
    'CREATE INDEX idx_niffler_settlement_snapshots_plan_time ON niffler_settlement_snapshots (product_plan_id, created_at_unix_ms)',
    'DO 0'
);

PREPARE niffler_settlement_snapshots_plan_time_index_stmt FROM @niffler_settlement_snapshots_plan_time_index_sql;
EXECUTE niffler_settlement_snapshots_plan_time_index_stmt;
DEALLOCATE PREPARE niffler_settlement_snapshots_plan_time_index_stmt;
