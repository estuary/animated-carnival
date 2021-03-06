use std::fmt::Display;

use chrono::{DateTime, Utc};
use heck::ToKebabCase;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::models::id::Id;

/// Connectors come in two flavors: source or materialization.
#[derive(Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "TEXT", rename_all = "camelCase")]
pub enum ConnectorType {
    Source,
    Materialization,
}

/// Connectors are Dockerized integrations with external systems. A Connector
/// may have many versions published over time, which are represented as a
/// `ConnectorImage`.
///
/// A `Connector` is a Rust representation of the Postgres database metadata
/// about a connector.
#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct Connector {
    /// When this record was created.
    pub created_at: DateTime<Utc>,
    /// User-facing description of this connector.
    pub description: String,
    /// Primary key for this record.
    pub id: Id<Connector>,
    /// User-facing name for this connector. Meant to be a short alternative to the full image path.
    pub name: String,
    /// User-facing name of who publishes this connector.
    pub maintainer: String,
    /// Connectors must be either a source or materialization.
    pub r#type: ConnectorType,
    /// When this record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl Connector {
    /// The Connector name but kebab-cased.
    ///
    /// eg. "Postgres" => "postgres"
    /// eg. "Hello World" => "hello-world"
    pub fn codename(&self) -> String {
        self.name.to_kebab_case()
    }

    pub fn is_materialization(&self) -> bool {
        matches!(self.r#type, ConnectorType::Materialization)
    }

    pub fn is_source(&self) -> bool {
        matches!(self.r#type, ConnectorType::Source)
    }

    /// Checks whether or not this connector can support a specific operation.
    pub fn supports(&self, operation: ConnectorOperation) -> bool {
        match operation {
            ConnectorOperation::Spec => true,
            ConnectorOperation::Discover => self.is_source(),
        }
    }
}

/// `NewConnector` represents the data required to insert a new `Connector`,
/// with remaining fields from `Connector` generated by Postgres.
#[derive(Debug, Deserialize, FromRow, Serialize)]
pub struct NewConnector {
    /// User-facing description of this connector.
    pub description: String,
    /// User-facing name for this connector. Meant to be a short alternative to the full image path.
    pub name: String,
    /// User-facing name of who publishes this connector.
    pub maintainer: String,
    /// Connectors must be either a source or materialization.
    pub r#type: ConnectorType,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConnectorOperation {
    Spec,
    Discover,
}

impl Display for ConnectorOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectorOperation::Spec => write!(f, "spec"),
            ConnectorOperation::Discover => write!(f, "discover"),
        }
    }
}
