use async_trait::async_trait;
use sqlx::{postgres::PgRow, Postgres, QueryBuilder, Row};

use super::{
    bounded_limit, bounded_offset, i64_from_u64, CreateNifflerProductPlanRecord,
    CreateNifflerUpstreamAccountRecord, CreateNifflerUpstreamServiceRecord, NifflerAccountStatus,
    NifflerCoreReadRepository, NifflerCoreWriteRepository, NifflerProductPlanListQuery,
    NifflerProductPlanModelListQuery, NifflerProtocolKind, NifflerServiceCapabilityKind,
    NifflerUpstreamAccountListQuery, NifflerUpstreamServiceListQuery, StoredNifflerProductPlan,
    StoredNifflerProductPlanListPage, StoredNifflerProductPlanModel,
    StoredNifflerProductPlanModelListPage, StoredNifflerUpstreamAccount,
    StoredNifflerUpstreamAccountListPage, StoredNifflerUpstreamService,
    StoredNifflerUpstreamServiceCapability, StoredNifflerUpstreamServiceListPage,
    UpsertNifflerProductPlanModelRecord, UpsertNifflerUpstreamServiceCapabilityRecord,
};
use crate::driver::postgres::PostgresPool;
use crate::error::SqlResultExt;
use crate::DataLayerError;

#[derive(Debug, Clone)]
pub struct SqlxNifflerCoreRepository {
    pool: PostgresPool,
}

impl SqlxNifflerCoreRepository {
    pub fn new(pool: PostgresPool) -> Self {
        Self { pool }
    }

    async fn reload_service(
        &self,
        service_id: &str,
    ) -> Result<StoredNifflerUpstreamService, DataLayerError> {
        self.find_upstream_service_by_id(service_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream service missing after write".into(),
                )
            })
    }

    async fn reload_account(
        &self,
        account_id: &str,
    ) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, display_name, email, phone, auth_kind, status,
  cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms,
  last_test_error, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_accounts
WHERE id = $1
LIMIT 1
"#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_account_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream account missing after write".into(),
                )
            })
    }

    async fn reload_capability(
        &self,
        capability_id: &str,
    ) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, upstream_service_id, protocol_kind, capability_kind, is_enabled,
  config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_service_capabilities
WHERE id = $1
LIMIT 1
"#,
        )
        .bind(capability_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_capability_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler upstream service capability missing after write".into(),
                )
            })
    }

    async fn reload_product_plan(
        &self,
        product_plan_id: &str,
    ) -> Result<StoredNifflerProductPlan, DataLayerError> {
        self.find_product_plan_by_id(product_plan_id)
            .await?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue("niffler product plan missing after write".into())
            })
    }

    async fn reload_product_plan_model(
        &self,
        product_plan_id: &str,
        model_name: &str,
    ) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, product_plan_id, model_name, is_enabled, sales_multiplier_override,
  created_at_unix_ms, updated_at_unix_ms
FROM niffler_product_plan_models
WHERE product_plan_id = $1 AND model_name = $2
LIMIT 1
"#,
        )
        .bind(product_plan_id)
        .bind(model_name)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref()
            .map(map_product_plan_model_row)
            .transpose()?
            .ok_or_else(|| {
                DataLayerError::UnexpectedValue(
                    "niffler product plan model missing after write".into(),
                )
            })
    }
}

#[async_trait]
impl NifflerCoreReadRepository for SqlxNifflerCoreRepository {
    async fn list_upstream_services(
        &self,
        query: &NifflerUpstreamServiceListQuery,
    ) -> Result<StoredNifflerUpstreamServiceListPage, DataLayerError> {
        let total = build_service_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_service_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_service_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerUpstreamServiceListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_upstream_service_by_id(
        &self,
        upstream_service_id: &str,
    ) -> Result<Option<StoredNifflerUpstreamService>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, display_name, service_kind, default_api_format, base_url,
  cost_multiplier, is_active, config, created_at_unix_ms, updated_at_unix_ms
FROM niffler_upstream_services
WHERE id = $1
LIMIT 1
"#,
        )
        .bind(upstream_service_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_service_row).transpose()
    }

    async fn list_upstream_accounts(
        &self,
        query: &NifflerUpstreamAccountListQuery,
    ) -> Result<StoredNifflerUpstreamAccountListPage, DataLayerError> {
        let total = build_account_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_account_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_account_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerUpstreamAccountListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn list_product_plans(
        &self,
        query: &NifflerProductPlanListQuery,
    ) -> Result<StoredNifflerProductPlanListPage, DataLayerError> {
        let total = build_product_plan_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_product_plan_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_product_plan_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerProductPlanListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }

    async fn find_product_plan_by_id(
        &self,
        product_plan_id: &str,
    ) -> Result<Option<StoredNifflerProductPlan>, DataLayerError> {
        let row = sqlx::query(
            r#"
SELECT
  id, display_name, is_public, is_active, sales_multiplier, description,
  created_at_unix_ms, updated_at_unix_ms
FROM niffler_product_plans
WHERE id = $1
LIMIT 1
"#,
        )
        .bind(product_plan_id)
        .fetch_optional(&self.pool)
        .await
        .map_sql_err()?;
        row.as_ref().map(map_product_plan_row).transpose()
    }

    async fn list_product_plan_models(
        &self,
        query: &NifflerProductPlanModelListQuery,
    ) -> Result<StoredNifflerProductPlanModelListPage, DataLayerError> {
        let total = build_product_plan_model_count_query(query)
            .build_query_scalar::<i64>()
            .fetch_one(&self.pool)
            .await
            .map_sql_err()?;
        let rows = build_product_plan_model_rows_query(query)
            .build()
            .fetch_all(&self.pool)
            .await
            .map_sql_err()?;
        let items = rows
            .iter()
            .map(map_product_plan_model_row)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoredNifflerProductPlanModelListPage {
            items,
            total: usize::try_from(total).unwrap_or_default(),
        })
    }
}

