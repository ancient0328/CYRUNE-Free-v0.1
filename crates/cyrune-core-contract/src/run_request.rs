#![forbid(unsafe_code)]

use crate::error::ContractError;
use crate::id::{CorrelationId, RequestId};
use crate::path_label::PathLabel;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    NoLlm,
    ExecutionAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoMode {
    Captured,
    Quiet,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRequest {
    pub request_id: RequestId,
    pub correlation_id: CorrelationId,
    pub run_kind: RunKind,
    pub user_input: String,
    pub policy_pack_id: String,
    pub binding_id: Option<String>,
    pub requested_capabilities: Vec<String>,
    pub io_mode: IoMode,
    pub adapter_id: Option<String>,
    pub argv: Option<Vec<String>>,
    pub cwd: Option<PathLabel>,
    pub env_overrides: Option<BTreeMap<String, String>>,
}

impl RunRequest {
    pub fn validate_shape(&self) -> Result<(), ContractError> {
        if self.user_input.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "user_input",
            });
        }
        if self.policy_pack_id.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "policy_pack_id",
            });
        }
        if let Some(binding_id) = &self.binding_id {
            if binding_id.trim().is_empty() {
                return Err(ContractError::EmptyField {
                    field: "binding_id",
                });
            }
        }

        match (&self.run_kind, &self.adapter_id) {
            (RunKind::NoLlm, Some(_)) => Err(ContractError::InvalidRequest {
                message: "adapter_id must be absent when run_kind=no_llm".to_string(),
            }),
            (RunKind::ExecutionAdapter, None) => Err(ContractError::InvalidRequest {
                message: "adapter_id is required when run_kind=execution_adapter".to_string(),
            }),
            (RunKind::ExecutionAdapter, Some(adapter_id)) if adapter_id.trim().is_empty() => {
                Err(ContractError::EmptyField {
                    field: "adapter_id",
                })
            }
            _ => Ok(()),
        }
    }
}
