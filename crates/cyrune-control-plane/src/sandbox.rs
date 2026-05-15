#![forbid(unsafe_code)]

use crate::resolved_turn_context::ResolvedTurnContext;
use cyrune_core_contract::{PathLabel, RunRequest};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    Deny,
    Allow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedSandboxSpec {
    pub cwd: PathBuf,
    pub read_allow_paths: Vec<PathBuf>,
    pub write_allow_paths: Vec<PathBuf>,
    pub env_allowlist: Vec<String>,
    pub env_overrides: BTreeMap<String, String>,
    pub network_mode: NetworkMode,
}

#[derive(Debug, Error)]
pub enum SandboxError {
    #[error("invalid sandbox input: {0}")]
    Invalid(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn normalize_sandbox_spec(
    context: &ResolvedTurnContext,
    request: &RunRequest,
    read_allow_paths: &[PathLabel],
    write_allow_paths: &[PathLabel],
) -> Result<NormalizedSandboxSpec, SandboxError> {
    let cwd_label = request.cwd.as_ref().ok_or_else(|| {
        SandboxError::Invalid("sandbox normalization requires request.cwd".to_string())
    })?;
    let cwd = canonicalize_cwd(cwd_label.as_str())?;

    let mut read_paths = BTreeSet::new();
    for path in read_allow_paths {
        read_paths.insert(canonicalize_allow_path(&cwd, path.as_str())?);
    }

    let mut write_paths = BTreeSet::new();
    for path in write_allow_paths {
        write_paths.insert(canonicalize_allow_path(&cwd, path.as_str())?);
    }

    let mut env_allowlist = context
        .selected_execution_adapter
        .as_ref()
        .map(|selected| selected.env_allowlist.clone())
        .unwrap_or_default();
    env_allowlist.sort();
    env_allowlist.dedup();
    for key in &env_allowlist {
        validate_env_key(key)?;
    }

    let mut env_overrides = BTreeMap::new();
    if let Some(overrides) = &request.env_overrides {
        for (key, value) in overrides {
            validate_env_key(key)?;
            if !env_allowlist.iter().any(|allowed| allowed == key) {
                return Err(SandboxError::Invalid(format!(
                    "env override key is not allowlisted: {key}"
                )));
            }
            env_overrides.insert(key.clone(), value.clone());
        }
    }

    Ok(NormalizedSandboxSpec {
        cwd,
        read_allow_paths: read_paths.into_iter().collect(),
        write_allow_paths: write_paths.into_iter().collect(),
        env_allowlist,
        env_overrides,
        network_mode: if context
            .allowed_capabilities
            .iter()
            .any(|capability| capability == "net")
        {
            NetworkMode::Allow
        } else {
            NetworkMode::Deny
        },
    })
}

pub fn canonicalize_cwd(value: &str) -> Result<PathBuf, SandboxError> {
    reject_shell_expansion(value)?;
    let path = Path::new(value);
    if !path.is_absolute() {
        return Err(SandboxError::Invalid(
            "cwd must be an absolute path".to_string(),
        ));
    }
    Ok(fs::canonicalize(path)?)
}

pub fn canonicalize_allow_path(cwd: &Path, raw: &str) -> Result<PathBuf, SandboxError> {
    reject_shell_expansion(raw)?;
    if raw.is_empty() {
        return Err(SandboxError::Invalid(
            "allow path must not be empty".to_string(),
        ));
    }

    let candidate = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        cwd.join(raw)
    };
    canonicalize_path_with_missing_leaf(&candidate)
}

pub fn is_path_allowed(target: &Path, allowlist: &[PathBuf]) -> Result<bool, SandboxError> {
    let candidate = canonicalize_path_with_missing_leaf(target)?;
    Ok(allowlist
        .iter()
        .any(|entry| candidate == *entry || candidate.starts_with(entry)))
}

fn canonicalize_path_with_missing_leaf(path: &Path) -> Result<PathBuf, SandboxError> {
    let normalized = lexical_normalize(path)?;
    if normalized.exists() {
        return Ok(fs::canonicalize(normalized)?);
    }

    let mut missing = Vec::new();
    let mut current = normalized.as_path();
    while !current.exists() {
        let Some(file_name) = current.file_name() else {
            return Err(SandboxError::Invalid(format!(
                "path cannot be canonicalized: {}",
                normalized.display()
            )));
        };
        missing.push(file_name.to_os_string());
        current = current.parent().ok_or_else(|| {
            SandboxError::Invalid(format!(
                "path cannot be canonicalized: {}",
                normalized.display()
            ))
        })?;
    }

    let mut resolved = fs::canonicalize(current)?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn lexical_normalize(path: &Path) -> Result<PathBuf, SandboxError> {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    return Err(SandboxError::Invalid(format!(
                        "path escapes canonical root: {}",
                        path.display()
                    )));
                }
            }
            Component::Normal(segment) => out.push(segment),
            Component::RootDir => out.push(Path::new("/")),
            Component::Prefix(prefix) => out.push(prefix.as_os_str()),
        }
    }
    Ok(out)
}

fn reject_shell_expansion(value: &str) -> Result<(), SandboxError> {
    if value.contains('~')
        || value.contains("${")
        || value.contains('*')
        || value.contains('?')
        || value.contains('[')
    {
        return Err(SandboxError::Invalid(format!(
            "path contains shell expansion token: {value}"
        )));
    }
    Ok(())
}

