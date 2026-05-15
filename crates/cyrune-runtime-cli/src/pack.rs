#![forbid(unsafe_code)]

use cyrune_daemon::command::{CommandError, HealthReport};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedLaunchPlan {
    pub cyrune_home: PathBuf,
    pub distribution_root: PathBuf,
    pub bundle_root: PathBuf,
    pub terminal_config_path: PathBuf,
    pub runtime_pid_path: PathBuf,
    pub distribution_root_override: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedProductizationIdentity {
    pub product_line_label: String,
    pub packaged_product_display_name: String,
    pub app_bundle_basename: String,
    pub terminal_bundle_executable_stem: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedNoticeBundle {
    pub license_bundle_path: String,
    pub sbom_path: String,
    pub third_party_notice_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedIntegrityEvidence {
    pub integrity_mode: String,
    pub signature_mode: String,
    pub update_policy: String,
    pub hash_list_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedInheritanceSnapshot {
    pub runtime_entry: String,
    pub daemon_entry: String,
    pub bundle_root_path: String,
    pub home_template_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedUpstreamIntakeJudgment {
    pub upstream_intake_mode: String,
    pub upstream_follow_triggers: Vec<String>,
    pub upstream_auto_follow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedProductizationSnapshot {
    pub identity: PackagedProductizationIdentity,
    pub notice_bundle: PackagedNoticeBundle,
    pub integrity_evidence: PackagedIntegrityEvidence,
    pub inheritance_snapshot: PackagedInheritanceSnapshot,
    pub upstream_intake_judgment: PackagedUpstreamIntakeJudgment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedReleasePreparationArtifact {
    pub artifact_class: String,
    pub platform: String,
    pub emitted_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedReleasePreparationUpstreamSourcePin {
    pub source_project: String,
    pub source_kind: String,
    pub exact_revision: String,
    pub source_archive: String,
    pub evidence_origin: String,
    pub source_reference_url: String,
    pub upstream_intake_mode: String,
    pub upstream_follow_triggers: Vec<String>,
    pub upstream_auto_follow: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedReleasePreparationSnapshot {
    pub reverse_dns_bundle_identifier: String,
    pub installer_artifact: PackagedReleasePreparationArtifact,
    pub archive_artifact: PackagedReleasePreparationArtifact,
    pub upstream_source_pin: PackagedReleasePreparationUpstreamSourcePin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedOrganizationOwnedReleasePreparationSnapshot {
    pub release_preparation: PackagedReleasePreparationSnapshot,
    pub signing_identity: String,
    pub notarization_provider: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLaunchInvocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagedLaunchFailureSurface {
    PreflightFailure,
    LauncherFailure,
}

impl PackagedLaunchFailureSurface {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreflightFailure => "preflight_failure",
            Self::LauncherFailure => "launcher_failure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedLaunchFailure {
    pub surface: PackagedLaunchFailureSurface,
    pub reason: &'static str,
    pub message: &'static str,
}

impl PackagedLaunchFailure {
    fn preflight(reason: &'static str, message: &'static str) -> Self {
        Self {
            surface: PackagedLaunchFailureSurface::PreflightFailure,
            reason,
            message,
        }
    }

    fn launcher(reason: &'static str, message: &'static str) -> Self {
        Self {
            surface: PackagedLaunchFailureSurface::LauncherFailure,
            reason,
            message,
        }
    }
}

impl fmt::Display for PackagedLaunchFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagedProductizationFailureSurface {
    ProductizationFailure,
}

impl PackagedProductizationFailureSurface {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ProductizationFailure => "productization_failure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedProductizationFailure {
    pub surface: PackagedProductizationFailureSurface,
    pub reason: &'static str,
    pub message: &'static str,
}

impl PackagedProductizationFailure {
    fn productization(reason: &'static str, message: &'static str) -> Self {
        Self {
            surface: PackagedProductizationFailureSurface::ProductizationFailure,
            reason,
            message,
        }
    }
}

impl fmt::Display for PackagedProductizationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagedReleasePreparationFailureSurface {
    ReleasePreparationFailure,
}

impl PackagedReleasePreparationFailureSurface {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReleasePreparationFailure => "release_preparation_failure",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedReleasePreparationFailure {
    pub surface: PackagedReleasePreparationFailureSurface,
    pub reason: &'static str,
    pub message: &'static str,
}

impl PackagedReleasePreparationFailure {
    fn release_preparation(reason: &'static str, message: &'static str) -> Self {
        Self {
            surface: PackagedReleasePreparationFailureSurface::ReleasePreparationFailure,
            reason,
            message,
        }
    }
}

impl fmt::Display for PackagedReleasePreparationFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagedLaunchExecution {
    pub plan: PackagedLaunchPlan,
    pub invocation: TerminalLaunchInvocation,
    pub exit_code: i32,
}

pub fn default_cyrune_home() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("CYRUNE_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(home) = env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".cyrune"));
    }
    Err("CYRUNE_HOME and HOME are both unavailable".to_string())
}

pub fn ensure_terminal_config(cyrune_home: &Path) -> Result<PathBuf, String> {
    cyrune_daemon::command::ensure_home_layout(cyrune_home).map_err(|error| error.to_string())?;
    cyrune_daemon::command::ensure_default_terminal_config(cyrune_home)
        .map_err(|error| error.to_string())?;
    Ok(cyrune_home
        .join("terminal")
        .join("config")
        .join("wezterm.lua"))
}

pub fn default_daemon_binary_path() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("CYRUNE_DAEMON_BIN") {
        return Ok(PathBuf::from(path));
    }
    let current = env::current_exe().map_err(|error| error.to_string())?;
    let file_name = current
        .file_name()
        .ok_or_else(|| "current executable file name is missing".to_string())?;
    let mut daemon_name = if file_name.to_string_lossy().contains(".exe") {
        "cyrune-daemon.exe".to_string()
    } else {
        "cyrune-daemon".to_string()
    };
    if let Some(extension) = current.extension().and_then(|ext| ext.to_str()) {
        if extension != "exe" && current.file_stem().is_some() {
            daemon_name = "cyrune-daemon".to_string();
        }
    }
    Ok(current.with_file_name(daemon_name))
}

pub fn write_text_if_missing(path: &Path, text: &str) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, text).map_err(|error| error.to_string())
}

pub fn read_packaged_productization_identity(
    distribution_root: &Path,
) -> Result<Option<PackagedProductizationIdentity>, String> {
    let manifest = read_release_manifest(distribution_root)?;
    parse_packaged_productization_identity(&manifest)
}

pub fn read_packaged_notice_bundle(
    distribution_root: &Path,
) -> Result<PackagedNoticeBundle, String> {
    let manifest = read_release_manifest(distribution_root)?;
    parse_packaged_notice_bundle(distribution_root, &manifest)
}

pub fn read_packaged_integrity_evidence(
    distribution_root: &Path,
) -> Result<PackagedIntegrityEvidence, String> {
    let manifest = read_release_manifest(distribution_root)?;
    parse_packaged_integrity_evidence(distribution_root, &manifest)
}

pub fn read_packaged_inheritance_snapshot(
    distribution_root: &Path,
) -> Result<PackagedInheritanceSnapshot, String> {
    let manifest = read_release_manifest(distribution_root)?;
    parse_packaged_inheritance_snapshot(distribution_root, &manifest)
}

pub fn read_packaged_upstream_intake_judgment(
    distribution_root: &Path,
) -> Result<PackagedUpstreamIntakeJudgment, String> {
    let manifest = read_release_manifest(distribution_root)?;
    parse_packaged_upstream_intake_judgment(&manifest)
}

pub fn validate_packaged_productization(
    distribution_root: &Path,
) -> Result<PackagedProductizationSnapshot, PackagedProductizationFailure> {
    let manifest = read_release_manifest(distribution_root).map_err(|_| {
        PackagedProductizationFailure::productization(
            "productization_metadata_invalid",
            "packaged productization metadata is invalid",
        )
    })?;
    let identity = parse_packaged_productization_identity(&manifest)
        .map_err(|_| {
            PackagedProductizationFailure::productization(
                "productization_metadata_invalid",
                "packaged productization metadata is invalid",
            )
        })?
        .ok_or_else(|| {
            PackagedProductizationFailure::productization(
                "productization_metadata_invalid",
                "packaged productization metadata is invalid",
            )
        })?;
    let notice_bundle =
        parse_packaged_notice_bundle(distribution_root, &manifest).map_err(|_| {
            PackagedProductizationFailure::productization(
                "notice_bundle_invalid",
                "packaged notice bundle is invalid",
            )
        })?;
    let integrity_evidence = parse_packaged_integrity_evidence(distribution_root, &manifest)
        .map_err(|_| {
            PackagedProductizationFailure::productization(
                "integrity_evidence_invalid",
                "packaged integrity evidence is invalid",
            )
        })?;
    let inheritance_snapshot = parse_packaged_inheritance_snapshot(distribution_root, &manifest)
        .map_err(|_| {
            PackagedProductizationFailure::productization(
                "inheritance_snapshot_invalid",
                "packaged inheritance snapshot is invalid",
            )
        })?;
    let upstream_intake_judgment =
        parse_packaged_upstream_intake_judgment(&manifest).map_err(|_| {
            PackagedProductizationFailure::productization(
                "upstream_intake_judgment_invalid",
                "packaged upstream intake judgment is invalid",
            )
        })?;

    Ok(PackagedProductizationSnapshot {
        identity,
        notice_bundle,
        integrity_evidence,
        inheritance_snapshot,
        upstream_intake_judgment,
    })
}

pub fn validate_packaged_release_preparation(
    distribution_root: &Path,
) -> Result<PackagedReleasePreparationSnapshot, PackagedReleasePreparationFailure> {
    let manifest = read_release_manifest(distribution_root).map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "release_preparation_metadata_invalid",
            "packaged release preparation metadata is invalid",
        )
    })?;
    let identity = parse_packaged_productization_identity(&manifest)
        .map_err(|_| {
            PackagedReleasePreparationFailure::release_preparation(
                "release_preparation_metadata_invalid",
                "packaged release preparation metadata is invalid",
            )
        })?
        .ok_or_else(|| {
            PackagedReleasePreparationFailure::release_preparation(
                "release_preparation_metadata_invalid",
                "packaged release preparation metadata is invalid",
            )
        })?;
    let notice_bundle =
        parse_packaged_notice_bundle(distribution_root, &manifest).map_err(|_| {
            PackagedReleasePreparationFailure::release_preparation(
                "release_preparation_metadata_invalid",
                "packaged release preparation metadata is invalid",
            )
        })?;
    let upstream_intake_judgment =
        parse_packaged_upstream_intake_judgment(&manifest).map_err(|_| {
            PackagedReleasePreparationFailure::release_preparation(
                "upstream_source_pin_invalid",
                "packaged upstream source pin is invalid",
            )
        })?;
    let manifest_object = manifest.as_object().ok_or_else(|| {
        PackagedReleasePreparationFailure::release_preparation(
            "release_preparation_metadata_invalid",
            "packaged release preparation metadata is invalid",
        )
    })?;
    let primary_os =
        required_manifest_member_string(manifest_object, "RELEASE_MANIFEST.json", "primary_os")
            .map_err(|_| {
                PackagedReleasePreparationFailure::release_preparation(
                    "artifact_naming_invalid",
                    "packaged release artifact naming is invalid",
                )
            })?;
    let distribution_unit = required_manifest_member_string(
        manifest_object,
        "RELEASE_MANIFEST.json",
        "distribution_unit",
    )
    .map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "artifact_naming_invalid",
            "packaged release artifact naming is invalid",
        )
    })?;
    let metadata = read_release_preparation_metadata(distribution_root).map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "release_preparation_metadata_invalid",
            "packaged release preparation metadata is invalid",
        )
    })?;

    let reverse_dns_bundle_identifier = parse_release_preparation_bundle_identifier(
        distribution_root,
        &metadata,
        &notice_bundle,
        &identity,
    )
    .map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "bundle_identifier_invalid",
            "packaged reverse-DNS bundle identifier is invalid",
        )
    })?;
    let installer_artifact = parse_release_preparation_artifact(
        &metadata,
        "installer_artifact",
        "app_bundle",
        &primary_os,
        &identity.app_bundle_basename,
    )
    .map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "artifact_naming_invalid",
            "packaged release artifact naming is invalid",
        )
    })?;
    let archive_artifact = parse_release_preparation_artifact(
        &metadata,
        "archive_artifact",
        "distribution_archive",
        &primary_os,
        &distribution_unit,
    )
    .map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "artifact_naming_invalid",
            "packaged release artifact naming is invalid",
        )
    })?;
    let upstream_source_pin =
        parse_release_preparation_upstream_source_pin(&metadata, &upstream_intake_judgment)
            .map_err(|_| {
                PackagedReleasePreparationFailure::release_preparation(
                    "upstream_source_pin_invalid",
                    "packaged upstream source pin is invalid",
                )
            })?;

    Ok(PackagedReleasePreparationSnapshot {
        reverse_dns_bundle_identifier,
        installer_artifact,
        archive_artifact,
        upstream_source_pin,
    })
}

