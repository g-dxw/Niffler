SET @niffler_route_attempts_created_at_index_sql := IF(
    (
        SELECT COUNT(*)
        FROM information_schema.statistics
        WHERE table_schema = DATABASE()
          AND table_name = 'niffler_route_attempts'
          AND index_name = 'idx_niffler_route_attempts_created_at'
    ) = 0,
    'CREATE INDEX idx_niffler_route_attempts_created_at ON niffler_route_attempts (created_at_unix_ms)',
    'DO 0'
);

PREPARE niffler_route_attempts_created_at_index_stmt FROM @niffler_route_attempts_created_at_index_sql;
EXECUTE niffler_route_attempts_created_at_index_stmt;
DEALLOCATE PREPARE niffler_route_attempts_created_at_index_stmt;
