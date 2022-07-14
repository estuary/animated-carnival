use super::{extract, JobStatus};

use serde::{Deserialize, Serialize};
use tracing::info;
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Directive {}

#[derive(Deserialize, Validate, schemars::JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Claims {
    // TODO(johnny): Introduce models::Tenant, also using TOKEN_RE.
    #[validate]
    requested_tenant: models::PartitionField,
}

#[tracing::instrument(skip_all, fields(directive, row.claims))]
pub async fn apply(
    directive: Directive,
    row: agent_sql::directives::Row,
    txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> anyhow::Result<JobStatus> {
    let (Directive {}, Claims { requested_tenant }) = match extract(directive, &row.user_claims) {
        Err(status) => return Ok(status),
        Ok(ok) => ok,
    };

    if row.catalog_prefix != "ops/" {
        return Ok(JobStatus::invalid_directive(anyhow::anyhow!(
            "BetaOnboard directive must have ops/ catalog prefix, not {}",
            row.catalog_prefix
        )));
    }
    if agent_sql::directives::beta_onboard::is_user_provisioned(row.user_id, &mut *txn).await? {
        return Ok(JobStatus::invalid_claims(anyhow::anyhow!(
            "Cannot provision a new tenant because the user has existing grants",
        )));
    }
    // Check that `requested_tenant` has no overlap with extant user grants.
    if agent_sql::directives::beta_onboard::tenant_exists(&requested_tenant, &mut *txn).await? {
        return Ok(JobStatus::invalid_claims(anyhow::anyhow!(
            "requested tenant {} is not available",
            requested_tenant.as_str()
        )));
    }

    info!(%row.user_id, requested_tenant=%requested_tenant.as_str(), "beta onboard");
    Ok(JobStatus::Success)
}
