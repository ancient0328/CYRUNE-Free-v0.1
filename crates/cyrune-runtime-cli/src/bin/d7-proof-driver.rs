#![forbid(unsafe_code)]

use cyrune_runtime_cli::pack::{
    validate_packaged_productization, validate_packaged_release_preparation,
    validate_packaged_release_preparation_org_owned,
};
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn main() {
    std::process::exit(run().unwrap_or_else(|error| {
        eprintln!("{error}");
        2
    }));
}

fn run() -> Result<i32, String> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage().to_string());
    };

    match command {
        "validate" => run_validate(&args[1..]),
        "validate-rc1-b" => run_validate_rc1_b(&args[1..]),
        "validate-rc1-c" => run_validate_rc1_c(&args[1..]),
        _ => Err(usage().to_string()),
    }
}

fn run_validate(args: &[String]) -> Result<i32, String> {
    let mut distribution_root = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--distribution-root" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--distribution-root requires <path>".to_string())?;
                distribution_root = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    let distribution_root =
        distribution_root.ok_or_else(|| "--distribution-root requires <path>".to_string())?;

    match validate_packaged_productization(distribution_root.as_path()) {
        Ok(snapshot) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "validated",
                    "snapshot": {
                        "identity": {
                            "product_line_label": snapshot.identity.product_line_label,
                            "packaged_product_display_name": snapshot.identity.packaged_product_display_name,
                            "app_bundle_basename": snapshot.identity.app_bundle_basename,
                            "terminal_bundle_executable_stem": snapshot.identity.terminal_bundle_executable_stem,
                        },
                        "notice_bundle": {
                            "license_bundle_path": snapshot.notice_bundle.license_bundle_path,
                            "sbom_path": snapshot.notice_bundle.sbom_path,
                            "third_party_notice_path": snapshot.notice_bundle.third_party_notice_path,
                        },
                        "integrity_evidence": {
                            "integrity_mode": snapshot.integrity_evidence.integrity_mode,
                            "signature_mode": snapshot.integrity_evidence.signature_mode,
                            "update_policy": snapshot.integrity_evidence.update_policy,
                            "hash_list_path": snapshot.integrity_evidence.hash_list_path,
                        },
                        "inheritance_snapshot": {
                            "runtime_entry": snapshot.inheritance_snapshot.runtime_entry,
                            "daemon_entry": snapshot.inheritance_snapshot.daemon_entry,
                            "bundle_root_path": snapshot.inheritance_snapshot.bundle_root_path,
                            "home_template_path": snapshot.inheritance_snapshot.home_template_path,
                        },
                        "upstream_intake_judgment": {
                            "upstream_intake_mode": snapshot.upstream_intake_judgment.upstream_intake_mode,
                            "upstream_follow_triggers": snapshot.upstream_intake_judgment.upstream_follow_triggers,
                            "upstream_auto_follow": snapshot.upstream_intake_judgment.upstream_auto_follow,
                        },
                    }
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(0)
        }
        Err(failure) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "failed",
                    "surface": failure.surface.as_str(),
                    "reason": failure.reason,
                    "message": failure.message,
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(1)
        }
    }
}

fn run_validate_rc1_b(args: &[String]) -> Result<i32, String> {
    let mut distribution_root = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--distribution-root" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--distribution-root requires <path>".to_string())?;
                distribution_root = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    let distribution_root =
        distribution_root.ok_or_else(|| "--distribution-root requires <path>".to_string())?;

    match validate_packaged_release_preparation(distribution_root.as_path()) {
        Ok(snapshot) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "validated",
                    "snapshot": {
                        "reverse_dns_bundle_identifier": snapshot.reverse_dns_bundle_identifier,
                        "installer_artifact": {
                            "artifact_class": snapshot.installer_artifact.artifact_class,
                            "platform": snapshot.installer_artifact.platform,
                            "emitted_name": snapshot.installer_artifact.emitted_name,
                        },
                        "archive_artifact": {
                            "artifact_class": snapshot.archive_artifact.artifact_class,
                            "platform": snapshot.archive_artifact.platform,
                            "emitted_name": snapshot.archive_artifact.emitted_name,
                        },
                        "upstream_source_pin": {
                            "source_project": snapshot.upstream_source_pin.source_project,
                            "source_kind": snapshot.upstream_source_pin.source_kind,
                            "exact_revision": snapshot.upstream_source_pin.exact_revision,
                            "source_archive": snapshot.upstream_source_pin.source_archive,
                            "evidence_origin": snapshot.upstream_source_pin.evidence_origin,
                            "source_reference_url": snapshot.upstream_source_pin.source_reference_url,
                            "upstream_intake_mode": snapshot.upstream_source_pin.upstream_intake_mode,
                            "upstream_follow_triggers": snapshot.upstream_source_pin.upstream_follow_triggers,
                            "upstream_auto_follow": snapshot.upstream_source_pin.upstream_auto_follow,
                        },
                    }
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(0)
        }
        Err(failure) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "failed",
                    "surface": failure.surface.as_str(),
                    "reason": failure.reason,
                    "message": failure.message,
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(1)
        }
    }
}

fn run_validate_rc1_c(args: &[String]) -> Result<i32, String> {
    let mut distribution_root = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--distribution-root" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--distribution-root requires <path>".to_string())?;
                distribution_root = Some(PathBuf::from(value));
                index += 2;
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    let distribution_root =
        distribution_root.ok_or_else(|| "--distribution-root requires <path>".to_string())?;

    match validate_packaged_release_preparation_org_owned(distribution_root.as_path()) {
        Ok(snapshot) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "validated",
                    "snapshot": {
                        "signing_identity": snapshot.signing_identity,
                        "notarization_provider": snapshot.notarization_provider,
                    }
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(0)
        }
        Err(failure) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "failed",
                    "surface": failure.surface.as_str(),
                    "reason": failure.reason,
                    "message": failure.message,
                }))
                .map_err(|error| error.to_string())?
            );
            Ok(1)
        }
    }
}

fn usage() -> &'static str {
    "usage: d7-proof-driver <validate|validate-rc1-b|validate-rc1-c> --distribution-root <path>"
}
