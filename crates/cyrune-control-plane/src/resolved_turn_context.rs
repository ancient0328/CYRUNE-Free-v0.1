#![forbid(unsafe_code)]

use cyrune_core_contract::{ContractError, IoMode, RequestId, RunId, RunKind};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub const SHIPPING_BINDING_ID: &str = "cyrune-free-shipping.v0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedKernelAdapters {
    pub working_store_adapter_id: String,
    pub processing_store_adapter_id: String,
    pub permanent_store_adapter_id: String,
    pub vector_index_adapter_id: String,
    pub embedding_engine_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectedExecutionAdapter {
    pub adapter_id: String,
    pub adapter_version: String,
    pub execution_kind: String,
    pub launcher_path: String,
    pub launcher_sha256: String,
    pub model_id: String,
    pub model_revision_or_digest: String,
    pub default_timeout_s: u64,
    pub allowed_capabilities: Vec<String>,
    pub env_allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    pub turn_timeout_s: u64,
    pub execution_timeout_s: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryStateRoots {
    pub processing_state_root: String,
    pub permanent_state_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingExactPin {
    pub engine_kind: String,
    pub upstream_model_id: String,
    pub upstream_revision: Option<String>,
    pub artifact_set: Vec<String>,
    pub artifact_sha256: BTreeMap<String, String>,
    pub dimensions: u16,
    pub pooling: String,
    pub normalization: String,
    pub prompt_profile: String,
    pub token_limit: u16,
    pub distance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedTurnContext {
    pub version: u8,
    pub request_id: RequestId,
    pub correlation_id: cyrune_core_contract::CorrelationId,
    pub run_id: RunId,
    pub requested_policy_pack_id: String,
    pub requested_binding_id: Option<String>,
    pub policy_pack_id: String,
    pub binding_id: String,
    pub resolved_kernel_adapters: ResolvedKernelAdapters,
    pub embedding_exact_pin: Option<EmbeddingExactPin>,
    pub memory_state_roots: Option<MemoryStateRoots>,
    pub allowed_capabilities: Vec<String>,
    pub sandbox_ref: String,
    pub run_kind: RunKind,
    pub io_mode: IoMode,
    pub selected_execution_adapter: Option<SelectedExecutionAdapter>,
    pub timeout_policy: TimeoutPolicy,
}

impl ResolvedTurnContext {
    #[must_use]
    pub fn is_shipping_binding(&self) -> bool {
        self.binding_id == SHIPPING_BINDING_ID
    }

    pub fn validate(&self) -> Result<(), ContractError> {
        if self.version != 1 {
            return Err(ContractError::InvalidRequest {
                message: "resolved turn context version must be 1".to_string(),
            });
        }
        if self.requested_policy_pack_id.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "requested_policy_pack_id",
            });
        }
        if let Some(requested_binding_id) = &self.requested_binding_id {
            if requested_binding_id.trim().is_empty() {
                return Err(ContractError::EmptyField {
                    field: "requested_binding_id",
                });
            }
        }
        if self.policy_pack_id.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "policy_pack_id",
            });
        }
        if self.binding_id.trim().is_empty() {
            return Err(ContractError::EmptyField {
                field: "binding_id",
            });
        }
        if self.timeout_policy.turn_timeout_s == 0 || self.timeout_policy.execution_timeout_s == 0 {
            return Err(ContractError::InvalidRequest {
                message: "timeout_policy must be positive".to_string(),
            });
        }
        match &self.memory_state_roots {
            Some(roots) if self.is_shipping_binding() => {
                validate_absolute_root(&roots.processing_state_root, "processing_state_root")?;
                validate_absolute_root(&roots.permanent_state_root, "permanent_state_root")?;
            }
            Some(_) => {
                return Err(ContractError::InvalidRequest {
                    message: "memory_state_roots is reserved for shipping binding".to_string(),
                });
            }
            None if self.is_shipping_binding() => {
                return Err(ContractError::InvalidRequest {
                    message: "memory_state_roots is required for shipping binding".to_string(),
                });
            }
            None => {}
        }
        match &self.embedding_exact_pin {
            Some(pin) if self.is_shipping_binding() => validate_embedding_exact_pin(pin)?,
            Some(_) => {
                return Err(ContractError::InvalidRequest {
                    message: "embedding_exact_pin is reserved for shipping binding".to_string(),
                });
            }
            None if self.is_shipping_binding() => {
                return Err(ContractError::InvalidRequest {
                    message: "embedding_exact_pin is required for shipping binding".to_string(),
                });
            }
            None => {}
        }
        if !self
            .allowed_capabilities
            .iter()
            .all(|capability| is_allowed_capability(capability))
        {
            return Err(ContractError::InvalidRequest {
                message: "allowed_capabilities contains value outside closed set".to_string(),
            });
        }
        match (&self.run_kind, &self.selected_execution_adapter) {
            (RunKind::NoLlm, None) => {}
            (RunKind::ExecutionAdapter, Some(selected)) => {
                if selected.default_timeout_s == 0 || selected.launcher_path.trim().is_empty() {
                    return Err(ContractError::InvalidRequest {
                        message: "selected execution adapter launcher_path/timeout must be valid"
                            .to_string(),
                    });
                }
            }
            (RunKind::NoLlm, Some(_)) => {
                return Err(ContractError::InvalidRequest {
                    message: "selected_execution_adapter must be null for no_llm".to_string(),
                });
            }
            (RunKind::ExecutionAdapter, None) => {
                return Err(ContractError::InvalidRequest {
                    message: "selected_execution_adapter is required for execution_adapter"
                        .to_string(),
                });
            }
        }
        Ok(())
    }
}