pub fn validate_packaged_release_preparation_org_owned(
    distribution_root: &Path,
) -> Result<PackagedOrganizationOwnedReleasePreparationSnapshot, PackagedReleasePreparationFailure>
{
    let metadata = read_release_preparation_metadata(distribution_root).map_err(|_| {
        PackagedReleasePreparationFailure::release_preparation(
            "release_preparation_metadata_invalid",
            "packaged release preparation metadata is invalid",
        )
    })?;
    metadata.as_object().ok_or_else(|| {
        PackagedReleasePreparationFailure::release_preparation(
            "release_preparation_metadata_invalid",
            "packaged release preparation metadata is invalid",
        )
    })?;
    let release_preparation = validate_packaged_release_preparation(distribution_root)?;

    let signing_identity =
        parse_release_preparation_organization_owned_value(&metadata, "signing_identity").map_err(
            |_| {
                PackagedReleasePreparationFailure::release_preparation(
                    "signing_identity_invalid",
                    "packaged signing identity is invalid",
                )
            },
        )?;
    let notarization_provider =
        parse_release_preparation_organization_owned_value(&metadata, "notarization_provider")
            .map_err(|_| {
                PackagedReleasePreparationFailure::release_preparation(
                    "notarization_provider_invalid",
                    "packaged notarization provider is invalid",
                )
            })?;

    Ok(PackagedOrganizationOwnedReleasePreparationSnapshot {
        release_preparation,
        signing_identity,
        notarization_provider,
    })
}

pub fn prepare_packaged_launch(
    cyrune_home: &Path,
) -> Result<PackagedLaunchPlan, PackagedLaunchFailure> {
    prepare_packaged_launch_with_distribution_root_override(cyrune_home, None)
}

pub fn build_terminal_launch_invocation(
    terminal_binary: &Path,
    plan: &PackagedLaunchPlan,
) -> TerminalLaunchInvocation {
    let mut env = BTreeMap::from([(
        "CYRUNE_HOME".to_string(),
        plan.cyrune_home.display().to_string(),
    )]);
    if let Some(distribution_root_override) = &plan.distribution_root_override {
        env.insert(
            "CYRUNE_DISTRIBUTION_ROOT".to_string(),
            distribution_root_override.display().to_string(),
        );
    }
    TerminalLaunchInvocation {
        program: terminal_binary.to_path_buf(),
        args: vec![
            "start".to_string(),
            "--config-file".to_string(),
            plan.terminal_config_path.display().to_string(),
        ],
        env,
    }
}

pub fn prepare_packaged_launch_invocation(
    terminal_binary: &Path,
    cyrune_home: &Path,
) -> Result<(PackagedLaunchPlan, TerminalLaunchInvocation), PackagedLaunchFailure> {
    prepare_packaged_launch_invocation_with_distribution_root_override(
        terminal_binary,
        cyrune_home,
        None,
    )
}

pub fn launch_packaged_terminal(
    terminal_binary: &Path,
    cyrune_home: &Path,
) -> Result<PackagedLaunchExecution, PackagedLaunchFailure> {
    launch_packaged_terminal_with_distribution_root_override(terminal_binary, cyrune_home, None)
}

