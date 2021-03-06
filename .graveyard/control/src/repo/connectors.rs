use futures::TryFutureExt;
use sqlx::PgPool;

use crate::models::connectors::{Connector, ConnectorType, NewConnector};
use crate::models::id::Id;

pub async fn fetch_all(db: &PgPool) -> Result<Vec<Connector>, sqlx::Error> {
    sqlx::query_as!(
        Connector,
        r#"
    SELECT id as "id!: Id<Connector>", description, name, maintainer, type as "type!: ConnectorType", created_at, updated_at
    FROM connectors
    "#
    )
    .fetch_all(db)
    .await
}

pub async fn fetch_one(db: &PgPool, id: Id<Connector>) -> Result<Connector, sqlx::Error> {
    sqlx::query_as!(
        Connector,
        r#"
    SELECT id as "id!: Id<Connector>", description, name, maintainer, type as "type!: ConnectorType", created_at, updated_at
    FROM connectors
    WHERE id = $1
    "#,
        id as Id<Connector>
    )
    .fetch_one(db)
    .await
}

pub async fn insert(db: &PgPool, input: NewConnector) -> Result<Connector, sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO connectors(description, name, maintainer, type, created_at, updated_at)
    VALUES ($1, $2, $3, $4, NOW(), NOW())
    RETURNING id as "id!: Id<Connector>"
    "#,
        input.description,
        input.name,
        input.maintainer,
        input.r#type as ConnectorType
    )
    .fetch_one(db)
    .and_then(|row| fetch_one(db, row.id))
    .await
}
