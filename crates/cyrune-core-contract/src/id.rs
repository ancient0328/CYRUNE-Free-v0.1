#![forbid(unsafe_code)]

use crate::error::ContractError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RequestId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CorrelationId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RunId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvidenceId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DenialId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CitationBundleId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClaimId(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SlotId(String);

macro_rules! impl_id_accessors {
    ($name:ident) => {
        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

impl_id_accessors!(RequestId);
impl_id_accessors!(CorrelationId);
impl_id_accessors!(RunId);
impl_id_accessors!(EvidenceId);
impl_id_accessors!(DenialId);
impl_id_accessors!(CitationBundleId);
impl_id_accessors!(ClaimId);
impl_id_accessors!(SlotId);

impl RequestId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_dated_id(&value, "REQ-", 4, "request_id")?;
        Ok(Self(value))
    }
}

impl CorrelationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_dated_id(&value, "RUN-", 4, "correlation_id")?;
        Ok(Self(value))
    }
}

impl RunId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_run_id(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn for_single_run(correlation_id: &CorrelationId) -> Self {
        Self(format!("{}-R01", correlation_id.as_str()))
    }

    pub fn ensure_matches_single_run(
        correlation_id: &CorrelationId,
        run_id: &RunId,
    ) -> Result<(), ContractError> {
        let expected = Self::for_single_run(correlation_id);
        if expected != *run_id {
            return Err(ContractError::InvalidIdentifier {
                field: "run_id",
                value: run_id.0.clone(),
            });
        }
        Ok(())
    }
}

impl EvidenceId {
    #[must_use]
    pub fn new(value: u64) -> Self {
        Self(format!("EVID-{value}"))
    }

    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_u64_id(&value, "EVID-", "evidence_id")?;
        Ok(Self(value))
    }
}

impl DenialId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_u64_id(&value, "DENY-", "denial_id")?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn from_evidence_id(evidence_id: &EvidenceId) -> Self {
        let numeric = evidence_id
            .as_str()
            .strip_prefix("EVID-")
            .expect("validated evidence_id must start with EVID-");
        Self(format!("DENY-{numeric}"))
    }
}

impl CitationBundleId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_dated_id(&value, "CB-", 4, "citation_bundle_id")?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn from_correlation_id(correlation_id: &CorrelationId) -> Self {
        let suffix = correlation_id
            .as_str()
            .strip_prefix("RUN-")
            .expect("validated correlation_id must start with RUN-");
        Self(format!("CB-{suffix}"))
    }
}

impl ClaimId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_fixed_numeric_id(&value, "CLM-", 3, "claim_id")?;
        Ok(Self(value))
    }
}

impl SlotId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ContractError> {
        let value = value.into();
        validate_fixed_numeric_id(&value, "W-", 3, "slot_id")?;
        Ok(Self(value))
    }
}

fn validate_dated_id(
    value: &str,
    prefix: &str,
    serial_digits: usize,
    field: &'static str,
) -> Result<(), ContractError> {
    let rest = value
        .strip_prefix(prefix)
        .ok_or_else(|| invalid_id(field, value))?;
    let mut parts = rest.split('-');
    let date = parts.next().ok_or_else(|| invalid_id(field, value))?;
    let serial = parts.next().ok_or_else(|| invalid_id(field, value))?;
    if parts.next().is_some()
        || date.len() != 8
        || !date.chars().all(|ch| ch.is_ascii_digit())
        || serial.len() != serial_digits
        || !serial.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(invalid_id(field, value));
    }
    Ok(())
}

fn validate_run_id(value: &str) -> Result<(), ContractError> {
    let rest = value
        .strip_prefix("RUN-")
        .ok_or_else(|| invalid_id("run_id", value))?;
    let mut parts = rest.split('-');
    let date = parts.next().ok_or_else(|| invalid_id("run_id", value))?;
    let serial = parts.next().ok_or_else(|| invalid_id("run_id", value))?;
    let run_part = parts.next().ok_or_else(|| invalid_id("run_id", value))?;
    if parts.next().is_some()
        || date.len() != 8
        || !date.chars().all(|ch| ch.is_ascii_digit())
        || serial.len() != 4
        || !serial.chars().all(|ch| ch.is_ascii_digit())
        || run_part.len() != 3
        || !run_part.starts_with('R')
        || !run_part[1..].chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(invalid_id("run_id", value));
    }
    Ok(())
}

fn validate_u64_id(value: &str, prefix: &str, field: &'static str) -> Result<(), ContractError> {
    let rest = value
        .strip_prefix(prefix)
        .ok_or_else(|| invalid_id(field, value))?;
    if rest.is_empty() || !rest.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(invalid_id(field, value));
    }
    Ok(())
}

fn validate_fixed_numeric_id(
    value: &str,
    prefix: &str,
    digits: usize,
    field: &'static str,
) -> Result<(), ContractError> {
    let rest = value
        .strip_prefix(prefix)
        .ok_or_else(|| invalid_id(field, value))?;
    if rest.len() != digits || !rest.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(invalid_id(field, value));
    }
    Ok(())
}

fn invalid_id(field: &'static str, value: &str) -> ContractError {
    ContractError::InvalidIdentifier {
        field,
        value: value.to_string(),
    }
}
