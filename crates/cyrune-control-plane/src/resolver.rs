#![forbid(unsafe_code)]

use crate::execution_registry::{
    RegistryError, RegistryRootMode, materialize_launcher, resolve_selected_execution_adapter,
};
use crate::request::validate_request;
use crate::resolved_turn_context::{
    EmbeddingExactPin, MemoryStateRoots, ResolvedKernelAdapters, ResolvedTurnContext,
    SHIPPING_BINDING_ID, TimeoutPolicy, is_allowed_capability,
};
use adapter_resolver::{
    DistroAdapterBinding, DistroPolicyPack, EffectiveLayer, ResolveError, ResolvedOutput,
    load_binding, load_catalog, load_policy, resolve_configuration,
};
use cyrune_core_contract::{ContractError, RunKind, RunRequest};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

const BRINGUP_EMBEDDING_ENGINE_REF: &str = "crane-embed-null.v0.1";
const SANDBOX_REF: &str = "SANDBOX_MINIMAL_CANONICAL.md#default-profile";
const DEFAULT_TURN_TIMEOUT_S: u64 = 120;
const SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH: &str =
    "embedding/exact-pins/cyrune-free-shipping.v0.1.json";
const SHIPPING_EXACT_PIN_ARTIFACT_PREFIX: &str = "embedding/artifacts";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolverInputs {
    pub cyrune_home: PathBuf,
    pub distribution_root: PathBuf,
    pub bundle_root: PathBuf,
    pub processing_state_root: PathBuf,
    pub permanent_state_root: PathBuf,
    pub catalog_dir: PathBuf,
    pub policy_dir: PathBuf,
    pub policy_path: PathBuf,
    pub binding_dir: PathBuf,
    pub binding_path: PathBuf,
}

impl ResolverInputs {
    #[must_use]
    pub fn new(
        cyrune_home: impl Into<PathBuf>,
        catalog_dir: impl Into<PathBuf>,
        policy_path: impl Into<PathBuf>,
        binding_path: impl Into<PathBuf>,
    ) -> Self {
        let cyrune_home = cyrune_home.into();
        let policy_path = policy_path.into();
        let binding_path = binding_path.into();
        let memory_root = cyrune_home.join("memory");
        Self {
            distribution_root: cyrune_home.clone(),
            bundle_root: cyrune_home.clone(),
            processing_state_root: memory_root.join("processing"),
            permanent_state_root: memory_root.join("permanent"),
            cyrune_home,
            catalog_dir: catalog_dir.into(),
            policy_dir: parent_dir_or_self(&policy_path),
            policy_path,
            binding_dir: parent_dir_or_self(&binding_path),
            binding_path,
        }
    }

    #[must_use]
    pub fn new_packaged(
        cyrune_home: impl Into<PathBuf>,
        distribution_root: impl Into<PathBuf>,
        bundle_root: impl Into<PathBuf>,
        catalog_dir: impl Into<PathBuf>,
        policy_path: impl Into<PathBuf>,
        binding_path: impl Into<PathBuf>,
    ) -> Self {
        let policy_path = policy_path.into();
        let binding_path = binding_path.into();
        let cyrune_home = cyrune_home.into();
        let memory_root = cyrune_home.join("memory");
        Self {
            cyrune_home,
            distribution_root: distribution_root.into(),
            bundle_root: bundle_root.into(),
            processing_state_root: memory_root.join("processing"),
            permanent_state_root: memory_root.join("permanent"),
            catalog_dir: catalog_dir.into(),
            policy_dir: parent_dir_or_self(&policy_path),
            policy_path,
            binding_dir: parent_dir_or_self(&binding_path),
            binding_path,
        }
    }

    #[must_use]
    pub fn requested_binding_path(&self, request: &RunRequest) -> PathBuf {
        match request.binding_id.as_deref() {
            Some(binding_id) => binding_candidates(&self.binding_dir, binding_id)
                .into_iter()
                .find(|path| path.exists())
                .unwrap_or_else(|| fallback_binding_candidate(&self.binding_dir, binding_id)),
            None => self.binding_path.clone(),
        }
    }

    #[must_use]
    pub fn public_unresolved_binding_id(&self, request: &RunRequest) -> String {
        let binding_path = self.requested_binding_path(request);
        if binding_path.exists() {
            load_binding(&binding_path)
                .map(|binding| binding.binding_id)
                .unwrap_or_else(|_| binding_id_from_path(&binding_path))
        } else {
            binding_id_from_path(&binding_path)
        }
    }

    fn default_binding(&self) -> Result<ResolvedBindingArtifact, ResolverError> {
        Ok(ResolvedBindingArtifact {
            binding: load_binding(&self.binding_path)?,
        })
    }

    fn resolve_binding(
        &self,
        request: &RunRequest,
    ) -> Result<ResolvedBindingArtifact, ResolverError> {
        match request.binding_id.as_deref() {
            Some(binding_id) => resolve_binding_artifact_by_id(&self.binding_dir, binding_id),
            None => self.default_binding(),
        }
    }

    fn resolve_policy(
        &self,
        policy_pack_id: &str,
    ) -> Result<ResolvedPolicyArtifact, ResolverError> {
        resolve_policy_artifact_by_id(&self.policy_dir, policy_pack_id)
    }
}