pub fn prepare_packaged_launch_with_distribution_root_override(
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<PackagedLaunchPlan, PackagedLaunchFailure> {
    let report = cyrune_daemon::command::packaged_launch_preflight_with_distribution_root_override(
        cyrune_home,
        distribution_root_override,
    )
    .map_err(|error| {
        packaged_preflight_failure_from_command_error(error, distribution_root_override.is_some())
    })?;
    packaged_launch_plan_from_health_report(report, distribution_root_override)
}

fn packaged_launch_plan_from_health_report(
    report: HealthReport,
    distribution_root_override: Option<&Path>,
) -> Result<PackagedLaunchPlan, PackagedLaunchFailure> {
    let distribution_root = PathBuf::from(&report.distribution_root);
    let bundle_root = PathBuf::from(&report.bundle_root);
    let terminal_config_path = PathBuf::from(&report.terminal_config_path);
    let runtime_pid_path = PathBuf::from(&report.runtime_pid_path);
    if report.status != "healthy" {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if !report.bundle_ready {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if !report.registry_ready {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if report.distribution_root.trim().is_empty() {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if report.bundle_root.trim().is_empty() {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if report.terminal_config_path.trim().is_empty() {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    if report.runtime_pid_path.trim().is_empty() {
        return Err(PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ));
    }
    Ok(PackagedLaunchPlan {
        cyrune_home: PathBuf::from(report.cyrune_home),
        distribution_root: distribution_root.clone(),
        bundle_root,
        terminal_config_path,
        runtime_pid_path,
        distribution_root_override: distribution_root_override.map(|_| distribution_root),
    })
}

fn prepare_packaged_launch_invocation_with_distribution_root_override(
    terminal_binary: &Path,
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<(PackagedLaunchPlan, TerminalLaunchInvocation), PackagedLaunchFailure> {
    let plan = prepare_packaged_launch_with_distribution_root_override(
        cyrune_home,
        distribution_root_override,
    )?;
    let invocation = build_terminal_launch_invocation(terminal_binary, &plan);
    Ok((plan, invocation))
}

pub fn launch_packaged_terminal_with_distribution_root_override(
    terminal_binary: &Path,
    cyrune_home: &Path,
    distribution_root_override: Option<&Path>,
) -> Result<PackagedLaunchExecution, PackagedLaunchFailure> {
    let (plan, invocation) = prepare_packaged_launch_invocation_with_distribution_root_override(
        terminal_binary,
        cyrune_home,
        distribution_root_override,
    )?;
    let mut command = Command::new(&invocation.program);
    command.args(&invocation.args);
    for (key, value) in &invocation.env {
        command.env(key, value);
    }
    let status = command.status().map_err(|_| {
        PackagedLaunchFailure::launcher(
            "terminal_binary_unavailable",
            "launcher terminal binary is unavailable",
        )
    })?;
    Ok(PackagedLaunchExecution {
        plan,
        invocation,
        exit_code: status.code().unwrap_or(1),
    })
}

fn packaged_preflight_failure_from_command_error(
    error: CommandError,
    had_distribution_root_override: bool,
) -> PackagedLaunchFailure {
    match error {
        CommandError::Invalid(_) if had_distribution_root_override => {
            PackagedLaunchFailure::preflight(
                "invalid_distribution_root_override",
                "packaged distribution root override is invalid",
            )
        }
        CommandError::Invalid(_) | CommandError::Public(_) => PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ),
        CommandError::Json(_)
        | CommandError::Io(_)
        | CommandError::Contract(_)
        | CommandError::Turn(_) => PackagedLaunchFailure::preflight(
            "packaged_launch_preflight_failed",
            "packaged launch preflight failed",
        ),
    }
}

fn parse_packaged_productization_identity(
    manifest: &Value,
) -> Result<Option<PackagedProductizationIdentity>, String> {
    let Some(identity) = manifest.get("productization_identity") else {
        return Ok(None);
    };
    let identity = identity.as_object().ok_or_else(|| {
        "RELEASE_MANIFEST.json productization_identity must be an object".to_string()
    })?;
    Ok(Some(PackagedProductizationIdentity {
        product_line_label: required_manifest_member_string(
            identity,
            "RELEASE_MANIFEST.json productization_identity",
            "product_line_label",
        )?,
        packaged_product_display_name: required_manifest_member_string(
            identity,
            "RELEASE_MANIFEST.json productization_identity",
            "packaged_product_display_name",
        )?,
        app_bundle_basename: required_manifest_member_string(
            identity,
            "RELEASE_MANIFEST.json productization_identity",
            "app_bundle_basename",
        )?,
        terminal_bundle_executable_stem: required_manifest_member_string(
            identity,
            "RELEASE_MANIFEST.json productization_identity",
            "terminal_bundle_executable_stem",
        )?,
    }))
}

fn parse_packaged_notice_bundle(
    distribution_root: &Path,
    manifest: &Value,
) -> Result<PackagedNoticeBundle, String> {
    let manifest = manifest
        .as_object()
        .ok_or_else(|| "RELEASE_MANIFEST.json must be an object".to_string())?;
    let license_bundle_path =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "license_bundle_path")?;
    let sbom_path =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "sbom_path")?;
    let license_bundle_root = resolve_distribution_relative_path(
        distribution_root,
        &license_bundle_path,
        "license_bundle_path",
    )?;
    if !license_bundle_root.is_dir() {
        return Err("RELEASE_MANIFEST.json license bundle is unavailable".to_string());
    }
    let third_party_notice_path = Path::new(&license_bundle_path)
        .join("THIRD-PARTY-NOTICES.md")
        .to_string_lossy()
        .to_string();
    let third_party_notice_file = resolve_distribution_relative_path(
        distribution_root,
        &third_party_notice_path,
        "license_bundle_path",
    )?;
    if !third_party_notice_file.is_file() {
        return Err("packaged notice bundle is incomplete".to_string());
    }
    for required_name in ["LICENSE-MIT.txt", "LICENSE-APACHE-2.0.txt"] {
        if !license_bundle_root.join(required_name).is_file() {
            return Err("packaged notice bundle is incomplete".to_string());
        }
    }

    let sbom_file = resolve_distribution_relative_path(distribution_root, &sbom_path, "sbom_path")?;
    if !sbom_file.is_file() {
        return Err("packaged sbom is unavailable".to_string());
    }
    let sbom_text = read_text_file(&sbom_file, "packaged sbom is invalid")?;
    let sbom: Value =
        serde_json::from_str(&sbom_text).map_err(|_| "packaged sbom is invalid".to_string())?;
    let spdx_version = sbom
        .get("spdxVersion")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "packaged sbom is invalid".to_string())?;
    if !spdx_version.starts_with("SPDX-") {
        return Err("packaged sbom is invalid".to_string());
    }

    let notice_text = read_text_file(
        &third_party_notice_file,
        "packaged notice bundle is incomplete",
    )?;
    if !notice_text.contains(&sbom_path) {
        return Err("packaged notice bundle does not match current sbom projection".to_string());
    }
    if let Some(identity) =
        parse_packaged_productization_identity(&Value::Object(manifest.clone()))?
    {
        for expected in [
            &identity.product_line_label,
            &identity.packaged_product_display_name,
            &identity.app_bundle_basename,
            &identity.terminal_bundle_executable_stem,
        ] {
            if !notice_text.contains(expected) {
                return Err(
                    "packaged notice bundle does not match current productization surface"
                        .to_string(),
                );
            }
        }
    }

    Ok(PackagedNoticeBundle {
        license_bundle_path,
        sbom_path,
        third_party_notice_path,
    })
}

fn parse_packaged_integrity_evidence(
    distribution_root: &Path,
    manifest: &Value,
) -> Result<PackagedIntegrityEvidence, String> {
    let manifest = manifest
        .as_object()
        .ok_or_else(|| "RELEASE_MANIFEST.json must be an object".to_string())?;
    let integrity_mode =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "integrity_mode")?;
    let signature_mode =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "signature_mode")?;
    let update_policy =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "update_policy")?;
    if integrity_mode != "sha256" {
        return Err("packaged integrity evidence is invalid".to_string());
    }
    if update_policy != "fixed-distribution/no-self-update" {
        return Err("packaged integrity evidence breaks no-self-update policy".to_string());
    }

    let hash_list_path = "SHA256SUMS.txt".to_string();
    let hash_list_file = distribution_root.join(&hash_list_path);
    if !hash_list_file.is_file() {
        return Err("packaged integrity evidence is incomplete".to_string());
    }
    let hash_list_text = read_text_file(&hash_list_file, "packaged integrity evidence is invalid")?;
    let hash_entries = parse_sha256sum_entries(&hash_list_text)?;
    for required_path in [
        "RELEASE_MANIFEST.json",
        "bin/cyr",
        "bin/cyrune-daemon",
        "share/licenses/LICENSE-MIT.txt",
        "share/licenses/LICENSE-APACHE-2.0.txt",
        "share/licenses/THIRD-PARTY-NOTICES.md",
        "share/sbom/cyrune-free-v0.1.spdx.json",
    ] {
        if !hash_entries.contains_key(required_path) {
            return Err(
                "packaged integrity evidence does not cover current package payload".to_string(),
            );
        }
    }

    Ok(PackagedIntegrityEvidence {
        integrity_mode,
        signature_mode,
        update_policy,
        hash_list_path,
    })
}

