use base64::display::Base64Display;
use futures::TryFutureExt;
use rand::RngCore;
use sqlx::PgPool;

use crate::models::accounts::Account;
use crate::models::credentials::{Credential, NewCredential};
use crate::models::id::Id;

pub async fn fetch_all(db: &PgPool) -> Result<Vec<Credential>, sqlx::Error> {
    sqlx::query_as!(
        Credential,
        r#"
    SELECT id as "id!: Id<Credential>",
           account_id as "account_id!: Id<Account>",
           expires_at,
           issuer,
           last_authorized_at,
           session_token,
           subject,
           created_at,
           updated_at
    FROM credentials
    "#
    )
    .fetch_all(db)
    .await
}

pub async fn fetch_one(db: &PgPool, id: Id<Credential>) -> Result<Credential, sqlx::Error> {
    sqlx::query_as!(
        Credential,
        r#"
    SELECT id as "id!: Id<Credential>",
           account_id as "account_id!: Id<Account>",
           expires_at,
           issuer,
           last_authorized_at,
           session_token,
           subject,
           created_at,
           updated_at
    FROM credentials
    WHERE id = $1
    "#,
        id as Id<Credential>
    )
    .fetch_one(db)
    .await
}

pub async fn insert(db: &PgPool, input: NewCredential) -> Result<Credential, sqlx::Error> {
    let session_token = random_token();

    sqlx::query!(
        r#"
    INSERT INTO credentials(
        account_id,
        expires_at,
        issuer,
        last_authorized_at,
        session_token,
        subject,
        created_at,
        updated_at
    )
    VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
    RETURNING id as "id!: Id<Credential>"
    "#,
        input.account_id as Id<Account>,
        input.expires_at,
        input.issuer,
        input.last_authorized_at,
        session_token,
        input.subject,
    )
    .fetch_one(db)
    .and_then(|row| fetch_one(db, row.id))
    .await
}

fn random_token() -> String {
    let mut bytes: [u8; 32] = [0; 32];
    rand::thread_rng().fill_bytes(&mut bytes[..]);
    Base64Display::with_config(bytes.as_ref(), base64::URL_SAFE_NO_PAD).to_string()
}

pub async fn find_by_account(
    db: &PgPool,
    account_id: Id<Account>,
) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as!(
        Credential,
        r#"
    SELECT id as "id!: Id<Credential>",
           account_id as "account_id!: Id<Account>",
           expires_at,
           issuer,
           last_authorized_at,
           session_token,
           subject,
           created_at,
           updated_at
    FROM credentials
    WHERE account_id = $1
    LIMIT 1
    "#,
        account_id as Id<Account>
    )
    .fetch_optional(db)
    .await
}

pub async fn fetch_by_account_and_session_token(
    db: &PgPool,
    account_id: Id<Account>,
    session_token: &str,
) -> Result<Credential, sqlx::Error> {
    sqlx::query_as!(
        Credential,
        r#"
    SELECT id as "id!: Id<Credential>",
           account_id as "account_id!: Id<Account>",
           expires_at,
           issuer,
           last_authorized_at,
           session_token,
           subject,
           created_at,
           updated_at
    FROM credentials
    WHERE account_id = $1 AND session_token = $2
    LIMIT 1
    "#,
        account_id as Id<Account>,
        session_token,
    )
    .fetch_one(db)
    .await
}