#[derive(Debug, Error)]
pub enum ResolverError {
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error(transparent)]
    Adapter(#[from] ResolveError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error("{0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct ResolvedExplainedPolicy {
    pub source_path: PathBuf,
    pub policy: DistroPolicyPack,
}

#[derive(Debug, Clone)]
struct ResolvedPolicyArtifact {
    source_path: PathBuf,
    policy: DistroPolicyPack,
}

#[derive(Debug, Clone)]
struct ResolvedBindingArtifact {
    binding: DistroAdapterBinding,
}

#[derive(Debug, Deserialize)]
struct EmbeddingExactPinSourceManifest {
    binding_id: String,
    engine_kind: String,
    upstream_model_id: String,
    upstream_revision: String,
    artifact_set: Vec<String>,
    artifact_sha256: BTreeMap<String, String>,
    artifact_paths: BTreeMap<String, String>,
    dimensions: u16,
    pooling: String,
    normalization: String,
    prompt_profile: String,
    token_limit: u16,
    distance: String,
}

pub fn resolve_turn_context(
    request: &RunRequest,
    inputs: &ResolverInputs,
) -> Result<ResolvedTurnContext, ResolverError> {
    let run_id = validate_request(request)?;

    let catalog = load_catalog(&inputs.catalog_dir)?;
    let policy = inputs.resolve_policy(&request.policy_pack_id)?.policy;
    let binding = inputs.resolve_binding(request)?.binding;
    let resolved = resolve_configuration(&catalog, &policy, &binding)?;

    let allowed_capabilities = normalize_capabilities(&request.requested_capabilities)?;
    let selected_execution_adapter = match request.run_kind {
        RunKind::NoLlm => None,
        RunKind::ExecutionAdapter => {
            let adapter_id = request
                .adapter_id
                .as_deref()
                .ok_or_else(|| ResolverError::Invalid("adapter_id is required".to_string()))?;
            let mut selected = if inputs.bundle_root == inputs.cyrune_home {
                resolve_selected_execution_adapter(
                    &inputs.cyrune_home,
                    RegistryRootMode::CurrentHome,
                    adapter_id,
                )?
            } else {
                resolve_selected_execution_adapter(
                    &inputs.bundle_root,
                    RegistryRootMode::PackagedBundle,
                    adapter_id,
                )?
            };
            if inputs.bundle_root != inputs.cyrune_home {
                let materialized = materialize_launcher(&inputs.bundle_root, &selected)?;
                selected.launcher_path = materialized.launcher_path.display().to_string();
            }
            Some(selected)
        }
    };

    let execution_timeout_s = selected_execution_adapter
        .as_ref()
        .map_or(DEFAULT_TURN_TIMEOUT_S, |selected| {
            selected.default_timeout_s
        });
    let binding_id = binding.binding_id;
    let shipping_exact_pin = if binding_id == SHIPPING_BINDING_ID {
        Some(load_shipping_exact_pin_manifest(inputs)?)
    } else {
        None
    };
    let resolved_kernel_adapters =
        resolved_kernel_adapters(&resolved, shipping_exact_pin.as_ref())?;
    let embedding_exact_pin =
        resolve_embedding_exact_pin(&binding_id, &resolved_kernel_adapters, shipping_exact_pin)?;

    let context = ResolvedTurnContext {
        version: 1,
        request_id: request.request_id.clone(),
        correlation_id: request.correlation_id.clone(),
        run_id,
        requested_policy_pack_id: request.policy_pack_id.clone(),
        requested_binding_id: request.binding_id.clone(),
        policy_pack_id: policy.policy_pack_id,
        binding_id: binding_id.clone(),
        resolved_kernel_adapters,
        embedding_exact_pin,
        memory_state_roots: memory_state_roots(inputs, &binding_id)?,
        allowed_capabilities,
        sandbox_ref: SANDBOX_REF.to_string(),
        run_kind: request.run_kind.clone(),
        io_mode: request.io_mode.clone(),
        selected_execution_adapter,
        timeout_policy: TimeoutPolicy {
            turn_timeout_s: DEFAULT_TURN_TIMEOUT_S,
            execution_timeout_s,
        },
    };
    context.validate()?;
    Ok(context)
}

pub fn resolve_explained_policy(
    requested_policy_pack_id: &str,
    inputs: &ResolverInputs,
) -> Result<ResolvedExplainedPolicy, ResolverError> {
    let resolved_policy = inputs.resolve_policy(requested_policy_pack_id)?;
    Ok(ResolvedExplainedPolicy {
        source_path: resolved_policy.source_path,
        policy: resolved_policy.policy,
    })
}

fn memory_state_roots(
    inputs: &ResolverInputs,
    binding_id: &str,
) -> Result<Option<MemoryStateRoots>, ResolverError> {
    if binding_id != SHIPPING_BINDING_ID {
        return Ok(None);
    }
    Ok(Some(MemoryStateRoots {
        processing_state_root: absolute_path_string(&inputs.processing_state_root)?,
        permanent_state_root: absolute_path_string(&inputs.permanent_state_root)?,
    }))
}