fn validate_absolute_root(root: &str, field: &'static str) -> Result<(), ContractError> {
    if root.trim().is_empty() {
        return Err(ContractError::EmptyField { field });
    }
    if !Path::new(root).is_absolute() {
        return Err(ContractError::InvalidRequest {
            message: format!("{field} must be an absolute path"),
        });
    }
    Ok(())
}

fn validate_embedding_exact_pin(pin: &EmbeddingExactPin) -> Result<(), ContractError> {
    if pin.engine_kind.trim().is_empty()
        || pin.upstream_model_id.trim().is_empty()
        || pin.prompt_profile.trim().is_empty()
        || pin.pooling.trim().is_empty()
        || pin.normalization.trim().is_empty()
        || pin.distance.trim().is_empty()
    {
        return Err(ContractError::InvalidRequest {
            message: "embedding_exact_pin requires non-empty semantic tuple fields".to_string(),
        });
    }
    if pin.dimensions == 0 || pin.token_limit == 0 || pin.artifact_set.is_empty() {
        return Err(ContractError::InvalidRequest {
            message:
                "embedding_exact_pin requires positive dimensions/token_limit and artifact_set"
                    .to_string(),
        });
    }
    Ok(())
}

#[must_use]
pub fn is_allowed_capability(capability: &str) -> bool {
    matches!(
        capability,
        "exec" | "fs_read" | "fs_write" | "git_write" | "net" | "browser" | "ci"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        EmbeddingExactPin, MemoryStateRoots, ResolvedKernelAdapters, ResolvedTurnContext,
        SHIPPING_BINDING_ID, TimeoutPolicy,
    };
    use cyrune_core_contract::{ContractError, CorrelationId, IoMode, RequestId, RunId, RunKind};

    fn base_context() -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260407-0001").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260407-0001").unwrap(),
            run_id: RunId::parse("RUN-20260407-0001-R01").unwrap(),
            requested_policy_pack_id: "cyrune-free-default".to_string(),
            requested_binding_id: None,
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: "cyrune-free-default".to_string(),
            resolved_kernel_adapters: ResolvedKernelAdapters {
                working_store_adapter_id: "memory-kv-inmem".to_string(),
                processing_store_adapter_id: "memory-kv-inmem".to_string(),
                permanent_store_adapter_id: "memory-kv-inmem".to_string(),
                vector_index_adapter_id: "memory-kv-inmem".to_string(),
                embedding_engine_ref: "crane-embed-null.v0.1".to_string(),
            },
            embedding_exact_pin: None,
            memory_state_roots: None,
            allowed_capabilities: vec!["fs_read".to_string()],
            sandbox_ref: "SANDBOX_MINIMAL_CANONICAL.md#default-profile".to_string(),
            run_kind: RunKind::NoLlm,
            io_mode: IoMode::Captured,
            selected_execution_adapter: None,
            timeout_policy: TimeoutPolicy {
                turn_timeout_s: 120,
                execution_timeout_s: 120,
            },
        }
    }

    fn shipping_embedding_exact_pin() -> EmbeddingExactPin {
        EmbeddingExactPin {
            engine_kind: "onnx-local".to_string(),
            upstream_model_id: "intfloat/multilingual-e5-small".to_string(),
            upstream_revision: None,
            artifact_set: vec![
                "model.onnx".to_string(),
                "tokenizer.json".to_string(),
                "config.json".to_string(),
                "special_tokens_map.json".to_string(),
                "tokenizer_config.json".to_string(),
            ],
            artifact_sha256: std::collections::BTreeMap::new(),
            dimensions: 384,
            pooling: "mean".to_string(),
            normalization: "l2_unit".to_string(),
            prompt_profile: "e5_query_passage_v1".to_string(),
            token_limit: 512,
            distance: "cosine".to_string(),
        }
    }

    #[test]
    fn non_shipping_context_rejects_memory_state_roots() {
        let mut context = base_context();
        context.memory_state_roots = Some(MemoryStateRoots {
            processing_state_root: "/tmp/cyrune/processing".to_string(),
            permanent_state_root: "/tmp/cyrune/permanent".to_string(),
        });

        let error = context.validate().unwrap_err();
        assert!(matches!(
            error,
            ContractError::InvalidRequest { message }
                if message.contains("memory_state_roots is reserved for shipping binding")
        ));
    }

    #[test]
    fn non_shipping_context_rejects_embedding_exact_pin() {
        let mut context = base_context();
        context.embedding_exact_pin = Some(shipping_embedding_exact_pin());

        let error = context.validate().unwrap_err();
        assert!(matches!(
            error,
            ContractError::InvalidRequest { message }
                if message.contains("embedding_exact_pin is reserved for shipping binding")
        ));
    }

    #[test]
    fn shipping_context_requires_embedding_exact_pin() {
        let mut context = base_context();
        context.binding_id = SHIPPING_BINDING_ID.to_string();
        context.requested_binding_id = Some(SHIPPING_BINDING_ID.to_string());
        context.resolved_kernel_adapters.processing_store_adapter_id =
            "memory-redb-processing".to_string();
        context.resolved_kernel_adapters.permanent_store_adapter_id =
            "memory-stoolap-permanent".to_string();
        context.memory_state_roots = Some(MemoryStateRoots {
            processing_state_root: "/tmp/cyrune/processing".to_string(),
            permanent_state_root: "/tmp/cyrune/permanent".to_string(),
        });

        let error = context.validate().unwrap_err();
        assert!(matches!(
            error,
            ContractError::InvalidRequest { message }
                if message.contains("embedding_exact_pin is required for shipping binding")
        ));
    }
}