fn parse_packaged_inheritance_snapshot(
    distribution_root: &Path,
    manifest: &Value,
) -> Result<PackagedInheritanceSnapshot, String> {
    let manifest = manifest
        .as_object()
        .ok_or_else(|| "RELEASE_MANIFEST.json must be an object".to_string())?;
    let runtime_entry =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "runtime_entry")?;
    let daemon_entry =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "daemon_entry")?;
    let bundle_root_path =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "bundle_root_path")?;
    let home_template_path =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "home_template_path")?;
    if runtime_entry != "bin/cyr" || daemon_entry != "bin/cyrune-daemon" {
        return Err("packaged inheritance snapshot breaks single-entry family".to_string());
    }
    if bundle_root_path != "share/cyrune/bundle-root"
        || home_template_path != "share/cyrune/home-template"
    {
        return Err(
            "packaged inheritance snapshot breaks authority or projection family".to_string(),
        );
    }
    let bundle_root = resolve_distribution_relative_path(
        distribution_root,
        &bundle_root_path,
        "bundle_root_path",
    )?;
    let home_template_root = resolve_distribution_relative_path(
        distribution_root,
        &home_template_path,
        "home_template_path",
    )?;
    if !bundle_root.is_dir() || !home_template_root.is_dir() {
        return Err("packaged inheritance snapshot is incomplete".to_string());
    }
    if let Some(identity) = manifest.get("productization_identity") {
        let identity = identity.as_object().ok_or_else(|| {
            "RELEASE_MANIFEST.json productization_identity must be an object".to_string()
        })?;
        for forbidden_field in [
            "runtime_entry",
            "daemon_entry",
            "bundle_root_path",
            "home_template_path",
        ] {
            if identity.contains_key(forbidden_field) {
                return Err(
                    "packaged inheritance snapshot is shadowed by productization metadata"
                        .to_string(),
                );
            }
        }
    }

    Ok(PackagedInheritanceSnapshot {
        runtime_entry,
        daemon_entry,
        bundle_root_path,
        home_template_path,
    })
}

fn parse_packaged_upstream_intake_judgment(
    manifest: &Value,
) -> Result<PackagedUpstreamIntakeJudgment, String> {
    let manifest = manifest
        .as_object()
        .ok_or_else(|| "RELEASE_MANIFEST.json must be an object".to_string())?;
    let upstream_intake_mode =
        required_manifest_member_string(manifest, "RELEASE_MANIFEST.json", "upstream_intake_mode")?;
    let upstream_follow_triggers = required_manifest_member_string_array(
        manifest,
        "RELEASE_MANIFEST.json",
        "upstream_follow_triggers",
    )?;
    let upstream_auto_follow =
        required_manifest_member_bool(manifest, "RELEASE_MANIFEST.json", "upstream_auto_follow")?;
    if upstream_intake_mode != "evidence-based" {
        return Err("packaged upstream intake judgment is invalid".to_string());
    }
    if upstream_auto_follow {
        return Err("packaged upstream intake judgment enables auto-follow".to_string());
    }
    let expected_triggers = BTreeMap::from([
        ("critical_bug", ()),
        ("required_feature", ()),
        ("security", ()),
    ]);
    let actual_triggers = upstream_follow_triggers
        .iter()
        .map(String::as_str)
        .map(|trigger| (trigger, ()))
        .collect::<BTreeMap<_, _>>();
    if upstream_follow_triggers.len() != expected_triggers.len()
        || actual_triggers != expected_triggers
    {
        return Err("packaged upstream intake judgment breaks closed trigger set".to_string());
    }

    Ok(PackagedUpstreamIntakeJudgment {
        upstream_intake_mode,
        upstream_follow_triggers,
        upstream_auto_follow,
    })
}

fn read_release_preparation_metadata(distribution_root: &Path) -> Result<Value, String> {
    let metadata_path = distribution_root.join("RELEASE_PREPARATION.json");
    let metadata_text = fs::read_to_string(&metadata_path).map_err(|error| error.to_string())?;
    serde_json::from_str(&metadata_text).map_err(|error| error.to_string())
}

fn parse_release_preparation_bundle_identifier(
    distribution_root: &Path,
    metadata: &Value,
    notice_bundle: &PackagedNoticeBundle,
    identity: &PackagedProductizationIdentity,
) -> Result<String, String> {
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "RELEASE_PREPARATION.json must be an object".to_string())?;
    let value = required_manifest_member_string(
        metadata,
        "RELEASE_PREPARATION.json",
        "reverse_dns_bundle_identifier",
    )?;
    if !is_valid_reverse_dns_identifier(&value) {
        return Err("packaged reverse-DNS bundle identifier is invalid".to_string());
    }
    let expected =
        expected_reverse_dns_bundle_identifier(distribution_root, notice_bundle, identity)?;
    if value != expected {
        return Err("packaged reverse-DNS bundle identifier is invalid".to_string());
    }
    Ok(value)
}

fn parse_release_preparation_artifact(
    metadata: &Value,
    field_name: &str,
    expected_artifact_class: &str,
    expected_platform: &str,
    expected_emitted_name: &str,
) -> Result<PackagedReleasePreparationArtifact, String> {
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "RELEASE_PREPARATION.json must be an object".to_string())?;
    let artifact = metadata
        .get(field_name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("RELEASE_PREPARATION.json.{field_name} is missing"))?;
    let artifact_class = required_manifest_member_string(
        artifact,
        &format!("RELEASE_PREPARATION.json.{field_name}"),
        "artifact_class",
    )?;
    let platform = required_manifest_member_string(
        artifact,
        &format!("RELEASE_PREPARATION.json.{field_name}"),
        "platform",
    )?;
    let emitted_name = required_manifest_member_string(
        artifact,
        &format!("RELEASE_PREPARATION.json.{field_name}"),
        "emitted_name",
    )?;
    if artifact_class != expected_artifact_class
        || platform != expected_platform
        || emitted_name != expected_emitted_name
        || !is_safe_relative_name(&emitted_name)
    {
        return Err("packaged release artifact naming is invalid".to_string());
    }
    Ok(PackagedReleasePreparationArtifact {
        artifact_class,
        platform,
        emitted_name,
    })
}

fn parse_release_preparation_upstream_source_pin(
    metadata: &Value,
    upstream_intake_judgment: &PackagedUpstreamIntakeJudgment,
) -> Result<PackagedReleasePreparationUpstreamSourcePin, String> {
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "RELEASE_PREPARATION.json must be an object".to_string())?;
    let pin = metadata
        .get("upstream_source_pin")
        .and_then(Value::as_object)
        .ok_or_else(|| "RELEASE_PREPARATION.json.upstream_source_pin is missing".to_string())?;
    let source_project = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "source_project",
    )?;
    let source_kind = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "source_kind",
    )?;
    let exact_revision = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "exact_revision",
    )?;
    let source_archive = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "source_archive",
    )?;
    let evidence_origin = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "evidence_origin",
    )?;
    let source_reference_url = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "source_reference_url",
    )?;
    let upstream_intake_mode = required_manifest_member_string(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "upstream_intake_mode",
    )?;
    let upstream_follow_triggers = required_manifest_member_string_array(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "upstream_follow_triggers",
    )?;
    let upstream_auto_follow = required_manifest_member_bool(
        pin,
        "RELEASE_PREPARATION.json.upstream_source_pin",
        "upstream_auto_follow",
    )?;
    if source_project != "wezterm/wezterm"
        || source_kind != "github-release-tag"
        || evidence_origin != "official-github-release"
        || !is_valid_wezterm_release_tag(&exact_revision)
        || source_archive != format!("wezterm-{exact_revision}-src.tar.gz")
        || source_reference_url
            != format!("https://github.com/wezterm/wezterm/releases/tag/{exact_revision}")
        || upstream_intake_mode != upstream_intake_judgment.upstream_intake_mode
        || upstream_follow_triggers != upstream_intake_judgment.upstream_follow_triggers
        || upstream_auto_follow != upstream_intake_judgment.upstream_auto_follow
    {
        return Err("packaged upstream source pin is invalid".to_string());
    }
    Ok(PackagedReleasePreparationUpstreamSourcePin {
        source_project,
        source_kind,
        exact_revision,
        source_archive,
        evidence_origin,
        source_reference_url,
        upstream_intake_mode,
        upstream_follow_triggers,
        upstream_auto_follow,
    })
}

fn parse_release_preparation_organization_owned_value(
    metadata: &Value,
    field_name: &str,
) -> Result<String, String> {
    let metadata = metadata
        .as_object()
        .ok_or_else(|| "RELEASE_PREPARATION.json must be an object".to_string())?;
    let value = required_manifest_member_string(metadata, "RELEASE_PREPARATION.json", field_name)?;
    if value.trim().is_empty() {
        return Err(format!("RELEASE_PREPARATION.json.{field_name} is invalid"));
    }
    Ok(value)
}