fn resolve_embedding_exact_pin(
    binding_id: &str,
    resolved_kernel_adapters: &ResolvedKernelAdapters,
    shipping_exact_pin: Option<EmbeddingExactPin>,
) -> Result<Option<EmbeddingExactPin>, ResolverError> {
    if binding_id != SHIPPING_BINDING_ID {
        return Ok(None);
    }

    let pin = shipping_exact_pin.ok_or_else(|| {
        resolver_validation(format!(
            "shipping exact pin authoritative source missing: {SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH}"
        ))
    })?;
    let expected_engine_ref = shipping_embedding_engine_ref_for_pin(&pin);
    if resolved_kernel_adapters.embedding_engine_ref != expected_engine_ref {
        return Err(resolver_validation(
            "shipping exact pin authoritative source is present but embedding engine alignment is unresolved"
                .to_string(),
        ));
    }
    Ok(Some(pin))
}

fn load_shipping_exact_pin_manifest(
    inputs: &ResolverInputs,
) -> Result<EmbeddingExactPin, ResolverError> {
    let manifest_path = inputs
        .bundle_root
        .join("embedding")
        .join("exact-pins")
        .join("cyrune-free-shipping.v0.1.json");
    let bytes = match fs::read(&manifest_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(resolver_validation(format!(
                "shipping exact pin authoritative source missing: {SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH}"
            )));
        }
        Err(_) => {
            return Err(resolver_validation(format!(
                "shipping exact pin authoritative source is unreadable: {SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH}"
            )));
        }
    };
    let manifest: EmbeddingExactPinSourceManifest = serde_json::from_slice(&bytes).map_err(|_| {
        resolver_validation(format!(
            "shipping exact pin authoritative source is invalid: {SHIPPING_EXACT_PIN_MANIFEST_RELATIVE_PATH}"
        ))
    })?;
    manifest.into_embedding_exact_pin(&inputs.bundle_root)
}

fn resolver_validation(message: String) -> ResolverError {
    ResolverError::Adapter(ResolveError::Validation(message))
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), ResolverError> {
    if value.trim().is_empty() {
        return Err(resolver_validation(format!(
            "shipping exact pin authoritative source is invalid: empty {field}"
        )));
    }
    Ok(())
}

fn validate_hex_sha256(value: &str, artifact_name: &str) -> Result<(), ResolverError> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(resolver_validation(format!(
            "shipping exact pin authoritative source is invalid: artifact sha256 is malformed: {artifact_name}"
        )));
    }
    Ok(())
}

fn validate_artifact_relative_path(
    artifact_name: &str,
    relative_path: &str,
) -> Result<PathBuf, ResolverError> {
    if relative_path.trim().is_empty() {
        return Err(resolver_validation(format!(
            "shipping exact pin authoritative source is invalid: artifact path is empty: {artifact_name}"
        )));
    }
    let path = PathBuf::from(relative_path);
    if path.is_absolute() {
        return Err(resolver_validation(format!(
            "shipping exact pin authoritative source is invalid: artifact path must be bundle-root-relative: {artifact_name}"
        )));
    }
    let mut components = path.components();
    match components.next() {
        Some(Component::Normal(first)) if first == OsStr::new("embedding") => {}
        _ => {
            return Err(resolver_validation(format!(
                "shipping exact pin authoritative source is invalid: artifact path must stay under {SHIPPING_EXACT_PIN_ARTIFACT_PREFIX}: {artifact_name}"
            )));
        }
    }
    match components.next() {
        Some(Component::Normal(second)) if second == OsStr::new("artifacts") => {}
        _ => {
            return Err(resolver_validation(format!(
                "shipping exact pin authoritative source is invalid: artifact path must stay under {SHIPPING_EXACT_PIN_ARTIFACT_PREFIX}: {artifact_name}"
            )));
        }
    }
    for component in components {
        match component {
            Component::Normal(_) => {}
            _ => {
                return Err(resolver_validation(format!(
                    "shipping exact pin authoritative source is invalid: artifact path escapes bundle root: {artifact_name}"
                )));
            }
        }
    }
    Ok(path)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

impl EmbeddingExactPinSourceManifest {
    fn into_embedding_exact_pin(
        self,
        bundle_root: &Path,
    ) -> Result<EmbeddingExactPin, ResolverError> {
        validate_non_empty(&self.binding_id, "binding_id")?;
        validate_non_empty(&self.engine_kind, "engine_kind")?;
        validate_non_empty(&self.upstream_model_id, "upstream_model_id")?;
        validate_non_empty(&self.upstream_revision, "upstream_revision")?;
        validate_non_empty(&self.pooling, "pooling")?;
        validate_non_empty(&self.normalization, "normalization")?;
        validate_non_empty(&self.prompt_profile, "prompt_profile")?;
        validate_non_empty(&self.distance, "distance")?;
        if self.binding_id != SHIPPING_BINDING_ID {
            return Err(resolver_validation(format!(
                "shipping exact pin authoritative source is invalid: binding_id must be {SHIPPING_BINDING_ID}"
            )));
        }
        if self.dimensions == 0 || self.token_limit == 0 || self.artifact_set.is_empty() {
            return Err(resolver_validation(
                "shipping exact pin authoritative source is invalid: dimensions/token_limit/artifact_set".to_string(),
            ));
        }

        let artifact_set: BTreeSet<&str> = self.artifact_set.iter().map(String::as_str).collect();
        if artifact_set.len() != self.artifact_set.len() {
            return Err(resolver_validation(
                "shipping exact pin authoritative source is invalid: artifact_set must be unique"
                    .to_string(),
            ));
        }
        let path_keys: BTreeSet<&str> = self.artifact_paths.keys().map(String::as_str).collect();
        let hash_keys: BTreeSet<&str> = self.artifact_sha256.keys().map(String::as_str).collect();
        if path_keys != artifact_set {
            return Err(resolver_validation(
                "shipping exact pin authoritative source is invalid: artifact_paths must match artifact_set"
                    .to_string(),
            ));
        }
        if hash_keys != artifact_set {
            return Err(resolver_validation(
                "shipping exact pin authoritative source is invalid: artifact_sha256 must match artifact_set"
                    .to_string(),
            ));
        }

        for artifact_name in &self.artifact_set {
            let relative_path = self.artifact_paths.get(artifact_name).ok_or_else(|| {
                resolver_validation(format!(
                    "shipping exact pin authoritative source is invalid: missing artifact path: {artifact_name}"
                ))
            })?;
            let expected_hash = self.artifact_sha256.get(artifact_name).ok_or_else(|| {
                resolver_validation(format!(
                    "shipping exact pin authoritative source is invalid: missing artifact sha256: {artifact_name}"
                ))
            })?;
            validate_hex_sha256(expected_hash, artifact_name)?;
            let artifact_path = validate_artifact_relative_path(artifact_name, relative_path)?;
            let bytes = fs::read(bundle_root.join(&artifact_path)).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    resolver_validation(format!(
                        "shipping exact pin authoritative source is invalid: artifact missing: {artifact_name}"
                    ))
                } else {
                    resolver_validation(format!(
                        "shipping exact pin authoritative source is invalid: artifact unreadable: {artifact_name}"
                    ))
                }
            })?;
            if sha256_hex(&bytes) != *expected_hash {
                return Err(resolver_validation(format!(
                    "shipping exact pin authoritative source is invalid: artifact hash mismatch: {artifact_name}"
                )));
            }
        }

        Ok(EmbeddingExactPin {
            engine_kind: self.engine_kind,
            upstream_model_id: self.upstream_model_id,
            upstream_revision: Some(self.upstream_revision),
            artifact_set: self.artifact_set,
            artifact_sha256: self.artifact_sha256,
            dimensions: self.dimensions,
            pooling: self.pooling,
            normalization: self.normalization,
            prompt_profile: self.prompt_profile,
            token_limit: self.token_limit,
            distance: self.distance,
        })
    }
}