#[async_trait]
impl NifflerCoreWriteRepository for SqlxNifflerCoreRepository {
    async fn create_upstream_service(
        &self,
        record: CreateNifflerUpstreamServiceRecord,
    ) -> Result<StoredNifflerUpstreamService, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_services (
  id, display_name, service_kind, default_api_format, base_url,
  cost_multiplier, is_active, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
"#,
        )
        .bind(&record.id)
        .bind(&record.display_name)
        .bind(&record.service_kind)
        .bind(&record.default_api_format)
        .bind(&record.base_url)
        .bind(record.cost_multiplier)
        .bind(record.is_active)
        .bind(&record.config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_service(&record.id).await
    }

    async fn create_upstream_account(
        &self,
        record: CreateNifflerUpstreamAccountRecord,
    ) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_accounts (
  id, upstream_service_id, display_name, email, phone, auth_kind, status,
  cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms,
  last_test_error, config, created_at_unix_ms, updated_at_unix_ms
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
"#,
        )
        .bind(&record.id)
        .bind(&record.upstream_service_id)
        .bind(&record.display_name)
        .bind(&record.email)
        .bind(&record.phone)
        .bind(&record.auth_kind)
        .bind(record.status.as_str())
        .bind(record.cost_multiplier)
        .bind(record.priority)
        .bind(
            record
                .cooldown_until_unix_ms
                .map(|value| i64_from_u64(value, "cooldown_until_unix_ms"))
                .transpose()?,
        )
        .bind(
            record
                .last_tested_at_unix_ms
                .map(|value| i64_from_u64(value, "last_tested_at_unix_ms"))
                .transpose()?,
        )
        .bind(&record.last_test_error)
        .bind(&record.config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_account(&record.id).await
    }

    async fn upsert_upstream_service_capability(
        &self,
        record: UpsertNifflerUpstreamServiceCapabilityRecord,
    ) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_upstream_service_capabilities (
  id, upstream_service_id, protocol_kind, capability_kind, is_enabled,
  config, created_at_unix_ms, updated_at_unix_ms
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
ON CONFLICT (upstream_service_id, protocol_kind, capability_kind) DO UPDATE SET
  id = EXCLUDED.id,
  is_enabled = EXCLUDED.is_enabled,
  config = EXCLUDED.config,
  updated_at_unix_ms = EXCLUDED.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.upstream_service_id)
        .bind(record.protocol_kind.as_str())
        .bind(record.capability_kind.as_str())
        .bind(record.is_enabled)
        .bind(&record.config)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_capability(&record.id).await
    }

    async fn create_product_plan(
        &self,
        record: CreateNifflerProductPlanRecord,
    ) -> Result<StoredNifflerProductPlan, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_product_plans (
  id, display_name, is_public, is_active, sales_multiplier, description,
  created_at_unix_ms, updated_at_unix_ms
)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
"#,
        )
        .bind(&record.id)
        .bind(&record.display_name)
        .bind(record.is_public)
        .bind(record.is_active)
        .bind(record.sales_multiplier)
        .bind(&record.description)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_product_plan(&record.id).await
    }

    async fn upsert_product_plan_model(
        &self,
        record: UpsertNifflerProductPlanModelRecord,
    ) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
        record.validate()?;
        sqlx::query(
            r#"
INSERT INTO niffler_product_plan_models (
  id, product_plan_id, model_name, is_enabled, sales_multiplier_override,
  created_at_unix_ms, updated_at_unix_ms
)
VALUES ($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT (product_plan_id, model_name) DO UPDATE SET
  is_enabled = EXCLUDED.is_enabled,
  sales_multiplier_override = EXCLUDED.sales_multiplier_override,
  updated_at_unix_ms = EXCLUDED.updated_at_unix_ms
"#,
        )
        .bind(&record.id)
        .bind(&record.product_plan_id)
        .bind(&record.model_name)
        .bind(record.is_enabled)
        .bind(record.sales_multiplier_override)
        .bind(i64_from_u64(
            record.created_at_unix_ms,
            "created_at_unix_ms",
        )?)
        .bind(i64_from_u64(
            record.updated_at_unix_ms,
            "updated_at_unix_ms",
        )?)
        .execute(&self.pool)
        .await
        .map_sql_err()?;
        self.reload_product_plan_model(&record.product_plan_id, &record.model_name)
            .await
    }
}