fn expected_reverse_dns_bundle_identifier(
    distribution_root: &Path,
    notice_bundle: &PackagedNoticeBundle,
    identity: &PackagedProductizationIdentity,
) -> Result<String, String> {
    let sbom_file = resolve_distribution_relative_path(
        distribution_root,
        &notice_bundle.sbom_path,
        "sbom_path",
    )?;
    let sbom_text = read_text_file(&sbom_file, "packaged sbom is invalid")?;
    let sbom: Value =
        serde_json::from_str(&sbom_text).map_err(|_| "packaged sbom is invalid".to_string())?;
    let document_namespace = sbom
        .get("documentNamespace")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "packaged sbom is invalid".to_string())?;
    let host = extract_document_namespace_host(document_namespace)?;
    let reversed_host = host.split('.').rev().collect::<Vec<_>>().join(".");
    let suffix = normalize_identifier_suffix(&identity.product_line_label)?;
    Ok(format!("{reversed_host}.{suffix}"))
}

fn extract_document_namespace_host(document_namespace: &str) -> Result<String, String> {
    let remainder = document_namespace
        .strip_prefix("https://")
        .or_else(|| document_namespace.strip_prefix("http://"))
        .ok_or_else(|| "packaged sbom is invalid".to_string())?;
    let host = remainder
        .split('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "packaged sbom is invalid".to_string())?;
    Ok(host.to_string())
}

fn normalize_identifier_suffix(label: &str) -> Result<String, String> {
    let suffix = label
        .split_whitespace()
        .filter_map(|chunk| {
            let normalized = chunk
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .collect::<String>()
                .to_ascii_lowercase();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized)
            }
        })
        .next_back()
        .ok_or_else(|| "packaged reverse-DNS bundle identifier is invalid".to_string())?;
    Ok(suffix)
}

fn is_valid_reverse_dns_identifier(value: &str) -> bool {
    let parts = value.split('.').collect::<Vec<_>>();
    parts.len() >= 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && !part.starts_with('-')
                && !part.ends_with('-')
                && part.chars().all(|character| {
                    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
                })
        })
}

fn is_safe_relative_name(value: &str) -> bool {
    let path = Path::new(value);
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, std::path::Component::Normal(_)))
}

fn is_valid_wezterm_release_tag(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 24 {
        return false;
    }
    bytes[..8].iter().all(u8::is_ascii_digit)
        && bytes[8] == b'-'
        && bytes[9..15].iter().all(u8::is_ascii_digit)
        && bytes[15] == b'-'
        && bytes[16..24].iter().all(u8::is_ascii_hexdigit)
}

fn parse_sha256sum_entries(hash_list_text: &str) -> Result<BTreeMap<String, String>, String> {
    let mut entries = BTreeMap::new();
    for line in hash_list_text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let Some((digest, relative_path)) = line.split_once("  ") else {
            return Err("packaged integrity evidence is invalid".to_string());
        };
        let digest = digest.trim();
        let relative_path = relative_path.trim();
        if digest.len() != 64
            || !digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
            || relative_path.is_empty()
        {
            return Err("packaged integrity evidence is invalid".to_string());
        }
        entries.insert(relative_path.to_string(), digest.to_string());
    }
    if entries.is_empty() {
        return Err("packaged integrity evidence is invalid".to_string());
    }
    Ok(entries)
}

fn read_release_manifest(distribution_root: &Path) -> Result<Value, String> {
    let manifest_path = distribution_root.join("RELEASE_MANIFEST.json");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|error| error.to_string())?;
    serde_json::from_str(&manifest_text).map_err(|error| error.to_string())
}

fn required_manifest_member_string(
    object: &serde_json::Map<String, Value>,
    object_label: &str,
    key: &str,
) -> Result<String, String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{object_label}.{key} is missing"))?;
    if value.trim().is_empty() {
        return Err(format!("{object_label}.{key} is missing"));
    }
    Ok(value.to_string())
}

fn required_manifest_member_string_array(
    object: &serde_json::Map<String, Value>,
    object_label: &str,
    key: &str,
) -> Result<Vec<String>, String> {
    let values = object
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{object_label}.{key} is missing"))?;
    if values.is_empty() {
        return Err(format!("{object_label}.{key} is missing"));
    }
    let parsed = values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("{object_label}.{key} is missing"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parsed)
}

fn required_manifest_member_bool(
    object: &serde_json::Map<String, Value>,
    object_label: &str,
    key: &str,
) -> Result<bool, String> {
    object
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{object_label}.{key} is missing"))
}

fn resolve_distribution_relative_path(
    distribution_root: &Path,
    relative_path: &str,
    field_name: &str,
) -> Result<PathBuf, String> {
    let relative_path = Path::new(relative_path);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(format!(
            "RELEASE_MANIFEST.json {field_name} must be distribution-relative"
        ));
    }
    Ok(distribution_root.join(relative_path))
}