fn absolute_path_string(path: &Path) -> Result<String, ResolverError> {
    if !path.is_absolute() {
        return Err(ResolverError::Invalid(format!(
            "state root must be absolute: {}",
            path.display()
        )));
    }
    Ok(path.display().to_string())
}

fn binding_id_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("binding.unresolved")
        .to_string()
}

fn parent_dir_or_self(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| path.to_path_buf())
}

fn resolve_binding_artifact_by_id(
    binding_dir: &Path,
    requested_binding_id: &str,
) -> Result<ResolvedBindingArtifact, ResolverError> {
    let mut matched = None;
    for path in json_manifest_paths(binding_dir)? {
        let binding = load_binding(&path)?;
        if binding.binding_id != requested_binding_id {
            continue;
        }
        if matched.is_some() {
            return Err(ResolverError::Adapter(ResolveError::Validation(format!(
                "binding exact match is not unique: {requested_binding_id}"
            ))));
        }
        matched = Some(ResolvedBindingArtifact { binding });
    }
    matched.ok_or_else(|| {
        ResolverError::Adapter(ResolveError::Validation(format!(
            "binding exact match not found: {requested_binding_id}"
        )))
    })
}

fn resolve_policy_artifact_by_id(
    policy_dir: &Path,
    requested_policy_pack_id: &str,
) -> Result<ResolvedPolicyArtifact, ResolverError> {
    let mut matched = None;
    for path in json_manifest_paths(policy_dir)? {
        let policy = load_policy(&path)?;
        if policy.policy_pack_id != requested_policy_pack_id {
            continue;
        }
        if matched.is_some() {
            return Err(ResolverError::Adapter(ResolveError::Validation(format!(
                "policy exact match is not unique: {requested_policy_pack_id}"
            ))));
        }
        matched = Some(ResolvedPolicyArtifact {
            source_path: path,
            policy,
        });
    }
    matched.ok_or_else(|| {
        ResolverError::Adapter(ResolveError::Validation(format!(
            "policy exact match not found: {requested_policy_pack_id}"
        )))
    })
}

fn json_manifest_paths(dir: &Path) -> Result<Vec<PathBuf>, ResolverError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir).map_err(ResolveError::from)? {
        let entry = entry.map_err(ResolveError::from)?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn binding_candidates(binding_dir: &Path, binding_id: &str) -> Vec<PathBuf> {
    if binding_id.ends_with(".json") {
        return vec![binding_dir.join(binding_id)];
    }

    let primary = fallback_binding_candidate(binding_dir, binding_id);
    let alternate = binding_dir.join(format!("{binding_id}.json"));
    if alternate == primary {
        vec![primary]
    } else {
        vec![primary, alternate]
    }
}

fn fallback_binding_candidate(binding_dir: &Path, binding_id: &str) -> PathBuf {
    if binding_id.contains(".v") {
        binding_dir.join(format!("{binding_id}.json"))
    } else {
        binding_dir.join(format!("{binding_id}.v0.1.json"))
    }
}

fn normalize_capabilities(capabilities: &[String]) -> Result<Vec<String>, ResolverError> {
    let mut ordered = BTreeSet::new();
    for capability in capabilities {
        if !is_allowed_capability(capability) {
            return Err(ResolverError::Invalid(format!(
                "capability outside closed set: {capability}"
            )));
        }
        ordered.insert(capability.clone());
    }
    Ok(ordered.into_iter().collect())
}

pub(crate) fn shipping_embedding_engine_ref_for_pin(pin: &EmbeddingExactPin) -> String {
    let model_id = pin
        .upstream_model_id
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '-'
            }
        })
        .collect::<String>();
    let revision = pin
        .upstream_revision
        .as_deref()
        .unwrap_or("unresolved")
        .chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '-'
            }
        })
        .collect::<String>();
    format!(
        "embedding-{}-{}-{}.v0.1",
        pin.engine_kind, model_id, revision
    )
}