fn validate_env_key(key: &str) -> Result<(), SandboxError> {
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
    {
        return Err(SandboxError::Invalid(format!(
            "env key is outside closed set: {key}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{NetworkMode, canonicalize_allow_path, is_path_allowed, normalize_sandbox_spec};
    use crate::resolved_turn_context::{
        ResolvedKernelAdapters, ResolvedTurnContext, SelectedExecutionAdapter, TimeoutPolicy,
    };
    use cyrune_core_contract::{
        CorrelationId, IoMode, PathLabel, RequestId, RunId, RunKind, RunRequest,
    };
    use std::collections::BTreeMap;
    use std::fs;
    use tempfile::tempdir;

    fn execution_context() -> ResolvedTurnContext {
        ResolvedTurnContext {
            version: 1,
            request_id: RequestId::parse("REQ-20260327-0005").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0005").unwrap(),
            run_id: RunId::parse("RUN-20260327-0005-R01").unwrap(),
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
            allowed_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
            sandbox_ref: "SANDBOX_MINIMAL_CANONICAL.md#default-profile".to_string(),
            run_kind: RunKind::ExecutionAdapter,
            io_mode: IoMode::Captured,
            selected_execution_adapter: Some(SelectedExecutionAdapter {
                adapter_id: "local-cli-single-process.v0.1".to_string(),
                adapter_version: "0.1.0".to_string(),
                execution_kind: "process_stdio".to_string(),
                launcher_path: "/bin/sh".to_string(),
                launcher_sha256: "sha256:launcher".to_string(),
                model_id: "model.local".to_string(),
                model_revision_or_digest: "sha256:model".to_string(),
                default_timeout_s: 120,
                allowed_capabilities: vec!["exec".to_string(), "fs_read".to_string()],
                env_allowlist: vec!["SAFE_VAR".to_string()],
            }),
            timeout_policy: TimeoutPolicy {
                turn_timeout_s: 120,
                execution_timeout_s: 120,
            },
        }
    }

    #[test]
    fn path_normalization_handles_relative_and_missing_leaf() {
        let temp = tempdir().unwrap();
        let cwd = temp.path().join("workspace");
        fs::create_dir_all(cwd.join("allowed")).unwrap();
        let normalized = canonicalize_allow_path(&cwd, "./allowed/new-file.txt").unwrap();
        assert!(normalized.ends_with("workspace/allowed/new-file.txt"));
    }

    #[test]
    fn symlink_escape_is_not_allowed_by_membership() {
        let temp = tempdir().unwrap();
        let cwd = temp.path().join("workspace");
        let allowed = cwd.join("allowed");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&allowed).unwrap();
        fs::create_dir_all(&outside).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, allowed.join("escape")).unwrap();

        let allowlist = vec![canonicalize_allow_path(&cwd, "./allowed").unwrap()];
        let target = allowed.join("escape").join("secret.txt");
        assert!(!is_path_allowed(&target, &allowlist).unwrap());
    }

    #[test]
    fn sandbox_spec_rejects_env_outside_allowlist() {
        let temp = tempdir().unwrap();
        let cwd = temp.path().join("workspace");
        fs::create_dir_all(&cwd).unwrap();
        let request = RunRequest {
            request_id: RequestId::parse("REQ-20260327-0005").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0005").unwrap(),
            run_kind: RunKind::ExecutionAdapter,
            user_input: "execute".to_string(),
            policy_pack_id: "cyrune-free-default.v0.1".to_string(),
            binding_id: None,
            requested_capabilities: vec!["exec".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: Some("local-cli-single-process.v0.1".to_string()),
            argv: Some(vec!["/usr/bin/true".to_string()]),
            cwd: Some(PathLabel::parse(cwd.display().to_string()).unwrap()),
            env_overrides: Some(BTreeMap::from([("UNSAFE".to_string(), "1".to_string())])),
        };

        let error = normalize_sandbox_spec(&execution_context(), &request, &[], &[]).unwrap_err();
        assert!(error.to_string().contains("not allowlisted"));
    }

    #[test]
    fn sandbox_spec_uses_deny_network_when_capability_absent() {
        let temp = tempdir().unwrap();
        let cwd = temp.path().join("workspace");
        fs::create_dir_all(&cwd).unwrap();
        let mut context = execution_context();
        context.allowed_capabilities = vec!["exec".to_string()];
        let request = RunRequest {
            request_id: RequestId::parse("REQ-20260327-0005").unwrap(),
            correlation_id: CorrelationId::parse("RUN-20260327-0005").unwrap(),
            run_kind: RunKind::ExecutionAdapter,
            user_input: "execute".to_string(),
            policy_pack_id: "cyrune-free-default.v0.1".to_string(),
            binding_id: None,
            requested_capabilities: vec!["exec".to_string()],
            io_mode: IoMode::Captured,
            adapter_id: Some("local-cli-single-process.v0.1".to_string()),
            argv: Some(vec!["/usr/bin/true".to_string()]),
            cwd: Some(PathLabel::parse(cwd.display().to_string()).unwrap()),
            env_overrides: Some(BTreeMap::from([("SAFE_VAR".to_string(), "1".to_string())])),
        };

        let spec = normalize_sandbox_spec(
            &context,
            &request,
            &[PathLabel::parse("./".to_string()).unwrap()],
            &[PathLabel::parse("./tmp.out".to_string()).unwrap()],
        )
        .unwrap();
        assert_eq!(spec.network_mode, NetworkMode::Deny);
    }
}
