#![forbid(unsafe_code)]

use crate::resolved_turn_context::{SelectedExecutionAdapter, is_allowed_capability};
use cyrune_core_contract::ContractError;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const REGISTRY_VERSION: &str = "cyrune.free.execution-adapter-registry.v1";

#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Validation(String),
}

#[derive(Debug, Deserialize)]
struct RegistryFile {
    registry_version: String,
    entries: Vec<RegistryEntry>,
}

#[derive(Debug, Deserialize)]
struct RegistryEntry {
    adapter_id: String,
    state: String,
    profile_path: String,
}

#[derive(Debug, Deserialize)]
struct RegistryProfile {
    adapter_id: String,
    adapter_version: String,
    execution_kind: String,
    launcher_path: String,
    launcher_sha256: String,
    model_id: String,
    model_revision_or_digest: String,
    allowed_capabilities: Vec<String>,
    default_timeout_s: u64,
    env_allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterializedLauncher {
    pub launcher_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryRootMode {
    CurrentHome,
    PackagedBundle,
}

pub fn resolve_selected_execution_adapter(
    root: &Path,
    mode: RegistryRootMode,
    adapter_id: &str,
) -> Result<SelectedExecutionAdapter, RegistryError> {
    let registry_root = registry_root(root, mode);
    let registry_path = registry_root.join("registry.json");
    let registry: RegistryFile = read_json(&registry_path)?;
    validate_registry(&registry)?;

    let entry = registry
        .entries
        .iter()
        .find(|entry| entry.adapter_id == adapter_id)
        .ok_or_else(|| RegistryError::Validation(format!("adapter_id not found: {adapter_id}")))?;

    if entry.state != "approved" {
        return Err(RegistryError::Validation(format!(
            "adapter state is not approved: {}",
            entry.state
        )));
    }

    let profile_path = registry_root.join(PathBuf::from(&entry.profile_path));
    let profile: RegistryProfile = read_json(&profile_path)?;
    validate_profile(&profile, adapter_id, mode)?;
    if mode == RegistryRootMode::PackagedBundle {
        let launcher_path = resolve_bundle_relative_path(root, &profile.launcher_path)?;
        validate_launcher_sha256(&launcher_path, &profile.launcher_sha256)?;
    }

    Ok(SelectedExecutionAdapter {
        adapter_id: profile.adapter_id,
        adapter_version: profile.adapter_version,
        execution_kind: profile.execution_kind,
        launcher_path: profile.launcher_path,
        launcher_sha256: profile.launcher_sha256,
        model_id: profile.model_id,
        model_revision_or_digest: profile.model_revision_or_digest,
        default_timeout_s: profile.default_timeout_s,
        allowed_capabilities: profile.allowed_capabilities,
        env_allowlist: profile.env_allowlist,
    })
}

pub fn registry_error_to_contract(error: RegistryError) -> ContractError {
    ContractError::InvalidRequest {
        message: format!("execution adapter registry resolution failed: {error}"),
    }
}

pub fn approved_registry_root(bundle_root: &Path) -> PathBuf {
    bundle_root
        .join("registry")
        .join("execution-adapters")
        .join("approved")
}

pub fn resolve_bundle_relative_path(
    bundle_root: &Path,
    raw: &str,
) -> Result<PathBuf, RegistryError> {
    if raw.trim().is_empty() {
        return Err(RegistryError::Validation(
            "launcher_path cannot be empty".to_string(),
        ));
    }
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return Err(RegistryError::Validation(
            "absolute launcher_path is not allowed".to_string(),
        ));
    }
    let bundle_root = fs::canonicalize(bundle_root).map_err(|error| {
        RegistryError::Validation(format!("bundle_root cannot be materialized: {error}"))
    })?;
    let resolved = fs::canonicalize(bundle_root.join(candidate)).map_err(|error| {
        RegistryError::Validation(format!(
            "launcher_path cannot be materialized: {raw} ({error})"
        ))
    })?;
    if !resolved.starts_with(&bundle_root) {
        return Err(RegistryError::Validation(
            "launcher_path escapes bundle_root".to_string(),
        ));
    }
    Ok(resolved)
}

pub fn materialize_launcher(
    bundle_root: &Path,
    selected: &SelectedExecutionAdapter,
) -> Result<MaterializedLauncher, RegistryError> {
    let launcher_path = if Path::new(&selected.launcher_path).is_absolute() {
        fs::canonicalize(PathBuf::from(&selected.launcher_path)).map_err(|error| {
            RegistryError::Validation(format!(
                "launcher_path cannot be materialized: {} ({error})",
                selected.launcher_path
            ))
        })?
    } else {
        resolve_bundle_relative_path(bundle_root, &selected.launcher_path)?
    };
    validate_launcher_sha256(&launcher_path, &selected.launcher_sha256)?;
    Ok(MaterializedLauncher { launcher_path })
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, RegistryError> {
    let text = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&text)?)
}

