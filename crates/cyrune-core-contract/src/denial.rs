#![forbid(unsafe_code)]

use crate::error::ContractError;
use crate::id::DenialId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonKind {
    InvalidRequest,
    PolicyDenied,
    BindingUnresolved,
    WorkingInvalid,
    CitationDenied,
    ExecutionFailed,
    LedgerCommitFailed,
    WorkingUpdateFailed,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RuleId(String);

impl RuleId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        let (prefix, digits) =
            value
                .split_once('-')
                .ok_or_else(|| ContractError::InvalidIdentifier {
                    field: "rule_id",
                    value: value.clone(),
                })?;
        let valid_prefix = matches!(
            prefix,
            "REQ" | "POL" | "BND" | "WRK" | "CIT" | "EXE" | "LDG" | "WUP" | "INT"
        );
        if !valid_prefix || digits.len() < 3 || !digits.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(ContractError::InvalidIdentifier {
                field: "rule_id",
                value,
            });
        }
        Ok(Self(format!("{prefix}-{digits}")))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        self.0.split('-').next().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Denial {
    pub denial_id: DenialId,
    pub rule_id: RuleId,
    pub reason_kind: ReasonKind,
    pub message: String,
    pub remediation: String,
}

impl Denial {
    pub fn new(
        denial_id: DenialId,
        rule_id: RuleId,
        reason_kind: ReasonKind,
        message: impl Into<String>,
        remediation: impl Into<String>,
    ) -> Result<Self, ContractError> {
        let message = message.into();
        let remediation = remediation.into();
        if message.trim().is_empty() {
            return Err(ContractError::EmptyField { field: "message" });
        }
        if remediation.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "remediation",
            });
        }
        Ok(Self {
            denial_id,
            rule_id,
            reason_kind,
            message,
            remediation,
        })
    }
}
