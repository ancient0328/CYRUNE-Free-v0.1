use chrono::Utc;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdapterManifest {
    pub adapter_id: String,
    pub version: String,
    pub layers: AdapterLayers,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdapterLayers {
    pub working: LayerCapability,
    pub processing: LayerCapability,
    pub permanent: LayerCapability,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayerCapability {
    pub limits: Limits,
    pub capabilities: Capabilities,
    pub performance_profile: PerformanceProfile,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Limits {
    pub min_items: i64,
    pub max_items: i64,
    pub min_ttl_ms: i64,
    pub max_ttl_ms: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Capabilities {
    pub supports_eviction: bool,
    pub supports_promotion: bool,
    pub supports_demotion: bool,
    pub supports_vector_search: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PerformanceProfile {
    pub read_latency_p95_ms: f64,
    pub write_latency_p95_ms: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DistroPolicyPack {
    pub distro_id: String,
    pub policy_pack_id: String,
    pub version: String,
    pub layers: PolicyLayers,
    pub fail_closed: FailClosed,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PolicyLayers {
    pub working: WorkingPolicy,
    pub processing: ProcessingPolicy,
    pub permanent: PermanentPolicy,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WorkingPolicy {
    pub target_items: i64,
    pub ttl_ms: i64,
    pub eviction_strategy: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProcessingPolicy {
    pub target_items: i64,
    pub ttl_ms: i64,
    pub promotion_threshold: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PermanentPolicy {
    pub retention_mode: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FailClosed {
    pub on_capacity_out_of_range: bool,
    pub on_ttl_out_of_range: bool,
    pub on_missing_capability: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DistroAdapterBinding {
    pub distro_id: String,
    pub binding_id: String,
    pub version: String,
    pub resolution_mode: String,
    pub layers: BindingLayers,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BindingLayers {
    pub working: LayerBinding,
    pub processing: LayerBinding,
    pub permanent: LayerBinding,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LayerBinding {
    pub adapter_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct LayerAggregate {
    limits: Limits,
    capabilities: Capabilities,
    adapter_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolvedOutput {
    pub distro_id: String,
    pub policy_pack_id: String,
    pub binding_id: String,
    pub resolved_at: String,
    pub effective: EffectiveOutput,
    pub decision_log: Vec<DecisionLogEntry>,
}

#[derive(Debug, Serialize)]
pub struct EffectiveOutput {
    pub layers: EffectiveLayers,
}

#[derive(Debug, Serialize)]
pub struct EffectiveLayers {
    pub working: EffectiveLayer,
    pub processing: EffectiveLayer,
    pub permanent: EffectiveLayer,
}

#[derive(Debug, Serialize, Clone)]
pub struct EffectiveLayer {
    pub adapter_ids: Vec<String>,
    pub limits: Limits,
    pub capabilities: Capabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_items: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eviction_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promotion_threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention_mode: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DecisionLogEntry {
    #[serde(rename = "type")]
    pub entry_type: String,
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_value: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
}

fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, ResolveError> {
    let text = fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<T>(&text)?;
    Ok(parsed)
}

fn is_valid_version(version: &str) -> bool {
    let mut chars = version.chars();
    if chars.next() != Some('v') {
        return false;
    }
    let rest: String = chars.collect();
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() != 2 {
        return false;
    }
    parts
        .iter()
        .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_distro_agnostic_adapter_id(id: &str) -> bool {
    if id.is_empty() || id.starts_with("cyrune-") || id.starts_with("forge-") {
        return false;
    }
    let mut prev_dash = false;
    for ch in id.chars() {
        let ok = ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-';
        if !ok {
            return false;
        }
        if ch == '-' {
            if prev_dash {
                return false;
            }
            prev_dash = true;
        } else {
            prev_dash = false;
        }
    }
    !id.starts_with('-') && !id.ends_with('-')
}

fn validate_limits(limits: &Limits, path: &str) -> Result<(), ResolveError> {
    if limits.min_items < 0
        || limits.max_items < 0
        || limits.min_ttl_ms < 0
        || limits.max_ttl_ms < 0
    {
        return Err(ResolveError::Validation(format!(
            "limits must be non-negative: {path}"
        )));
    }
    if limits.min_items > limits.max_items {
        return Err(ResolveError::Validation(format!(
            "min_items > max_items: {path}"
        )));
    }
    if limits.min_ttl_ms > limits.max_ttl_ms {
        return Err(ResolveError::Validation(format!(
            "min_ttl_ms > max_ttl_ms: {path}"
        )));
    }
    Ok(())
}

pub fn validate_manifest(manifest: &AdapterManifest, source: &str) -> Result<(), ResolveError> {
    if !is_distro_agnostic_adapter_id(&manifest.adapter_id) {
        return Err(ResolveError::Validation(format!(
            "adapter_id must be distro-agnostic: {} ({source})",
            manifest.adapter_id
        )));
    }
    if !is_valid_version(&manifest.version) {
        return Err(ResolveError::Validation(format!(
            "invalid version in manifest {source}: {}",
            manifest.version
        )));
    }

    validate_limits(
        &manifest.layers.working.limits,
        &format!("{source}.layers.working"),
    )?;
    validate_limits(
        &manifest.layers.processing.limits,
        &format!("{source}.layers.processing"),
    )?;
    validate_limits(
        &manifest.layers.permanent.limits,
        &format!("{source}.layers.permanent"),
    )?;

    for (name, perf) in [
        ("working", &manifest.layers.working.performance_profile),
        (
            "processing",
            &manifest.layers.processing.performance_profile,
        ),
        ("permanent", &manifest.layers.permanent.performance_profile),
    ] {
        if perf.read_latency_p95_ms < 0.0 || perf.write_latency_p95_ms < 0.0 {
            return Err(ResolveError::Validation(format!(
                "performance values must be non-negative: {source}.layers.{name}"
            )));
        }
    }

    Ok(())
}

pub fn validate_policy(policy: &DistroPolicyPack, source: &str) -> Result<(), ResolveError> {
    if !is_valid_version(&policy.version) {
        return Err(ResolveError::Validation(format!(
            "invalid policy version {source}: {}",
            policy.version
        )));
    }
    if policy.layers.working.target_items < 0
        || policy.layers.working.ttl_ms < 0
        || policy.layers.processing.target_items < 0
        || policy.layers.processing.ttl_ms < 0
    {
        return Err(ResolveError::Validation(format!(
            "policy values must be non-negative: {source}"
        )));
    }
    if !matches!(
        policy.layers.working.eviction_strategy.as_str(),
        "lru" | "fifo" | "priority" | "manual"
    ) {
        return Err(ResolveError::Validation(format!(
            "invalid working.eviction_strategy: {}",
            policy.layers.working.eviction_strategy
        )));
    }
    if !(0.0..=1.0).contains(&policy.layers.processing.promotion_threshold) {
        return Err(ResolveError::Validation(format!(
            "invalid processing.promotion_threshold: {}",
            policy.layers.processing.promotion_threshold
        )));
    }
    if !matches!(
        policy.layers.permanent.retention_mode.as_str(),
        "immutable" | "versioned" | "governed"
    ) {
        return Err(ResolveError::Validation(format!(
            "invalid permanent.retention_mode: {}",
            policy.layers.permanent.retention_mode
        )));
    }
    Ok(())
}

pub fn validate_binding(binding: &DistroAdapterBinding, source: &str) -> Result<(), ResolveError> {
    if !is_valid_version(&binding.version) {
        return Err(ResolveError::Validation(format!(
            "invalid binding version {source}: {}",
            binding.version
        )));
    }
    if !matches!(binding.resolution_mode.as_str(), "single" | "chain") {
        return Err(ResolveError::Validation(format!(
            "invalid resolution_mode in {source}: {}",
            binding.resolution_mode
        )));
    }
    for (layer_name, ids) in [
        ("working", &binding.layers.working.adapter_ids),
        ("processing", &binding.layers.processing.adapter_ids),
        ("permanent", &binding.layers.permanent.adapter_ids),
    ] {
        if ids.is_empty() {
            return Err(ResolveError::Validation(format!(
                "binding layer has empty adapter_ids: {source}.layers.{layer_name}"
            )));
        }
        for id in ids {
            if !is_distro_agnostic_adapter_id(id) {
                return Err(ResolveError::Validation(format!(
                    "binding adapter_id must be distro-agnostic: {id}"
                )));
            }
        }
    }
    Ok(())
}

pub fn load_catalog(catalog_dir: &Path) -> Result<BTreeMap<String, AdapterManifest>, ResolveError> {
    if !catalog_dir.exists() {
        return Err(ResolveError::Validation(format!(
            "catalog directory not found: {}",
            catalog_dir.display()
        )));
    }
    let mut paths: Vec<PathBuf> = fs::read_dir(catalog_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    paths.sort();

    let mut catalog = BTreeMap::new();
    for path in paths {
        let manifest: AdapterManifest = read_json_file(&path)?;
        validate_manifest(&manifest, &path.display().to_string())?;
        if catalog.contains_key(&manifest.adapter_id) {
            return Err(ResolveError::Validation(format!(
                "duplicate adapter_id in catalog: {}",
                manifest.adapter_id
            )));
        }
        catalog.insert(manifest.adapter_id.clone(), manifest);
    }
    if catalog.is_empty() {
        return Err(ResolveError::Validation("catalog is empty".to_string()));
    }
    Ok(catalog)
}

pub fn load_policy(path: &Path) -> Result<DistroPolicyPack, ResolveError> {
    let policy: DistroPolicyPack = read_json_file(path)?;
    validate_policy(&policy, &path.display().to_string())?;
    Ok(policy)
}

pub fn load_binding(path: &Path) -> Result<DistroAdapterBinding, ResolveError> {
    let binding: DistroAdapterBinding = read_json_file(path)?;
    validate_binding(&binding, &path.display().to_string())?;
    Ok(binding)
}

fn layer_capability<'a>(manifest: &'a AdapterManifest, layer: &str) -> &'a LayerCapability {
    match layer {
        "working" => &manifest.layers.working,
        "processing" => &manifest.layers.processing,
        "permanent" => &manifest.layers.permanent,
        _ => unreachable!("unsupported layer: {layer}"),
    }
}

fn clamp(value: i64, min_v: i64, max_v: i64) -> i64 {
    std::cmp::max(min_v, std::cmp::min(value, max_v))
}

fn aggregate_layer(
    catalog: &BTreeMap<String, AdapterManifest>,
    layer: &str,
    adapter_ids: &[String],
    resolution_mode: &str,
) -> Result<LayerAggregate, ResolveError> {
    let selected: Vec<String> = if resolution_mode == "single" {
        vec![adapter_ids[0].clone()]
    } else {
        adapter_ids.to_vec()
    };

    let mut manifests: Vec<&AdapterManifest> = Vec::with_capacity(selected.len());
    for adapter_id in &selected {
        let manifest = catalog.get(adapter_id).ok_or_else(|| {
            ResolveError::Validation(format!(
                "binding references unknown adapter_id: {adapter_id}"
            ))
        })?;
        manifests.push(manifest);
    }

    let mut min_items = i64::MIN;
    let mut max_items = i64::MAX;
    let mut min_ttl = i64::MIN;
    let mut max_ttl = i64::MAX;
    let mut supports_eviction = true;
    let mut supports_promotion = true;
    let mut supports_demotion = true;
    let mut supports_vector_search = true;

    for manifest in manifests {
        let layer_cap = layer_capability(manifest, layer);
        min_items = std::cmp::max(min_items, layer_cap.limits.min_items);
        max_items = std::cmp::min(max_items, layer_cap.limits.max_items);
        min_ttl = std::cmp::max(min_ttl, layer_cap.limits.min_ttl_ms);
        max_ttl = std::cmp::min(max_ttl, layer_cap.limits.max_ttl_ms);

        supports_eviction &= layer_cap.capabilities.supports_eviction;
        supports_promotion &= layer_cap.capabilities.supports_promotion;
        supports_demotion &= layer_cap.capabilities.supports_demotion;
        supports_vector_search &= layer_cap.capabilities.supports_vector_search;
    }

    if min_items > max_items || min_ttl > max_ttl {
        return Err(ResolveError::Validation(format!(
            "incompatible adapter chain limits at layer={layer}"
        )));
    }

    Ok(LayerAggregate {
        limits: Limits {
            min_items,
            max_items,
            min_ttl_ms: min_ttl,
            max_ttl_ms: max_ttl,
        },
        capabilities: Capabilities {
            supports_eviction,
            supports_promotion,
            supports_demotion,
            supports_vector_search,
        },
        adapter_ids: selected,
    })
}

fn required_capabilities(layer: &str, policy: &DistroPolicyPack) -> Vec<&'static str> {
    let mut required = Vec::new();
    if layer == "working" && policy.layers.working.eviction_strategy != "manual" {
        required.push("supports_eviction");
    }
    if layer == "processing" {
        required.push("supports_promotion");
    }
    required
}

fn capability_value(caps: &Capabilities, key: &str) -> bool {
    match key {
        "supports_eviction" => caps.supports_eviction,
        "supports_promotion" => caps.supports_promotion,
        "supports_demotion" => caps.supports_demotion,
        "supports_vector_search" => caps.supports_vector_search,
        _ => false,
    }
}

fn push_clamp_log(
    log: &mut Vec<DecisionLogEntry>,
    kind: &str,
    layer: &str,
    policy: i64,
    effective: i64,
    min: i64,
    max: i64,
) {
    log.push(DecisionLogEntry {
        entry_type: kind.to_string(),
        layer: layer.to_string(),
        policy_value: Some(policy),
        effective_value: Some(effective),
        min: Some(min),
        max: Some(max),
        capability: None,
        action: None,
    });
}

fn push_missing_capability_log(log: &mut Vec<DecisionLogEntry>, layer: &str, capability: &str) {
    log.push(DecisionLogEntry {
        entry_type: "missing_capability".to_string(),
        layer: layer.to_string(),
        policy_value: None,
        effective_value: None,
        min: None,
        max: None,
        capability: Some(capability.to_string()),
        action: Some("accepted_by_policy".to_string()),
    });
}

pub fn resolve_configuration(
    catalog: &BTreeMap<String, AdapterManifest>,
    policy: &DistroPolicyPack,
    binding: &DistroAdapterBinding,
) -> Result<ResolvedOutput, ResolveError> {
    if policy.distro_id != binding.distro_id {
        return Err(ResolveError::Validation(
            "policy.distro_id and binding.distro_id must match".to_string(),
        ));
    }

    let mut decision_log = Vec::new();
    let mut resolved_working = EffectiveLayer {
        adapter_ids: Vec::new(),
        limits: Limits {
            min_items: 0,
            max_items: 0,
            min_ttl_ms: 0,
            max_ttl_ms: 0,
        },
        capabilities: Capabilities {
            supports_eviction: false,
            supports_promotion: false,
            supports_demotion: false,
            supports_vector_search: false,
        },
        target_items: None,
        ttl_ms: None,
        eviction_strategy: None,
        promotion_threshold: None,
        retention_mode: None,
    };
    let mut resolved_processing = resolved_working.clone();
    let mut resolved_permanent = resolved_working.clone();

    for (layer_name, adapter_ids) in [
        ("working", &binding.layers.working.adapter_ids),
        ("processing", &binding.layers.processing.adapter_ids),
        ("permanent", &binding.layers.permanent.adapter_ids),
    ] {
        let agg = aggregate_layer(catalog, layer_name, adapter_ids, &binding.resolution_mode)?;
        for cap in required_capabilities(layer_name, policy) {
            if !capability_value(&agg.capabilities, cap) {
                if policy.fail_closed.on_missing_capability {
                    return Err(ResolveError::Validation(format!(
                        "missing capability at layer={layer_name}: {cap}"
                    )));
                }
                push_missing_capability_log(&mut decision_log, layer_name, cap);
            }
        }

        let mut layer = EffectiveLayer {
            adapter_ids: agg.adapter_ids,
            limits: agg.limits.clone(),
            capabilities: agg.capabilities.clone(),
            target_items: None,
            ttl_ms: None,
            eviction_strategy: None,
            promotion_threshold: None,
            retention_mode: None,
        };

        match layer_name {
            "working" => {
                let policy_items = policy.layers.working.target_items;
                let effective_items =
                    clamp(policy_items, agg.limits.min_items, agg.limits.max_items);
                if effective_items != policy_items {
                    push_clamp_log(
                        &mut decision_log,
                        "clamp_capacity",
                        layer_name,
                        policy_items,
                        effective_items,
                        agg.limits.min_items,
                        agg.limits.max_items,
                    );
                    if policy.fail_closed.on_capacity_out_of_range {
                        return Err(ResolveError::Validation(format!(
                            "capacity out of range at layer={layer_name}: {policy_items}"
                        )));
                    }
                }
                let policy_ttl = policy.layers.working.ttl_ms;
                let effective_ttl = clamp(policy_ttl, agg.limits.min_ttl_ms, agg.limits.max_ttl_ms);
                if effective_ttl != policy_ttl {
                    push_clamp_log(
                        &mut decision_log,
                        "clamp_ttl",
                        layer_name,
                        policy_ttl,
                        effective_ttl,
                        agg.limits.min_ttl_ms,
                        agg.limits.max_ttl_ms,
                    );
                    if policy.fail_closed.on_ttl_out_of_range {
                        return Err(ResolveError::Validation(format!(
                            "ttl out of range at layer={layer_name}: {policy_ttl}"
                        )));
                    }
                }
                layer.target_items = Some(effective_items);
                layer.ttl_ms = Some(effective_ttl);
                layer.eviction_strategy = Some(policy.layers.working.eviction_strategy.clone());
                resolved_working = layer;
            }
            "processing" => {
                let policy_items = policy.layers.processing.target_items;
                let effective_items =
                    clamp(policy_items, agg.limits.min_items, agg.limits.max_items);
                if effective_items != policy_items {
                    push_clamp_log(
                        &mut decision_log,
                        "clamp_capacity",
                        layer_name,
                        policy_items,
                        effective_items,
                        agg.limits.min_items,
                        agg.limits.max_items,
                    );
                    if policy.fail_closed.on_capacity_out_of_range {
                        return Err(ResolveError::Validation(format!(
                            "capacity out of range at layer={layer_name}: {policy_items}"
                        )));
                    }
                }
                let policy_ttl = policy.layers.processing.ttl_ms;
                let effective_ttl = clamp(policy_ttl, agg.limits.min_ttl_ms, agg.limits.max_ttl_ms);
                if effective_ttl != policy_ttl {
                    push_clamp_log(
                        &mut decision_log,
                        "clamp_ttl",
                        layer_name,
                        policy_ttl,
                        effective_ttl,
                        agg.limits.min_ttl_ms,
                        agg.limits.max_ttl_ms,
                    );
                    if policy.fail_closed.on_ttl_out_of_range {
                        return Err(ResolveError::Validation(format!(
                            "ttl out of range at layer={layer_name}: {policy_ttl}"
                        )));
                    }
                }
                layer.target_items = Some(effective_items);
                layer.ttl_ms = Some(effective_ttl);
                layer.promotion_threshold = Some(policy.layers.processing.promotion_threshold);
                resolved_processing = layer;
            }
            "permanent" => {
                layer.retention_mode = Some(policy.layers.permanent.retention_mode.clone());
                resolved_permanent = layer;
            }
            _ => unreachable!("unsupported layer"),
        }
    }

    Ok(ResolvedOutput {
        distro_id: policy.distro_id.clone(),
        policy_pack_id: policy.policy_pack_id.clone(),
        binding_id: binding.binding_id.clone(),
        resolved_at: Utc::now().to_rfc3339(),
        effective: EffectiveOutput {
            layers: EffectiveLayers {
                working: resolved_working,
                processing: resolved_processing,
                permanent: resolved_permanent,
            },
        },
        decision_log,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use tempfile::TempDir;

    fn fixture_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn resolve_success() {
        let root = fixture_root();
        let catalog = load_catalog(&root.join("catalog")).unwrap();
        let policy = load_policy(&root.join("policies/cyrune-free-default.v0.1.json")).unwrap();
        let binding = load_binding(&root.join("bindings/cyrune-free-default.v0.1.json")).unwrap();
        let out = resolve_configuration(&catalog, &policy, &binding).unwrap();
        assert_eq!(out.distro_id, "cyrune-free");
        assert_eq!(out.effective.layers.working.target_items, Some(10));
        assert_eq!(out.effective.layers.processing.target_items, Some(20_000));
        assert!(out.decision_log.is_empty());
    }

    #[test]
    fn out_of_range_capacity_fails_closed() {
        let root = fixture_root();
        let catalog = load_catalog(&root.join("catalog")).unwrap();
        let mut policy = load_policy(&root.join("policies/cyrune-free-default.v0.1.json")).unwrap();
        let binding = load_binding(&root.join("bindings/cyrune-free-default.v0.1.json")).unwrap();
        policy.layers.working.target_items = 9_999;
        let e = resolve_configuration(&catalog, &policy, &binding).unwrap_err();
        assert!(format!("{e}").contains("capacity out of range"));
    }

    #[test]
    fn missing_capability_fails_closed() {
        let root = fixture_root();
        let policy = load_policy(&root.join("policies/cyrune-free-default.v0.1.json")).unwrap();
        let binding = load_binding(&root.join("bindings/cyrune-free-default.v0.1.json")).unwrap();

        let tmp = TempDir::new().unwrap();
        let mut manifest: Value =
            read_json_file(&root.join("catalog/memory-kv-inmem.v0.1.json")).unwrap();
        manifest["layers"]["processing"]["capabilities"]["supports_promotion"] = Value::Bool(false);
        fs::write(
            tmp.path().join("memory-kv-inmem.v0.1.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let catalog = load_catalog(tmp.path()).unwrap();
        let e = resolve_configuration(&catalog, &policy, &binding).unwrap_err();
        assert!(format!("{e}").contains("missing capability"));
    }
}