fn read_text_file(path: &Path, error_message: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|_| error_message.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        PackagedInheritanceSnapshot, PackagedIntegrityEvidence, PackagedLaunchFailureSurface,
        PackagedLaunchPlan, PackagedNoticeBundle,
        PackagedOrganizationOwnedReleasePreparationSnapshot, PackagedProductizationFailureSurface,
        PackagedProductizationIdentity, PackagedProductizationSnapshot,
        PackagedReleasePreparationArtifact, PackagedReleasePreparationFailureSurface,
        PackagedReleasePreparationSnapshot, PackagedReleasePreparationUpstreamSourcePin,
        PackagedUpstreamIntakeJudgment, build_terminal_launch_invocation,
        launch_packaged_terminal_with_distribution_root_override,
        prepare_packaged_launch_with_distribution_root_override,
        read_packaged_inheritance_snapshot, read_packaged_integrity_evidence,
        read_packaged_notice_bundle, read_packaged_productization_identity,
        read_packaged_upstream_intake_judgment, validate_packaged_productization,
        validate_packaged_release_preparation, validate_packaged_release_preparation_org_owned,
    };
    use std::fs;
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    fn write_text(path: &Path, text: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, text).unwrap();
    }

    fn make_executable(path: &Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(path, permissions).unwrap();
        }
    }

    fn write_packaged_distribution(root: &Path) -> PathBuf {
        let distribution_root = root.join("distribution");
        let bundle_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("bundle-root");
        let home_template_root = distribution_root
            .join("share")
            .join("cyrune")
            .join("home-template");

        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "distribution_unit": "cyrune-free-v0.1.tar.gz",
  "primary_os": "macOS",
  "productization_identity": {
    "app_bundle_basename": "CYRUNE.app",
    "packaged_product_display_name": "CYRUNE",
    "product_line_label": "CYRUNE Terminal",
    "terminal_bundle_executable_stem": "cyrune"
  },
  "bundle_root_path": "share/cyrune/bundle-root",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "daemon_entry": "bin/cyrune-daemon",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update",
  "upstream_intake_mode": "evidence-based",
  "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
  "upstream_auto_follow": false
}"#,
        );
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "cyrune-free-v0.1.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "notarization_provider": "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE",
  "reverse_dns_bundle_identifier": "local.cyrune.terminal",
  "signing_identity": "ORG_OWNED_SIGNING_IDENTITY_FIXTURE",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );
        write_text(
            &distribution_root.join("SHA256SUMS.txt"),
            concat!(
                "1111111111111111111111111111111111111111111111111111111111111111  RELEASE_MANIFEST.json\n",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa  RELEASE_PREPARATION.json\n",
                "2222222222222222222222222222222222222222222222222222222222222222  bin/cyr\n",
                "3333333333333333333333333333333333333333333333333333333333333333  bin/cyrune-daemon\n",
                "4444444444444444444444444444444444444444444444444444444444444444  share/licenses/LICENSE-MIT.txt\n",
                "5555555555555555555555555555555555555555555555555555555555555555  share/licenses/LICENSE-APACHE-2.0.txt\n",
                "6666666666666666666666666666666666666666666666666666666666666666  share/licenses/THIRD-PARTY-NOTICES.md\n",
                "7777777777777777777777777777777777777777777777777777777777777777  share/sbom/cyrune-free-v0.1.spdx.json\n",
            ),
        );
        write_text(
            &distribution_root
                .join("share")
                .join("licenses")
                .join("LICENSE-MIT.txt"),
            "MIT\n",
        );
        write_text(
            &distribution_root
                .join("share")
                .join("licenses")
                .join("LICENSE-APACHE-2.0.txt"),
            "Apache-2.0\n",
        );
        write_text(
            &distribution_root
                .join("share")
                .join("licenses")
                .join("THIRD-PARTY-NOTICES.md"),
            r#"# THIRD-PARTY NOTICES

## Current productization surface

- Product line label: `CYRUNE Terminal`
- Packaged product display name: `CYRUNE`
- App bundle basename: `CYRUNE.app`
- Terminal bundle executable stem: `cyrune`
- Current packaged product surface metadata is also carried in `RELEASE_MANIFEST.json.productization_identity`

## Dependency inventory

- Cargo dependency inventory and license expressions are recorded in `share/sbom/cyrune-free-v0.1.spdx.json`
"#,
        );
        write_text(
            &distribution_root
                .join("share")
                .join("sbom")
                .join("cyrune-free-v0.1.spdx.json"),
            r#"{
  "documentNamespace": "https://cyrune.local/sbom/cyrune-free-v0.1/2026-04-12T00:00:00Z",
  "spdxVersion": "SPDX-2.3"
}"#,
        );
        fs::create_dir_all(bundle_root.join("adapter").join("catalog")).unwrap();
        write_text(
            &bundle_root
                .join("adapter")
                .join("policies")
                .join("cyrune-free-default.v0.1.json"),
            "{}\n",
        );
        write_text(
            &bundle_root
                .join("adapter")
                .join("bindings")
                .join("cyrune-free-default.v0.1.json"),
            "{}\n",
        );
        write_text(
            &bundle_root
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("registry.json"),
            "{ \"registry_version\": \"v1\", \"entries\": [] }\n",
        );
        write_text(
            &bundle_root
                .join("registry")
                .join("execution-adapters")
                .join("approved")
                .join("profiles")
                .join("local-cli-single-process.v0.1.json"),
            "{ \"profile\": true }\n",
        );
        write_text(
            &bundle_root
                .join("terminal")
                .join("templates")
                .join("wezterm.lua"),
            "return {}\n",
        );
        let launcher_path = bundle_root
            .join("runtime")
            .join("ipc")
            .join("local-cli-single-process.sh");
        write_text(&launcher_path, "#!/bin/sh\nexit 0\n");
        make_executable(&launcher_path);
        write_text(
            &home_template_root
                .join("embedding")
                .join("exact-pins")
                .join("cyrune-free-shipping.v0.1.json"),
            "{\"pin\":true}\n",
        );
        distribution_root
    }

    #[test]
    fn prepare_packaged_launch_uses_packaged_metadata_and_projection() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let distribution_root = write_packaged_distribution(temp.path());

        let plan = prepare_packaged_launch_with_distribution_root_override(
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap();

        assert_eq!(plan.cyrune_home, cyrune_home);
        assert_eq!(
            plan.distribution_root,
            fs::canonicalize(&distribution_root).unwrap()
        );
        assert_eq!(
            plan.bundle_root,
            fs::canonicalize(
                temp.path()
                    .join("distribution")
                    .join("share")
                    .join("cyrune")
                    .join("bundle-root"),
            )
            .unwrap()
        );
        assert!(plan.terminal_config_path.is_file());
        assert!(
            cyrune_home
                .join("embedding")
                .join("exact-pins")
                .join("cyrune-free-shipping.v0.1.json")
                .is_file()
        );
        let identity = read_packaged_productization_identity(distribution_root.as_path())
            .unwrap()
            .unwrap();
        assert_eq!(
            identity,
            PackagedProductizationIdentity {
                product_line_label: "CYRUNE Terminal".to_string(),
                packaged_product_display_name: "CYRUNE".to_string(),
                app_bundle_basename: "CYRUNE.app".to_string(),
                terminal_bundle_executable_stem: "cyrune".to_string(),
            }
        );
        let notice_bundle = read_packaged_notice_bundle(distribution_root.as_path()).unwrap();
        assert_eq!(
            notice_bundle,
            PackagedNoticeBundle {
                license_bundle_path: "share/licenses".to_string(),
                sbom_path: "share/sbom/cyrune-free-v0.1.spdx.json".to_string(),
                third_party_notice_path: "share/licenses/THIRD-PARTY-NOTICES.md".to_string(),
            }
        );
        let integrity_evidence =
            read_packaged_integrity_evidence(distribution_root.as_path()).unwrap();
        assert_eq!(
            integrity_evidence,
            PackagedIntegrityEvidence {
                integrity_mode: "sha256".to_string(),
                signature_mode: "macos-adhoc".to_string(),
                update_policy: "fixed-distribution/no-self-update".to_string(),
                hash_list_path: "SHA256SUMS.txt".to_string(),
            }
        );
        let inheritance_snapshot =
            read_packaged_inheritance_snapshot(distribution_root.as_path()).unwrap();
        assert_eq!(
            inheritance_snapshot,
            PackagedInheritanceSnapshot {
                runtime_entry: "bin/cyr".to_string(),
                daemon_entry: "bin/cyrune-daemon".to_string(),
                bundle_root_path: "share/cyrune/bundle-root".to_string(),
                home_template_path: "share/cyrune/home-template".to_string(),
            }
        );
        let upstream_intake_judgment =
            read_packaged_upstream_intake_judgment(distribution_root.as_path()).unwrap();
        assert_eq!(
            upstream_intake_judgment,
            PackagedUpstreamIntakeJudgment {
                upstream_intake_mode: "evidence-based".to_string(),
                upstream_follow_triggers: vec![
                    "security".to_string(),
                    "critical_bug".to_string(),
                    "required_feature".to_string(),
                ],
                upstream_auto_follow: false,
            }
        );
    }

    #[test]
    fn build_terminal_launch_invocation_preserves_single_entry_and_override_only() {
        let plan = PackagedLaunchPlan {
            cyrune_home: PathBuf::from("/tmp/home"),
            distribution_root: PathBuf::from("/tmp/dist"),
            bundle_root: PathBuf::from("/tmp/dist/share/cyrune/bundle-root"),
            terminal_config_path: PathBuf::from("/tmp/home/terminal/config/wezterm.lua"),
            runtime_pid_path: PathBuf::from("/tmp/home/runtime/daemon.pid"),
            distribution_root_override: Some(PathBuf::from("/tmp/dist")),
        };

        let invocation =
            build_terminal_launch_invocation(Path::new("/usr/local/bin/wezterm"), &plan);

        assert_eq!(invocation.program, PathBuf::from("/usr/local/bin/wezterm"));
        assert_eq!(
            invocation.args,
            vec![
                "start".to_string(),
                "--config-file".to_string(),
                "/tmp/home/terminal/config/wezterm.lua".to_string(),
            ]
        );
        assert_eq!(
            invocation.env.get("CYRUNE_HOME"),
            Some(&"/tmp/home".to_string())
        );
        assert_eq!(
            invocation.env.get("CYRUNE_DISTRIBUTION_ROOT"),
            Some(&"/tmp/dist".to_string())
        );
        assert!(!invocation.env.contains_key("BUNDLE_ROOT"));
    }

    #[test]
    fn launch_packaged_terminal_executes_terminal_binary_with_expected_env() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let distribution_root = write_packaged_distribution(temp.path());
        let launch_probe = temp.path().join("launch-probe.txt");
        let terminal_binary = temp.path().join("fake-wezterm.sh");
        write_text(
            &terminal_binary,
            &format!(
                "#!/bin/sh\nprintf 'CYRUNE_HOME=%s\\n' \"$CYRUNE_HOME\" > \"{}\"\nprintf 'CYRUNE_DISTRIBUTION_ROOT=%s\\n' \"$CYRUNE_DISTRIBUTION_ROOT\" >> \"{}\"\nprintf 'ARGS=%s\\n' \"$*\" >> \"{}\"\nexit 0\n",
                launch_probe.display(),
                launch_probe.display(),
                launch_probe.display(),
            ),
        );
        make_executable(&terminal_binary);

        let execution = launch_packaged_terminal_with_distribution_root_override(
            &terminal_binary,
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap();

        let probe = fs::read_to_string(launch_probe).unwrap();
        assert_eq!(execution.exit_code, 0);
        assert!(probe.contains(&format!("CYRUNE_HOME={}", cyrune_home.display())));
        assert!(probe.contains(&format!(
            "CYRUNE_DISTRIBUTION_ROOT={}",
            fs::canonicalize(distribution_root).unwrap().display()
        )));
        assert!(probe.contains("ARGS=start --config-file"));
        assert!(probe.contains("terminal/config/wezterm.lua"));
    }

    #[test]
    fn preflight_failure_is_sanitized_and_does_not_leak_override_path() {
        let temp = tempdir().unwrap();
        let bad_override = temp
            .path()
            .join("sensitive")
            .join("host-only")
            .join("relative-root");
        let error = prepare_packaged_launch_with_distribution_root_override(
            &temp.path().join("home"),
            Some(bad_override.as_path()),
        )
        .unwrap_err();

        assert_eq!(
            error.surface,
            PackagedLaunchFailureSurface::PreflightFailure
        );
        assert_eq!(error.reason, "invalid_distribution_root_override");
        assert_eq!(
            error.message,
            "packaged distribution root override is invalid"
        );
        assert!(
            !error
                .to_string()
                .contains(&bad_override.display().to_string())
        );
        assert!(!error.to_string().contains("No such file"));
    }

    #[test]
    fn launcher_failure_is_sanitized_and_does_not_leak_terminal_binary_path() {
        let temp = tempdir().unwrap();
        let cyrune_home = temp.path().join("home");
        let distribution_root = write_packaged_distribution(temp.path());
        let missing_terminal = temp
            .path()
            .join("missing")
            .join("private")
            .join("wezterm-not-there");

        let error = launch_packaged_terminal_with_distribution_root_override(
            &missing_terminal,
            &cyrune_home,
            Some(distribution_root.as_path()),
        )
        .unwrap_err();

        assert_eq!(error.surface, PackagedLaunchFailureSurface::LauncherFailure);
        assert_eq!(error.reason, "terminal_binary_unavailable");
        assert_eq!(error.message, "launcher terminal binary is unavailable");
        assert!(
            !error
                .to_string()
                .contains(&missing_terminal.display().to_string())
        );
        assert!(!error.to_string().contains("No such file"));
    }

    #[test]
    fn read_packaged_productization_identity_returns_none_when_block_is_absent() {
        let temp = tempdir().unwrap();
        let distribution_root = temp.path().join("distribution");
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "home_template_path": "share/cyrune/home-template",
  "runtime_entry": "bin/cyr",
  "daemon_entry": "bin/cyrune-daemon"
}"#,
        );

        let identity = read_packaged_productization_identity(distribution_root.as_path()).unwrap();
        assert!(identity.is_none());
    }

    #[test]
    fn read_packaged_productization_identity_rejects_blank_values() {
        let temp = tempdir().unwrap();
        let distribution_root = temp.path().join("distribution");
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "productization_identity": {
    "app_bundle_basename": "CYRUNE.app",
    "packaged_product_display_name": "CYRUNE",
    "product_line_label": "",
    "terminal_bundle_executable_stem": "cyrune"
  }
}"#,
        );

        let error = read_packaged_productization_identity(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "RELEASE_MANIFEST.json productization_identity.product_line_label is missing"
        );
    }

    #[test]
    fn read_packaged_notice_bundle_rejects_missing_notice_file() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        fs::remove_file(
            distribution_root
                .join("share")
                .join("licenses")
                .join("THIRD-PARTY-NOTICES.md"),
        )
        .unwrap();

        let error = read_packaged_notice_bundle(distribution_root.as_path()).unwrap_err();
        assert_eq!(error, "packaged notice bundle is incomplete");
    }

    #[test]
    fn read_packaged_notice_bundle_rejects_surface_drift() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root
                .join("share")
                .join("licenses")
                .join("THIRD-PARTY-NOTICES.md"),
            r#"# THIRD-PARTY NOTICES

