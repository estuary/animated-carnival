use super::{extract, JobStatus};

use agent_sql::directives::Row;
use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Directive {}

#[derive(Deserialize, Validate, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Claims {
    #[validate]
    requested_prefix: models::Prefix,
}

#[tracing::instrument(skip_all, fields(directive, row.claims))]
pub async fn apply(
    directive: Directive,
    row: Row,
    _txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<JobStatus> {
    let (Directive {}, Claims { requested_prefix }) = match extract(directive, &row.user_claims) {
        Err(status) => return Ok(status),
        Ok(ok) => ok,
    };

    if row.catalog_prefix != "ops/" {
        return Ok(JobStatus::invalid_directive(anyhow::anyhow!(
            "BetaOnboard directive must have ops/ catalog prefix, not {}",
            row.catalog_prefix
        )));
    }

    info!(%row.user_id, requested_prefix=%requested_prefix.as_str(), "beta onboard");
    Ok(JobStatus::Success)
}