fn resolved_kernel_adapters(
    resolved: &ResolvedOutput,
    shipping_exact_pin: Option<&EmbeddingExactPin>,
) -> Result<ResolvedKernelAdapters, ResolverError> {
    Ok(ResolvedKernelAdapters {
        working_store_adapter_id: first_adapter_id(&resolved.effective.layers.working, "working")?,
        processing_store_adapter_id: first_adapter_id(
            &resolved.effective.layers.processing,
            "processing",
        )?,
        permanent_store_adapter_id: first_adapter_id(
            &resolved.effective.layers.permanent,
            "permanent",
        )?,
        vector_index_adapter_id: first_adapter_id(&resolved.effective.layers.working, "vector")?,
        embedding_engine_ref: shipping_exact_pin
            .map(shipping_embedding_engine_ref_for_pin)
            .unwrap_or_else(|| BRINGUP_EMBEDDING_ENGINE_REF.to_string()),
    })
}

fn first_adapter_id(layer: &EffectiveLayer, layer_name: &str) -> Result<String, ResolverError> {
    layer
        .adapter_ids
        .first()
        .cloned()
        .ok_or_else(|| ResolverError::Invalid(format!("no adapter id resolved for {layer_name}")))
}

#[cfg(test)]
mod tests {
    use super::{
        BRINGUP_EMBEDDING_ENGINE_REF, ResolverInputs, resolve_explained_policy,
        resolve_turn_context, shipping_embedding_engine_ref_for_pin,
    };
    use cyrune_core_contract::{
        ContractError, CorrelationId, IoMode, RequestId, RunKind, RunRequest,
    };
    use serde_json::json;
    use sha2::Digest;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    const CATALOG_JSON: &str = r#"{
  "adapter_id": "memory-kv-inmem",
  "version": "v0.1",
  "layers": {
    "working": {
      "limits": { "min_items": 0, "max_items": 64, "min_ttl_ms": 0, "max_ttl_ms": 3600000 },
      "capabilities": {
        "supports_eviction": true,
        "supports_promotion": true,
        "supports_demotion": true,
        "supports_vector_search": true
      },
      "performance_profile": { "read_latency_p95_ms": 1.0, "write_latency_p95_ms": 1.0 }
    },
    "processing": {
      "limits": { "min_items": 0, "max_items": 50000, "min_ttl_ms": 0, "max_ttl_ms": 3628800000 },
      "capabilities": {
        "supports_eviction": true,
        "supports_promotion": true,
        "supports_demotion": true,
        "supports_vector_search": true
      },
      "performance_profile": { "read_latency_p95_ms": 1.0, "write_latency_p95_ms": 1.0 }
    },
    "permanent": {
      "limits": { "min_items": 0, "max_items": 50000, "min_ttl_ms": 0, "max_ttl_ms": 9999999999 },
      "capabilities": {
        "supports_eviction": false,
        "supports_promotion": true,
        "supports_demotion": true,
        "supports_vector_search": true
      },
      "performance_profile": { "read_latency_p95_ms": 1.0, "write_latency_p95_ms": 1.0 }
    }
  }
}"#;

    const POLICY_JSON: &str = r#"{
  "distro_id": "cyrune-free",
  "policy_pack_id": "cyrune-free-default",
  "version": "v0.1",
  "layers": {
    "working": { "target_items": 10, "ttl_ms": 3600000, "eviction_strategy": "priority" },
    "processing": { "target_items": 20000, "ttl_ms": 3628800000, "promotion_threshold": 0.8 },
    "permanent": { "retention_mode": "immutable" }
  },
  "fail_closed": {
    "on_capacity_out_of_range": true,
    "on_ttl_out_of_range": true,
    "on_missing_capability": true
  }
}"#;

    const ALT_POLICY_JSON: &str = r#"{
  "distro_id": "cyrune-free",
  "policy_pack_id": "cyrune-free-alt",
  "version": "v0.1",
  "layers": {
    "working": { "target_items": 10, "ttl_ms": 3600000, "eviction_strategy": "priority" },
    "processing": { "target_items": 20000, "ttl_ms": 3628800000, "promotion_threshold": 0.8 },
    "permanent": { "retention_mode": "immutable" }
  },
  "fail_closed": {
    "on_capacity_out_of_range": true,
    "on_ttl_out_of_range": true,
    "on_missing_capability": true
  }
}"#;

    const BINDING_JSON: &str = r#"{
  "distro_id": "cyrune-free",
  "binding_id": "cyrune-free-default",
  "version": "v0.1",
  "resolution_mode": "single",
  "layers": {
    "working": { "adapter_ids": ["memory-kv-inmem"] },
    "processing": { "adapter_ids": ["memory-kv-inmem"] },
    "permanent": { "adapter_ids": ["memory-kv-inmem"] }
  }
}"#;

    const REGISTRY_JSON: &str = r#"{
  "registry_version": "cyrune.free.execution-adapter-registry.v1",
  "entries": [
    {
      "adapter_id": "local-cli-single-process.v0.1",
      "state": "approved",
      "profile_path": "profiles/local-cli-single-process.v0.1.json"
    }
  ]
}"#;

    fn base_request(run_kind: RunKind, adapter_id: Option<&str>) -> RunRequest {
        RunRequest {
            request_id: RequestId::parse("REQ-20260327-0002").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0002").unwrap(),
            run_kind,
            user_input: "Resolve Free context.".to_string(),
            policy_pack_id: "cyrune-free-default".to_string(),
            binding_id: None,
            requested_capabilities: vec!["fs_read".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: adapter_id.map(ToOwned::to_owned),
            argv: None,
            cwd: None,
            env_overrides: None,
        }
    }

    fn write_fixture(path: &std::path::Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn profile_json(
        launcher_path: &str,
        launcher_sha256: &str,
        model_revision_or_digest: &str,
    ) -> String {
        format!(
            r#"{{
  "adapter_id": "local-cli-single-process.v0.1",
  "adapter_version": "0.1.0",
  "execution_kind": "process_stdio",
  "launcher_path": "{launcher_path}",
  "launcher_sha256": "{launcher_sha256}",
  "model_id": "model.local",
  "model_revision_or_digest": "{model_revision_or_digest}",
  "allowed_capabilities": ["exec", "fs_read"],
  "default_timeout_s": 120,
  "env_allowlist": []
}}"#
        )
    }

    fn write_launcher(path: &Path) -> String {
        write_fixture(path, "#!/bin/sh\nexit 0\n");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
        let bytes = fs::read(path).unwrap();
        let digest = sha2::Sha256::digest(bytes);
        let mut out = String::with_capacity(digest.len() * 2);
        for byte in digest {
            out.push_str(&format!("{byte:02x}"));
        }
        format!("sha256:{out}")
    }

    fn write_registry_fixture(bundle_root: &Path, model_revision_or_digest: &str) {
        let launcher_path = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let launcher_sha256 = write_launcher(&launcher_path);
        let approved_dir = bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved");
        write_fixture(&approved_dir.join("registry.json"), REGISTRY_JSON);
        write_fixture(
            &approved_dir
                .join("profiles")
                .join("local-cli-single-process.v0.1.json"),
            &profile_json(
                "runtime/ipc/local-cli-single-process.sh",
                &launcher_sha256,
                model_revision_or_digest,
            ),
        );
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let digest = sha2::Sha256::digest(bytes);
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn write_shipping_exact_pin_source(bundle_root: &Path, corrupt_hash: bool) {
        let artifact_dir = bundle_root
            .join("embedding")
            .join("artifacts")
            .join("multilingual-e5-small");
        let exact_pin_path = bundle_root
            .join("embedding")
            .join("exact-pins")
            .join("cyrune-free-shipping.v0.1.json");
        let artifact_names = [
            "model.onnx",
            "tokenizer.json",
            "config.json",
            "special_tokens_map.json",
            "tokenizer_config.json",
        ];

        let mut artifact_sha256 = serde_json::Map::new();
        let mut artifact_paths = serde_json::Map::new();
        for artifact_name in artifact_names {
            let bytes = format!("artifact::{artifact_name}").into_bytes();
            let artifact_path = artifact_dir.join(artifact_name);
            write_fixture(&artifact_path, std::str::from_utf8(&bytes).unwrap());
            let hash = if corrupt_hash && artifact_name == "model.onnx" {
                "0".repeat(64)
            } else {
                sha256_hex(&bytes)
            };
            artifact_sha256.insert(artifact_name.to_string(), json!(hash));
            artifact_paths.insert(
                artifact_name.to_string(),
                json!(format!(
                    "embedding/artifacts/multilingual-e5-small/{artifact_name}"
                )),
            );
        }

        write_fixture(
            &exact_pin_path,
            &serde_json::to_string_pretty(&json!({
                "binding_id": "cyrune-free-shipping.v0.1",
                "engine_kind": "onnx-local",
                "upstream_model_id": "intfloat/multilingual-e5-small",
                "upstream_revision": "ffdcc22",
                "artifact_set": artifact_names,
                "artifact_sha256": artifact_sha256,
                "artifact_paths": artifact_paths,
                "dimensions": 384,
                "pooling": "mean",
                "normalization": "l2_unit",
                "prompt_profile": "e5_query_passage_v1",
                "token_limit": 512,
                "distance": "cosine"
            }))
            .unwrap(),
        );
    }

    fn copy_dir_all(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let source = entry.path();
            let target = dst.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_dir_all(&source, &target);
            } else {
                fs::copy(&source, &target).unwrap();
            }
        }
    }

    fn test_inputs() -> ResolverInputs {
        let temp = tempdir().unwrap();
        let base = temp.path().to_path_buf();
        std::mem::forget(temp);
        let cyrune_home = base.join("state-home");
        let distribution_root = base.join("distribution");
        let bundle_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        let adapter_dir = bundle_root.join("adapter");
        let catalog_dir = adapter_dir.join("catalog");
        let crane_root = crane_root();
        let policy_path = adapter_dir
            .join("policies")
            .join("cyrune-free-default.v0.1.json");
        let binding_path = adapter_dir
            .join("bindings")
            .join("cyrune-free-default.v0.1.json");
        copy_dir_all(
            &crane_root
                .join("Adapter")
                .join("v0.1")
                .join("0")
                .join("catalog"),
            &catalog_dir,
        );
        write_fixture(&catalog_dir.join("memory-kv-inmem.v0.1.json"), CATALOG_JSON);
        write_fixture(&policy_path, POLICY_JSON);
        write_fixture(
            &adapter_dir
                .join("policies")
                .join("cyrune-free-alt.v0.1.json"),
            ALT_POLICY_JSON,
        );
        write_fixture(&binding_path, BINDING_JSON);
        write_fixture(
            &adapter_dir
                .join("bindings")
                .join("cyrune-free-shipping.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-shipping.v0.1.json"),
            )
            .unwrap(),
        );
        write_registry_fixture(&bundle_root, "sha256:model");
        ResolverInputs::new_packaged(
            cyrune_home,
            distribution_root,
            bundle_root,
            catalog_dir,
            policy_path,
            binding_path,
        )
    }

    fn crane_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        for ancestor in manifest_dir.ancestors() {
            if ancestor.join("Adapter").join("v0.1").join("0").exists() {
                return ancestor.to_path_buf();
            }
        }
        panic!("CRANE_ROOT could not be derived from CARGO_MANIFEST_DIR");
    }

    fn shipping_inputs() -> ResolverInputs {
        let temp = tempdir().unwrap();
        let base = temp.path().to_path_buf();
        std::mem::forget(temp);
        let cyrune_home = base.join("state-home");
        let distribution_root = base.join("distribution");
        let bundle_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        let adapter_dir = bundle_root.join("adapter");
        let crane_root = crane_root();
        copy_dir_all(
            &crane_root
                .join("Adapter")
                .join("v0.1")
                .join("0")
                .join("catalog"),
            &adapter_dir.join("catalog"),
        );
        write_fixture(
            &adapter_dir
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("policies")
                    .join("cyrune-free-default.v0.1.json"),
            )
            .unwrap(),
        );
        write_fixture(
            &adapter_dir
                .join("policies")
                .join("cyrune-free-alt.v0.1.json"),
            ALT_POLICY_JSON,
        );
        write_fixture(
            &adapter_dir
                .join("bindings")
                .join("cyrune-free-shipping.v0.1.json"),
            &fs::read_to_string(
                crane_root
                    .join("Adapter")
                    .join("v0.1")
                    .join("0")
                    .join("bindings")
                    .join("cyrune-free-shipping.v0.1.json"),
            )
            .unwrap(),
        );
        write_registry_fixture(&bundle_root, "sha256:model");
        ResolverInputs::new_packaged(
            cyrune_home,
            distribution_root,
            bundle_root.clone(),
            adapter_dir.join("catalog"),
            adapter_dir
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            adapter_dir
                .join("bindings")
                .join("cyrune-free-shipping.v0.1.json"),
        )
    }

    #[test]
    fn shipping_inputs_use_bundle_authority_roots() {
        let inputs = shipping_inputs();
        let expected_bundle_root = inputs
            .distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        assert_eq!(inputs.bundle_root, expected_bundle_root);
        assert!(
            inputs
                .catalog_dir
                .starts_with(inputs.bundle_root.join("adapter"))
        );
        assert!(
            inputs
                .policy_path
                .starts_with(inputs.bundle_root.join("adapter"))
        );
        assert!(
            inputs
                .binding_path
                .starts_with(inputs.bundle_root.join("adapter"))
        );
    }

    #[test]
    fn unresolved_binding_is_rejected() {
        let inputs = test_inputs();
        fs::write(
            &inputs.binding_path,
            BINDING_JSON.replace("memory-kv-inmem", "missing-adapter"),
        )
        .unwrap();
        let error = resolve_turn_context(&base_request(RunKind::NoLlm, None), &inputs).unwrap_err();
        assert!(matches!(error, super::ResolverError::Adapter(_)));
    }

    #[test]
    fn unknown_adapter_is_rejected() {
        let inputs = test_inputs();
        let error = resolve_turn_context(
            &base_request(RunKind::ExecutionAdapter, Some("unknown-adapter.v0.1")),
            &inputs,
        )
        .unwrap_err();
        assert!(matches!(error, super::ResolverError::Registry(_)));
    }

    #[test]
    fn snapshot_is_immutable_after_registry_change() {
        let inputs = test_inputs();
        let context = resolve_turn_context(
            &base_request(
                RunKind::ExecutionAdapter,
                Some("local-cli-single-process.v0.1"),
            ),
            &inputs,
        )
        .unwrap();
        let profile_path = inputs
            .bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved")
            .join("profiles")
            .join("local-cli-single-process.v0.1.json");
        fs::write(
            profile_path,
            profile_json(
                "runtime/ipc/local-cli-single-process.sh",
                context
                    .selected_execution_adapter
                    .as_ref()
                    .unwrap()
                    .launcher_sha256
                    .as_str(),
                "sha256:changed-model",
            ),
        )
        .unwrap();
        assert_eq!(
            context
                .selected_execution_adapter
                .as_ref()
                .unwrap()
                .model_revision_or_digest,
            "sha256:model"
        );
        assert!(
            Path::new(
                &context
                    .selected_execution_adapter
                    .as_ref()
                    .unwrap()
                    .launcher_path
            )
            .is_absolute()
        );
    }

    #[test]
    fn no_llm_context_uses_null_selected_execution_adapter() {
        let inputs = test_inputs();
        let context = resolve_turn_context(&base_request(RunKind::NoLlm, None), &inputs).unwrap();
        assert!(context.selected_execution_adapter.is_none());
        assert!(context.embedding_exact_pin.is_none());
        assert!(context.memory_state_roots.is_none());
        assert_eq!(context.run_id.as_str(), "RUN-20260327-0002-R01");
        assert_eq!(context.allowed_capabilities, vec!["fs_read".to_string()]);
        assert_eq!(context.requested_policy_pack_id, "cyrune-free-default");
        assert_eq!(context.requested_binding_id, None);
        assert_eq!(context.policy_pack_id, "cyrune-free-default");
        assert_eq!(context.binding_id, "cyrune-free-default");
    }

    #[test]
    fn explicit_shipping_binding_requires_exact_pin_source() {
        let inputs = test_inputs();
        let mut request = base_request(RunKind::NoLlm, None);
        request.binding_id = Some("cyrune-free-shipping.v0.1".to_string());

        let error = resolve_turn_context(&request, &inputs).unwrap_err();

        assert!(matches!(
            error,
            super::ResolverError::Adapter(adapter_resolver::ResolveError::Validation(message))
                if message.contains("shipping exact pin authoritative source missing")
        ));
    }

    #[test]
    fn explicit_policy_pack_overrides_default_policy_path() {
        let inputs = test_inputs();
        let mut request = base_request(RunKind::NoLlm, None);
        request.policy_pack_id = "cyrune-free-alt".to_string();

        let context = resolve_turn_context(&request, &inputs).unwrap();

        assert_eq!(context.requested_policy_pack_id, "cyrune-free-alt");
        assert_eq!(context.policy_pack_id, "cyrune-free-alt");
    }

    #[test]
    fn non_canonical_binding_spelling_is_rejected() {
        let inputs = test_inputs();
        let mut request = base_request(RunKind::NoLlm, None);
        request.binding_id = Some("cyrune-free-default.v0.1".to_string());

        let error = resolve_turn_context(&request, &inputs).unwrap_err();

        assert!(matches!(
            error,
            super::ResolverError::Adapter(adapter_resolver::ResolveError::Validation(message))
                if message.contains("binding exact match not found")
        ));
    }

    #[test]
    fn non_canonical_policy_pack_spelling_is_rejected() {
        let inputs = test_inputs();

        let error = resolve_explained_policy("cyrune-free-default.v0.1", &inputs).unwrap_err();

        assert!(matches!(
            error,
            super::ResolverError::Adapter(adapter_resolver::ResolveError::Validation(message))
                if message.contains("policy exact match not found")
        ));
    }

    #[test]
    fn closed_set_violation_is_rejected() {
        let inputs = test_inputs();
        let mut request = base_request(RunKind::NoLlm, None);
        request
            .requested_capabilities
            .push("unsupported".to_string());
        let error = resolve_turn_context(&request, &inputs).unwrap_err();
        assert!(matches!(
            error,
            super::ResolverError::Invalid(message)
                if message.contains("capability outside closed set")
        ));
    }

    #[test]
    fn registry_error_maps_to_contract_error() {
        let error = super::ResolverError::Registry(super::RegistryError::Validation(
            "missing registry".to_string(),
        ));
        let mapped = match error {
            super::ResolverError::Registry(source) => {
                super::super::execution_registry::registry_error_to_contract(source)
            }
            _ => unreachable!(),
        };
        assert!(matches!(
            mapped,
            ContractError::InvalidRequest { message } if message.contains("missing registry")
        ));
    }

    #[test]
    fn shipping_binding_rejects_when_exact_pin_source_is_missing() {
        let inputs = shipping_inputs();
        let error = resolve_turn_context(&base_request(RunKind::NoLlm, None), &inputs).unwrap_err();

        assert!(matches!(
            error,
            super::ResolverError::Adapter(adapter_resolver::ResolveError::Validation(message))
                if message.contains("shipping exact pin authoritative source missing")
        ));
    }

    #[test]
    fn shipping_binding_rejects_invalid_exact_pin_source_hash() {
        let inputs = shipping_inputs();
        write_shipping_exact_pin_source(&inputs.bundle_root, true);

        let error = resolve_turn_context(&base_request(RunKind::NoLlm, None), &inputs).unwrap_err();

        assert!(matches!(
            error,
            super::ResolverError::Adapter(adapter_resolver::ResolveError::Validation(message))
                if message.contains("artifact hash mismatch: model.onnx")
        ));
    }

    #[test]
    fn shipping_binding_resolves_source_driven_non_null_engine_ref() {
        let inputs = shipping_inputs();
        write_shipping_exact_pin_source(&inputs.bundle_root, false);

        let context = resolve_turn_context(&base_request(RunKind::NoLlm, None), &inputs).unwrap();
        let pin = context.embedding_exact_pin.as_ref().unwrap();

        assert_eq!(
            context.resolved_kernel_adapters.embedding_engine_ref,
            shipping_embedding_engine_ref_for_pin(pin)
        );
        assert_ne!(
            context.resolved_kernel_adapters.embedding_engine_ref,
            BRINGUP_EMBEDDING_ENGINE_REF
        );
    }
}