## Current productization surface

- Product line label: `Different`
- Packaged product display name: `Else`
- Cargo dependency inventory and license expressions are recorded in `share/sbom/cyrune-free-v0.1.spdx.json`
"#,
        );

        let error = read_packaged_notice_bundle(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged notice bundle does not match current productization surface"
        );
    }

    #[test]
    fn read_packaged_integrity_evidence_rejects_missing_hash_list() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        fs::remove_file(distribution_root.join("SHA256SUMS.txt")).unwrap();

        let error = read_packaged_integrity_evidence(distribution_root.as_path()).unwrap_err();
        assert_eq!(error, "packaged integrity evidence is incomplete");
    }

    #[test]
    fn read_packaged_integrity_evidence_rejects_update_policy_drift() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "daemon_entry": "bin/cyrune-daemon",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "rolling"
}"#,
        );

        let error = read_packaged_integrity_evidence(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged integrity evidence breaks no-self-update policy"
        );
    }

    #[test]
    fn read_packaged_upstream_intake_judgment_rejects_closed_trigger_drift() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "daemon_entry": "bin/cyrune-daemon",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update",
  "upstream_intake_mode": "evidence-based",
  "upstream_follow_triggers": ["security", "optional_feature"],
  "upstream_auto_follow": false
}"#,
        );

        let error =
            read_packaged_upstream_intake_judgment(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged upstream intake judgment breaks closed trigger set"
        );
    }

    #[test]
    fn read_packaged_upstream_intake_judgment_rejects_auto_follow() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "daemon_entry": "bin/cyrune-daemon",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update",
  "upstream_intake_mode": "evidence-based",
  "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
  "upstream_auto_follow": true
}"#,
        );

        let error =
            read_packaged_upstream_intake_judgment(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged upstream intake judgment enables auto-follow"
        );
    }

    #[test]
    fn validate_packaged_productization_accepts_current_manifest() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());

        let snapshot = validate_packaged_productization(distribution_root.as_path()).unwrap();

        assert_eq!(
            snapshot,
            PackagedProductizationSnapshot {
                identity: PackagedProductizationIdentity {
                    product_line_label: "CYRUNE Terminal".to_string(),
                    packaged_product_display_name: "CYRUNE".to_string(),
                    app_bundle_basename: "CYRUNE.app".to_string(),
                    terminal_bundle_executable_stem: "cyrune".to_string(),
                },
                notice_bundle: PackagedNoticeBundle {
                    license_bundle_path: "share/licenses".to_string(),
                    sbom_path: "share/sbom/cyrune-free-v0.1.spdx.json".to_string(),
                    third_party_notice_path: "share/licenses/THIRD-PARTY-NOTICES.md".to_string(),
                },
                integrity_evidence: PackagedIntegrityEvidence {
                    integrity_mode: "sha256".to_string(),
                    signature_mode: "macos-adhoc".to_string(),
                    update_policy: "fixed-distribution/no-self-update".to_string(),
                    hash_list_path: "SHA256SUMS.txt".to_string(),
                },
                inheritance_snapshot: PackagedInheritanceSnapshot {
                    runtime_entry: "bin/cyr".to_string(),
                    daemon_entry: "bin/cyrune-daemon".to_string(),
                    bundle_root_path: "share/cyrune/bundle-root".to_string(),
                    home_template_path: "share/cyrune/home-template".to_string(),
                },
                upstream_intake_judgment: PackagedUpstreamIntakeJudgment {
                    upstream_intake_mode: "evidence-based".to_string(),
                    upstream_follow_triggers: vec![
                        "security".to_string(),
                        "critical_bug".to_string(),
                        "required_feature".to_string(),
                    ],
                    upstream_auto_follow: false,
                },
            }
        );
    }

    #[test]
    fn validate_packaged_productization_rejects_notice_bundle_with_productization_surface() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        fs::remove_file(
            distribution_root
                .join("share")
                .join("licenses")
                .join("THIRD-PARTY-NOTICES.md"),
        )
        .unwrap();

        let error = validate_packaged_productization(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedProductizationFailureSurface::ProductizationFailure
        );
        assert_eq!(error.reason, "notice_bundle_invalid");
        assert_eq!(error.message, "packaged notice bundle is invalid");
        assert!(!error.to_string().contains("No such file"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_productization_rejects_missing_manifest_without_path_leak() {
        let temp = tempdir().unwrap();
        let distribution_root = temp
            .path()
            .join("private")
            .join("host-only")
            .join("distribution");

        let error = validate_packaged_productization(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedProductizationFailureSurface::ProductizationFailure
        );
        assert_eq!(error.reason, "productization_metadata_invalid");
        assert_eq!(error.message, "packaged productization metadata is invalid");
        assert!(!error.to_string().contains("No such file"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_productization_rejects_upstream_policy_without_raw_detail_leak() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "productization_identity": {
    "app_bundle_basename": "CYRUNE.app",
    "packaged_product_display_name": "CYRUNE",
    "product_line_label": "CYRUNE Terminal",
    "terminal_bundle_executable_stem": "cyrune"
  },
  "bundle_root_path": "share/cyrune/bundle-root",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "daemon_entry": "bin/cyrune-daemon",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update",
  "upstream_intake_mode": "evidence-based",
  "upstream_follow_triggers": ["security", "optional_feature"],
  "upstream_auto_follow": false
}"#,
        );

        let error = validate_packaged_productization(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedProductizationFailureSurface::ProductizationFailure
        );
        assert_eq!(error.reason, "upstream_intake_judgment_invalid");
        assert_eq!(
            error.message,
            "packaged upstream intake judgment is invalid"
        );
        assert!(!error.to_string().contains("optional_feature"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_accepts_rule_fixed_family() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());

        let snapshot = validate_packaged_release_preparation(distribution_root.as_path()).unwrap();

        assert_eq!(
            snapshot,
            PackagedReleasePreparationSnapshot {
                reverse_dns_bundle_identifier: "local.cyrune.terminal".to_string(),
                installer_artifact: PackagedReleasePreparationArtifact {
                    artifact_class: "app_bundle".to_string(),
                    platform: "macOS".to_string(),
                    emitted_name: "CYRUNE.app".to_string(),
                },
                archive_artifact: PackagedReleasePreparationArtifact {
                    artifact_class: "distribution_archive".to_string(),
                    platform: "macOS".to_string(),
                    emitted_name: "cyrune-free-v0.1.tar.gz".to_string(),
                },
                upstream_source_pin: PackagedReleasePreparationUpstreamSourcePin {
                    source_project: "wezterm/wezterm".to_string(),
                    source_kind: "github-release-tag".to_string(),
                    exact_revision: "20240203-110809-5046fc22".to_string(),
                    source_archive: "wezterm-20240203-110809-5046fc22-src.tar.gz".to_string(),
                    evidence_origin: "official-github-release".to_string(),
                    source_reference_url:
                        "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22"
                            .to_string(),
                    upstream_intake_mode: "evidence-based".to_string(),
                    upstream_follow_triggers: vec![
                        "security".to_string(),
                        "critical_bug".to_string(),
                        "required_feature".to_string(),
                    ],
                    upstream_auto_follow: false,
                },
            }
        );
    }

    #[test]
    fn validate_packaged_release_preparation_org_owned_accepts_organization_owned_family() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());

        let snapshot =
            validate_packaged_release_preparation_org_owned(distribution_root.as_path()).unwrap();

        assert_eq!(
            snapshot,
            PackagedOrganizationOwnedReleasePreparationSnapshot {
                release_preparation: PackagedReleasePreparationSnapshot {
                    reverse_dns_bundle_identifier: "local.cyrune.terminal".to_string(),
                    installer_artifact: PackagedReleasePreparationArtifact {
                        artifact_class: "app_bundle".to_string(),
                        platform: "macOS".to_string(),
                        emitted_name: "CYRUNE.app".to_string(),
                    },
                    archive_artifact: PackagedReleasePreparationArtifact {
                        artifact_class: "distribution_archive".to_string(),
                        platform: "macOS".to_string(),
                        emitted_name: "cyrune-free-v0.1.tar.gz".to_string(),
                    },
                    upstream_source_pin: PackagedReleasePreparationUpstreamSourcePin {
                        source_project: "wezterm/wezterm".to_string(),
                        source_kind: "github-release-tag".to_string(),
                        exact_revision: "20240203-110809-5046fc22".to_string(),
                        source_archive: "wezterm-20240203-110809-5046fc22-src.tar.gz".to_string(),
                        evidence_origin: "official-github-release".to_string(),
                        source_reference_url:
                            "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22"
                                .to_string(),
                        upstream_intake_mode: "evidence-based".to_string(),
                        upstream_follow_triggers: vec![
                            "security".to_string(),
                            "critical_bug".to_string(),
                            "required_feature".to_string(),
                        ],
                        upstream_auto_follow: false,
                    },
                },
                signing_identity: "ORG_OWNED_SIGNING_IDENTITY_FIXTURE".to_string(),
                notarization_provider: "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE".to_string(),
            }
        );
    }

    #[test]
    fn validate_packaged_release_preparation_rejects_malformed_bundle_identifier_without_leak() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "cyrune-free-v0.1.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "reverse_dns_bundle_identifier": "Terminal",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );

        let error = validate_packaged_release_preparation(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "bundle_identifier_invalid");
        assert_eq!(
            error.message,
            "packaged reverse-DNS bundle identifier is invalid"
        );
        assert!(!error.to_string().contains("Terminal"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_rejects_artifact_naming_drift_without_leak() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "../private.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "reverse_dns_bundle_identifier": "local.cyrune.terminal",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );

        let error = validate_packaged_release_preparation(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "artifact_naming_invalid");
        assert_eq!(error.message, "packaged release artifact naming is invalid");
        assert!(!error.to_string().contains("../private.tar.gz"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_rejects_upstream_pin_drift_without_leak() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "cyrune-free-v0.1.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "reverse_dns_bundle_identifier": "local.cyrune.terminal",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "optional_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );

        let error = validate_packaged_release_preparation(distribution_root.as_path()).unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "upstream_source_pin_invalid");
        assert_eq!(error.message, "packaged upstream source pin is invalid");
        assert!(!error.to_string().contains("optional_feature"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_org_owned_rejects_missing_signing_identity_without_leak()
     {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "cyrune-free-v0.1.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "notarization_provider": "ORG_OWNED_NOTARIZATION_PROVIDER_FIXTURE",
  "reverse_dns_bundle_identifier": "local.cyrune.terminal",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );

        let error = validate_packaged_release_preparation_org_owned(distribution_root.as_path())
            .unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "signing_identity_invalid");
        assert_eq!(error.message, "packaged signing identity is invalid");
        assert!(!error.to_string().contains("signing_identity"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_org_owned_rejects_invalid_notarization_provider_without_leak()
     {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_PREPARATION.json"),
            r#"{
  "archive_artifact": {
    "artifact_class": "distribution_archive",
    "emitted_name": "cyrune-free-v0.1.tar.gz",
    "platform": "macOS"
  },
  "installer_artifact": {
    "artifact_class": "app_bundle",
    "emitted_name": "CYRUNE.app",
    "platform": "macOS"
  },
  "metadata_version": "d7-rc1-rule-fixed.v1",
  "notarization_provider": {
    "account": "PRIVATE-NOTARY-PROVIDER"
  },
  "reverse_dns_bundle_identifier": "local.cyrune.terminal",
  "signing_identity": "ORG_OWNED_SIGNING_IDENTITY_FIXTURE",
  "upstream_source_pin": {
    "evidence_origin": "official-github-release",
    "exact_revision": "20240203-110809-5046fc22",
    "source_archive": "wezterm-20240203-110809-5046fc22-src.tar.gz",
    "source_kind": "github-release-tag",
    "source_project": "wezterm/wezterm",
    "source_reference_url": "https://github.com/wezterm/wezterm/releases/tag/20240203-110809-5046fc22",
    "upstream_auto_follow": false,
    "upstream_follow_triggers": ["security", "critical_bug", "required_feature"],
    "upstream_intake_mode": "evidence-based"
  }
}"#,
        );

        let error = validate_packaged_release_preparation_org_owned(distribution_root.as_path())
            .unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "notarization_provider_invalid");
        assert_eq!(error.message, "packaged notarization provider is invalid");
        assert!(!error.to_string().contains("PRIVATE-NOTARY-PROVIDER"));
        assert!(!error.to_string().contains("notarization_provider"));
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn validate_packaged_release_preparation_org_owned_keeps_root_metadata_invalid_boundary() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(&distribution_root.join("RELEASE_PREPARATION.json"), "[]\n");

        let error = validate_packaged_release_preparation_org_owned(distribution_root.as_path())
            .unwrap_err();

        assert_eq!(
            error.surface,
            PackagedReleasePreparationFailureSurface::ReleasePreparationFailure
        );
        assert_eq!(error.reason, "release_preparation_metadata_invalid");
        assert_eq!(
            error.message,
            "packaged release preparation metadata is invalid"
        );
        assert!(
            !error
                .to_string()
                .contains(&distribution_root.display().to_string())
        );
    }

    #[test]
    fn read_packaged_inheritance_snapshot_rejects_single_entry_drift() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "bundle_root_path": "share/cyrune/bundle-root",
  "daemon_entry": "bin/cyrune-daemon",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyrune",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update"
}"#,
        );

        let error = read_packaged_inheritance_snapshot(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged inheritance snapshot breaks single-entry family"
        );
    }

    #[test]
    fn read_packaged_inheritance_snapshot_rejects_shadowed_authority_fields() {
        let temp = tempdir().unwrap();
        let distribution_root = write_packaged_distribution(temp.path());
        write_text(
            &distribution_root.join("RELEASE_MANIFEST.json"),
            r#"{
  "productization_identity": {
    "app_bundle_basename": "CYRUNE.app",
    "bundle_root_path": "shadowed",
    "packaged_product_display_name": "CYRUNE",
    "product_line_label": "CYRUNE Terminal",
    "terminal_bundle_executable_stem": "cyrune"
  },
  "bundle_root_path": "share/cyrune/bundle-root",
  "daemon_entry": "bin/cyrune-daemon",
  "home_template_path": "share/cyrune/home-template",
  "integrity_mode": "sha256",
  "license_bundle_path": "share/licenses",
  "runtime_entry": "bin/cyr",
  "sbom_path": "share/sbom/cyrune-free-v0.1.spdx.json",
  "signature_mode": "macos-adhoc",
  "update_policy": "fixed-distribution/no-self-update"
}"#,
        );

        let error = read_packaged_inheritance_snapshot(distribution_root.as_path()).unwrap_err();
        assert_eq!(
            error,
            "packaged inheritance snapshot is shadowed by productization metadata"
        );
    }
}
