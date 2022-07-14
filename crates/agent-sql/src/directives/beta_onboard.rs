use sqlx::types::Uuid;

pub async fn tenant_exists(
    tenant: &str,
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<bool> {
    let prefix = format!("{tenant}/");

    // A tenant is defined as "used" if it overlaps with an object
    // role of either user => role or role => role grant.
    let exists = sqlx::query!(
        r#"
        select 1 as "exists" from user_grants
        where starts_with(lower($1), lower(object_role)) or
              starts_with(lower(object_role), lower($1))
        union all
        select 1 from role_grants
        where starts_with(lower($1), lower(object_role)) or
              starts_with(lower(object_role), lower($1))
        "#,
        prefix as String,
    )
    .fetch_optional(txn)
    .await?;

    Ok(exists.is_some())
}

pub async fn is_user_provisioned(
    user_id: Uuid,
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<bool> {
    let exists = sqlx::query!(
        r#"
        select 1 as "exists" from user_grants
        where user_id = $1
        "#,
        user_id as Uuid,
    )
    .fetch_optional(txn)
    .await?;

    Ok(exists.is_some())
}

pub async fn provision_user(
    user_id: Uuid,
    tenant: &str,
    detail: Option<String>,
    ops_logs_spec: &serde_json::Value,
    ops_stats_spec: &serde_json::Value,
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> sqlx::Result<()> {
    let prefix = format!("{tenant}/");

    sqlx::query!(
        r#"
    with s1 as (
      insert into user_grants (user_id, object_role, capability, detail) values
        ($1, $2, 'admin', $3)
    ),
    s2 as (
      insert into role_grants (subject_role, object_role, capability, detail) values
        ($2, 'ops/' || $2, 'read', $3)
    ),
    s3 as (
      insert into storage_mappings (catalog_prefix, spec, detail) values
        ($2, '{"stores": [{"provider": "GCS", "bucket": "estuary-trial"}]}', $3),
        ('recovery/' || $2, '{"stores": [{"provider": "GCS", "bucket": "estuary-trial"}]}', $3)
    )
    insert into live_specs (catalog_name, last_build_id, last_pub_id, spec, spec_type) values
      ('ops/' || $2 || 'logs',  '00:00:00:00:00:00:00:00', '00:00:00:00:00:00:00:00', $4, 'collection'),
      ('ops/' || $2 || 'stats', '00:00:00:00:00:00:00:00', '00:00:00:00:00:00:00:00', $5, 'collection')
    "#,
        user_id as Uuid,
        prefix as String,
        detail.clone() as Option<String>,
        ops_logs_spec as &serde_json::Value,
        ops_stats_spec as &serde_json::Value,
    )
    .execute(txn)
    .await?;

    Ok(())
}