fn build_service_count_query(
    query: &NifflerUpstreamServiceListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_upstream_services");
    push_service_filters(&mut builder, query);
    builder
}

fn build_service_rows_query(query: &NifflerUpstreamServiceListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new(
        "SELECT id, display_name, service_kind, default_api_format, base_url, cost_multiplier, \
         is_active, config, created_at_unix_ms, updated_at_unix_ms FROM niffler_upstream_services",
    );
    push_service_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC, display_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_service_filters(
    builder: &mut QueryBuilder<'_, Postgres>,
    query: &NifflerUpstreamServiceListQuery,
) {
    let mut has_where = false;
    if !query.include_inactive {
        builder.push(" WHERE is_active = TRUE");
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(display_name ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(" OR service_kind ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(")");
    }
}

fn build_account_count_query(
    query: &NifflerUpstreamAccountListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_upstream_accounts");
    push_account_filters(&mut builder, query);
    builder
}

fn build_account_rows_query(query: &NifflerUpstreamAccountListQuery) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new(
        "SELECT id, upstream_service_id, display_name, email, phone, auth_kind, status, \
         cost_multiplier, priority, cooldown_until_unix_ms, last_tested_at_unix_ms, \
         last_test_error, config, created_at_unix_ms, updated_at_unix_ms \
         FROM niffler_upstream_accounts",
    );
    push_account_filters(&mut builder, query);
    builder.push(" ORDER BY priority ASC, created_at_unix_ms DESC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_account_filters(
    builder: &mut QueryBuilder<'_, Postgres>,
    query: &NifflerUpstreamAccountListQuery,
) {
    let mut has_where = false;
    if let Some(service_id) = query
        .upstream_service_id
        .as_ref()
        .filter(|value| !value.trim().is_empty())
    {
        builder.push(" WHERE upstream_service_id = ");
        builder.push_bind(service_id.clone());
        has_where = true;
    }
    if let Some(status) = query.status {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("status = ");
        builder.push_bind(status.as_str());
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(display_name ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(" OR email ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(" OR phone ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(")");
    }
}

fn build_product_plan_count_query(
    query: &NifflerProductPlanListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_product_plans");
    push_product_plan_filters(&mut builder, query);
    builder
}

fn build_product_plan_rows_query(
    query: &NifflerProductPlanListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new(
        "SELECT id, display_name, is_public, is_active, sales_multiplier, description, \
         created_at_unix_ms, updated_at_unix_ms FROM niffler_product_plans",
    );
    push_product_plan_filters(&mut builder, query);
    builder.push(" ORDER BY created_at_unix_ms DESC, display_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_product_plan_filters(
    builder: &mut QueryBuilder<'_, Postgres>,
    query: &NifflerProductPlanListQuery,
) {
    let mut has_where = false;
    if !query.include_inactive {
        builder.push(" WHERE is_active = TRUE");
        has_where = true;
    }
    if query.public_only {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("is_public = TRUE");
        has_where = true;
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(if has_where { " AND " } else { " WHERE " });
        builder.push("(display_name ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(" OR description ILIKE ");
        builder.push_bind(format!("%{search}%"));
        builder.push(")");
    }
}

fn build_product_plan_model_count_query(
    query: &NifflerProductPlanModelListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new("SELECT COUNT(*) FROM niffler_product_plan_models");
    push_product_plan_model_filters(&mut builder, query);
    builder
}

fn build_product_plan_model_rows_query(
    query: &NifflerProductPlanModelListQuery,
) -> QueryBuilder<'_, Postgres> {
    let mut builder = QueryBuilder::new(
        "SELECT id, product_plan_id, model_name, is_enabled, sales_multiplier_override, \
         created_at_unix_ms, updated_at_unix_ms FROM niffler_product_plan_models",
    );
    push_product_plan_model_filters(&mut builder, query);
    builder.push(" ORDER BY model_name ASC LIMIT ");
    builder.push_bind(bounded_limit(query.limit));
    builder.push(" OFFSET ");
    builder.push_bind(bounded_offset(query.offset));
    builder
}

fn push_product_plan_model_filters(
    builder: &mut QueryBuilder<'_, Postgres>,
    query: &NifflerProductPlanModelListQuery,
) {
    builder.push(" WHERE product_plan_id = ");
    builder.push_bind(query.product_plan_id.clone());
    if query.enabled_only {
        builder.push(" AND is_enabled = TRUE");
    }
    if let Some(search) = query
        .search
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        builder.push(" AND model_name ILIKE ");
        builder.push_bind(format!("%{search}%"));
    }
}

fn map_service_row(row: &PgRow) -> Result<StoredNifflerUpstreamService, DataLayerError> {
    Ok(StoredNifflerUpstreamService {
        id: row.try_get("id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        service_kind: row.try_get("service_kind").map_sql_err()?,
        default_api_format: row.try_get("default_api_format").map_sql_err()?,
        base_url: row.try_get("base_url").map_sql_err()?,
        cost_multiplier: row.try_get("cost_multiplier").map_sql_err()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        config: row.try_get("config").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_account_row(row: &PgRow) -> Result<StoredNifflerUpstreamAccount, DataLayerError> {
    let status: String = row.try_get("status").map_sql_err()?;
    Ok(StoredNifflerUpstreamAccount {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        email: row.try_get("email").map_sql_err()?,
        phone: row.try_get("phone").map_sql_err()?,
        auth_kind: row.try_get("auth_kind").map_sql_err()?,
        status: NifflerAccountStatus::from_database(&status)?,
        cost_multiplier: row.try_get("cost_multiplier").map_sql_err()?,
        priority: row.try_get("priority").map_sql_err()?,
        cooldown_until_unix_ms: row
            .try_get::<Option<i64>, _>("cooldown_until_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "cooldown_until_unix_ms"))
            .transpose()?,
        last_tested_at_unix_ms: row
            .try_get::<Option<i64>, _>("last_tested_at_unix_ms")
            .map_sql_err()?
            .map(|value| super::u64_from_i64(value, "last_tested_at_unix_ms"))
            .transpose()?,
        last_test_error: row.try_get("last_test_error").map_sql_err()?,
        config: row.try_get("config").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_product_plan_row(row: &PgRow) -> Result<StoredNifflerProductPlan, DataLayerError> {
    Ok(StoredNifflerProductPlan {
        id: row.try_get("id").map_sql_err()?,
        display_name: row.try_get("display_name").map_sql_err()?,
        is_public: row.try_get("is_public").map_sql_err()?,
        is_active: row.try_get("is_active").map_sql_err()?,
        sales_multiplier: row.try_get("sales_multiplier").map_sql_err()?,
        description: row.try_get("description").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_product_plan_model_row(
    row: &PgRow,
) -> Result<StoredNifflerProductPlanModel, DataLayerError> {
    Ok(StoredNifflerProductPlanModel {
        id: row.try_get("id").map_sql_err()?,
        product_plan_id: row.try_get("product_plan_id").map_sql_err()?,
        model_name: row.try_get("model_name").map_sql_err()?,
        is_enabled: row.try_get("is_enabled").map_sql_err()?,
        sales_multiplier_override: row.try_get("sales_multiplier_override").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}

fn map_capability_row(
    row: &PgRow,
) -> Result<StoredNifflerUpstreamServiceCapability, DataLayerError> {
    let protocol_kind: String = row.try_get("protocol_kind").map_sql_err()?;
    let capability_kind: String = row.try_get("capability_kind").map_sql_err()?;
    Ok(StoredNifflerUpstreamServiceCapability {
        id: row.try_get("id").map_sql_err()?,
        upstream_service_id: row.try_get("upstream_service_id").map_sql_err()?,
        protocol_kind: NifflerProtocolKind::from_database(&protocol_kind)?,
        capability_kind: NifflerServiceCapabilityKind::from_database(&capability_kind)?,
        is_enabled: row.try_get("is_enabled").map_sql_err()?,
        config: row.try_get("config").map_sql_err()?,
        created_at_unix_ms: super::u64_from_i64(
            row.try_get("created_at_unix_ms").map_sql_err()?,
            "created_at_unix_ms",
        )?,
        updated_at_unix_ms: super::u64_from_i64(
            row.try_get("updated_at_unix_ms").map_sql_err()?,
            "updated_at_unix_ms",
        )?,
    })
}