fn validate_registry(registry: &RegistryFile) -> Result<(), RegistryError> {
    if registry.registry_version != REGISTRY_VERSION {
        return Err(RegistryError::Validation(format!(
            "unknown registry_version: {}",
            registry.registry_version
        )));
    }
    if registry.entries.is_empty() {
        return Err(RegistryError::Validation(
            "registry has no entries".to_string(),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for entry in &registry.entries {
        if !seen.insert(entry.adapter_id.as_str()) {
            return Err(RegistryError::Validation(format!(
                "duplicate adapter_id in registry: {}",
                entry.adapter_id
            )));
        }
        if entry.profile_path.trim().is_empty() {
            return Err(RegistryError::Validation(format!(
                "profile_path missing for adapter_id: {}",
                entry.adapter_id
            )));
        }
    }
    Ok(())
}

fn registry_root(root: &Path, mode: RegistryRootMode) -> PathBuf {
    match mode {
        RegistryRootMode::CurrentHome => root
            .join("registry")
            .join("execution-adapters")
            .join("approved"),
        RegistryRootMode::PackagedBundle => approved_registry_root(root),
    }
}

fn validate_profile(
    profile: &RegistryProfile,
    adapter_id: &str,
    mode: RegistryRootMode,
) -> Result<(), RegistryError> {
    if profile.adapter_id != adapter_id {
        return Err(RegistryError::Validation(format!(
            "profile adapter_id mismatch: expected {adapter_id}, got {}",
            profile.adapter_id
        )));
    }
    if profile.adapter_version.trim().is_empty()
        || profile.execution_kind != "process_stdio"
        || profile.launcher_path.trim().is_empty()
        || profile.launcher_sha256.trim().is_empty()
        || profile.model_id.trim().is_empty()
        || profile.model_revision_or_digest.trim().is_empty()
        || profile.default_timeout_s == 0
    {
        return Err(RegistryError::Validation(format!(
            "profile is missing required fields for adapter_id: {adapter_id}"
        )));
    }
    if mode == RegistryRootMode::PackagedBundle && Path::new(&profile.launcher_path).is_absolute() {
        return Err(RegistryError::Validation(
            "absolute launcher_path is not allowed".to_string(),
        ));
    }
    if profile
        .allowed_capabilities
        .iter()
        .any(|capability| !is_allowed_capability(capability))
    {
        return Err(RegistryError::Validation(format!(
            "profile contains capability outside closed set for adapter_id: {adapter_id}"
        )));
    }
    for key in &profile.env_allowlist {
        if key.is_empty()
            || !key
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        {
            return Err(RegistryError::Validation(format!(
                "profile contains env_allowlist key outside closed set for adapter_id {adapter_id}: {key}"
            )));
        }
    }
    Ok(())
}

fn validate_launcher_sha256(path: &Path, expected: &str) -> Result<(), RegistryError> {
    let bytes = fs::read(path)?;
    let actual = format!("sha256:{}", sha256_hex(&bytes));
    if actual != expected {
        return Err(RegistryError::Validation(format!(
            "launcher_sha256 mismatch: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        RegistryRootMode, materialize_launcher, resolve_selected_execution_adapter, sha256_hex,
    };
    use crate::resolved_turn_context::SelectedExecutionAdapter;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn write_fixture(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn write_launcher(path: &Path) -> String {
        write_fixture(path, "#!/bin/sh\nexit 0\n");
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
        format!("sha256:{}", sha256_hex(&fs::read(path).unwrap()))
    }

    fn write_registry_fixture(bundle_root: &Path, launcher_path: &str, launcher_sha256: &str) {
        let approved_dir = bundle_root
            .join("registry")
            .join("execution-adapters")
            .join("approved");
        write_fixture(
            &approved_dir.join("registry.json"),
            r#"{
  "registry_version": "cyrune.free.execution-adapter-registry.v1",
  "entries": [
    {
      "adapter_id": "local-cli-single-process.v0.1",
      "state": "approved",
      "profile_path": "profiles/local-cli-single-process.v0.1.json"
    }
  ]
}"#,
        );
        write_fixture(
            &approved_dir
                .join("profiles")
                .join("local-cli-single-process.v0.1.json"),
            &format!(
                r#"{{
  "adapter_id": "local-cli-single-process.v0.1",
  "adapter_version": "0.1.0",
  "execution_kind": "process_stdio",
  "launcher_path": "{launcher_path}",
  "launcher_sha256": "{launcher_sha256}",
  "model_id": "model.local",
  "model_revision_or_digest": "sha256:model",
  "allowed_capabilities": ["exec", "fs_read"],
  "default_timeout_s": 120,
  "env_allowlist": []
}}"#
            ),
        );
    }

    #[test]
    fn resolve_selected_execution_adapter_reads_bundle_root_registry() {
        let temp = tempdir().unwrap();
        let bundle_root = temp.path().join("bundle-root");
        let launcher_path = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let launcher_sha256 = write_launcher(&launcher_path);
        write_registry_fixture(
            &bundle_root,
            "runtime/ipc/local-cli-single-process.sh",
            &launcher_sha256,
        );

        let selected = resolve_selected_execution_adapter(
            &bundle_root,
            RegistryRootMode::PackagedBundle,
            "local-cli-single-process.v0.1",
        )
        .unwrap();

        assert_eq!(selected.adapter_id, "local-cli-single-process.v0.1");
        assert_eq!(
            selected.launcher_path,
            "runtime/ipc/local-cli-single-process.sh"
        );
        assert!(!selected.launcher_sha256.is_empty());
    }

    #[test]
    fn resolve_selected_execution_adapter_rejects_absolute_launcher_path() {
        let temp = tempdir().unwrap();
        let bundle_root = temp.path().join("bundle-root");
        let launcher_path = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let launcher_sha256 = write_launcher(&launcher_path);
        write_registry_fixture(
            &bundle_root,
            launcher_path.to_str().unwrap(),
            &launcher_sha256,
        );

        let error = resolve_selected_execution_adapter(
            &bundle_root,
            RegistryRootMode::PackagedBundle,
            "local-cli-single-process.v0.1",
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("absolute launcher_path is not allowed")
        );
    }

    #[test]
    fn resolve_selected_execution_adapter_rejects_launcher_escape_outside_bundle_root() {
        let temp = tempdir().unwrap();
        let bundle_root = temp.path().join("bundle-root");
        let launcher_path = temp.path().join("outside.sh");
        let launcher_sha256 = write_launcher(&launcher_path);
        write_registry_fixture(&bundle_root, "../outside.sh", &launcher_sha256);

        let error = resolve_selected_execution_adapter(
            &bundle_root,
            RegistryRootMode::PackagedBundle,
            "local-cli-single-process.v0.1",
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("launcher_path escapes bundle_root")
        );
    }

    #[test]
    #[cfg(unix)]
    fn launcher_materialization_verifies_digest() {
        let temp = tempdir().unwrap();
        let bundle_root = temp.path().join("bundle-root");
        let launcher_path = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        let launcher_sha256 = write_launcher(&launcher_path);

        let selected = SelectedExecutionAdapter {
            adapter_id: "local-cli-single-process.v0.1".to_string(),
            adapter_version: "0.1.0".to_string(),
            execution_kind: "process_stdio".to_string(),
            launcher_path: "runtime/ipc/local-cli-single-process.sh".to_string(),
            launcher_sha256: launcher_sha256.clone(),
            model_id: "model.local".to_string(),
            model_revision_or_digest: "sha256:model".to_string(),
            default_timeout_s: 120,
            allowed_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            env_allowlist: Vec::new(),
        };

        let materialized = materialize_launcher(&bundle_root, &selected).unwrap();
        assert_eq!(
            materialized.launcher_path,
            fs::canonicalize(&launcher_path).unwrap()
        );

        let invalid = SelectedExecutionAdapter {
            launcher_sha256: "sha256:deadbeef".to_string(),
            ..selected
        };
        let error = materialize_launcher(&bundle_root, &invalid).unwrap_err();
        assert!(error.to_string().contains("launcher_sha256 mismatch"));
    }
}
