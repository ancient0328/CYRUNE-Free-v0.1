#![forbid(unsafe_code)]

use crate::cli::{build_run_payload, invoke_daemon_single_capture_stderr, render_json_payload};
use crate::pack::default_cyrune_home;
use cyrune_core_contract::{CorrelationId, IoMode, RequestId, RunKind, RunRequest};
use cyrune_daemon::ipc::IpcCommand;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;

const SCHEMA_VERSION: &str = "cyrune.free.first-success-verifier-report.v1";
const TERMINAL_BINDING_SCHEMA_VERSION: &str = "cyrune.free.terminal-binding.v1";
const PUBLIC_FIRST_SUCCESS_INPUT: &str = "ship-goal public first success";
const DEFAULT_POLICY_PACK_ID: &str = "cyrune-free-default";
const RUN_MODE: &str = "no_llm";

pub fn run_verify(args: &[String]) -> Result<i32, String> {
    match args.first().map(String::as_str) {
        Some("first-success") if args.len() == 1 => run_first_success_verifier(),
        _ => Err("usage: cyr verify first-success".to_string()),
    }
}

pub fn run_first_success_verifier() -> Result<i32, String> {
    let cyrune_home = match default_cyrune_home().and_then(|path| resolve_absolute(&path)) {
        Ok(path) => path,
        Err(error) => {
            return emit_failure_report(
                "FSV-PRECONDITION",
                format!("failed to resolve CYRUNE_HOME: {error}"),
                Vec::new(),
                None,
                None,
                None,
            );
        }
    };
    let state_root = match cyrune_home.parent().map(Path::to_path_buf) {
        Some(path) => path,
        None => {
            return emit_failure_report(
                "FSV-PRECONDITION",
                "CYRUNE_HOME has no parent path",
                Vec::new(),
                None,
                None,
                Some(&cyrune_home),
            );
        }
    };
    let request = match public_first_success_request() {
        Ok(request) => request,
        Err(error) => {
            return emit_failure_report(
                "FSV-PRECONDITION",
                format!("failed to construct public first-success request: {error}"),
                Vec::new(),
                None,
                Some(&state_root),
                Some(&cyrune_home),
            );
        }
    };
    let payload = match build_run_payload(&request) {
        Ok(payload) => payload,
        Err(error) => {
            return emit_failure_report(
                "FSV-PRECONDITION",
                format!("failed to construct public first-success payload: {error}"),
                Vec::new(),
                None,
                Some(&state_root),
                Some(&cyrune_home),
            );
        }
    };

    let captured = match invoke_daemon_single_capture_stderr(IpcCommand::Run, payload) {
        Ok(captured) => captured,
        Err(error) => {
            return emit_failure_report(
                "FSV-RUNTIME",
                format!("daemon/run invocation failed: {error}"),
                Vec::new(),
                None,
                Some(&state_root),
                Some(&cyrune_home),
            );
        }
    };

    match verify_first_success_payload(
        captured.payload,
        captured.diagnostics,
        &state_root,
        &cyrune_home,
    ) {
        Ok(report) => {
            println!("{}", render_json_payload(&report)?);
            Ok(0)
        }
        Err(failure) => emit_failure_report(
            failure.code,
            failure.message,
            failure.diagnostics,
            failure.response,
            Some(&state_root),
            Some(&cyrune_home),
        ),
    }
}

struct VerificationFailure {
    code: &'static str,
    message: String,
    diagnostics: Vec<String>,
    response: Option<Value>,
}

fn verify_first_success_payload(
    response: Value,
    diagnostics: Vec<String>,
    state_root: &Path,
    cyrune_home: &Path,
) -> Result<Value, VerificationFailure> {
    if !response.is_object() {
        return Err(VerificationFailure {
            code: "FSV-REJECTED-PAYLOAD",
            message: "run response is not a JSON object".to_string(),
            diagnostics: diagnostics.clone(),
            response: Some(response),
        });
    }
    let outcome = required_string(&response, "outcome", &diagnostics)?;
    if outcome != "accepted" {
        return Err(VerificationFailure {
            code: "FSV-REJECTED-PAYLOAD",
            message: format!("run response outcome is not accepted: {outcome}"),
            diagnostics,
            response: Some(response),
        });
    }
    let response_to = required_string(&response, "response_to", &diagnostics)?;
    let correlation_id = required_string(&response, "correlation_id", &diagnostics)?;
    let run_id = required_string(&response, "run_id", &diagnostics)?;
    let evidence_id = required_string(&response, "evidence_id", &diagnostics)?;
    let policy_pack_id = required_string(&response, "policy_pack_id", &diagnostics)?;
    let citation_bundle_id = required_string(&response, "citation_bundle_id", &diagnostics)?;
    let working_hash_after = required_string(&response, "working_hash_after", &diagnostics)?;
    if policy_pack_id != DEFAULT_POLICY_PACK_ID {
        return Err(VerificationFailure {
            code: "FSV-POLICY-MISMATCH",
            message: format!("policy_pack_id is not {DEFAULT_POLICY_PACK_ID}: {policy_pack_id}"),
            diagnostics,
            response: Some(response),
        });
    }

    let evidence_dir_rel = PathBuf::from("ledger").join("evidence").join(&evidence_id);
    let terminal_binding_rel = PathBuf::from("ledger")
        .join("terminal-bindings")
        .join(format!("{evidence_id}.json"));
    let evidence_dir = cyrune_home.join(&evidence_dir_rel);
    let terminal_binding_path = cyrune_home.join(&terminal_binding_rel);
    if !evidence_dir.is_dir() {
        return Err(VerificationFailure {
            code: "FSV-EVIDENCE-MISSING",
            message: format!(
                "accepted evidence directory missing: {}",
                evidence_dir.display()
            ),
            diagnostics,
            response: Some(response),
        });
    }
    if !terminal_binding_path.is_file() {
        return Err(VerificationFailure {
            code: "FSV-TERMINAL-BINDING-MISSING",
            message: format!(
                "terminal binding marker missing: {}",
                terminal_binding_path.display()
            ),
            diagnostics,
            response: Some(response),
        });
    }

    let manifest = read_json(&evidence_dir.join("manifest.json"), "FSV-EVIDENCE-MISSING").map_err(
        |mut failure| {
            failure.diagnostics = diagnostics.clone();
            failure.response = Some(response.clone());
            failure
        },
    )?;
    let run = read_json(&evidence_dir.join("run.json"), "FSV-EVIDENCE-MISSING").map_err(
        |mut failure| {
            failure.diagnostics = diagnostics.clone();
            failure.response = Some(response.clone());
            failure
        },
    )?;
    let policy = read_json(&evidence_dir.join("policy.json"), "FSV-EVIDENCE-MISSING").map_err(
        |mut failure| {
            failure.diagnostics = diagnostics.clone();
            failure.response = Some(response.clone());
            failure
        },
    )?;
    let citation = read_json(
        &evidence_dir.join("citation_bundle.json"),
        "FSV-EVIDENCE-MISSING",
    )
    .map_err(|mut failure| {
        failure.diagnostics = diagnostics.clone();
        failure.response = Some(response.clone());
        failure
    })?;
    let working_delta = read_json(
        &evidence_dir.join("working_delta.json"),
        "FSV-EVIDENCE-MISSING",
    )
    .map_err(|mut failure| {
        failure.diagnostics = diagnostics.clone();
        failure.response = Some(response.clone());
        failure
    })?;
    let hashes = read_json(&evidence_dir.join("hashes.json"), "FSV-EVIDENCE-MISSING").map_err(
        |mut failure| {
            failure.diagnostics = diagnostics.clone();
            failure.response = Some(response.clone());
            failure
        },
    )?;
    let terminal = read_json(&terminal_binding_path, "FSV-TERMINAL-BINDING-MISSING").map_err(
        |mut failure| {
            failure.diagnostics = diagnostics.clone();
            failure.response = Some(response.clone());
            failure
        },
    )?;

    expect_eq(
        &manifest,
        "outcome",
        "accepted",
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "evidence_id",
        &evidence_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "correlation_id",
        &correlation_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "run_id",
        &run_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "policy_pack_id",
        &policy_pack_id,
        "FSV-POLICY-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "citation_bundle_id",
        &citation_bundle_id,
        "FSV-CITATION-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &manifest,
        "working_hash_after",
        &working_hash_after,
        "FSV-WORKING-HASH-MISMATCH",
        &diagnostics,
        &response,
    )?;
    if manifest.get("rr_present").and_then(Value::as_bool) != Some(true) {
        return Err(VerificationFailure {
            code: "FSV-MISSING-FIELD",
            message: "manifest rr_present is not true".to_string(),
            diagnostics,
            response: Some(response),
        });
    }
    expect_eq(
        &run,
        "request_id",
        &response_to,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &run,
        "correlation_id",
        &correlation_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &run,
        "run_id",
        &run_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &policy,
        "policy_pack_id",
        &policy_pack_id,
        "FSV-POLICY-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &citation,
        "bundle_id",
        &citation_bundle_id,
        "FSV-CITATION-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &working_delta,
        "correlation_id",
        &correlation_id,
        "FSV-ID-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &working_delta,
        "resulting_hash",
        &working_hash_after,
        "FSV-WORKING-HASH-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &terminal,
        "schema_version",
        TERMINAL_BINDING_SCHEMA_VERSION,
        "FSV-TERMINAL-BINDING-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &terminal,
        "outcome",
        "accepted",
        "FSV-TERMINAL-BINDING-MISMATCH",
        &diagnostics,
        &response,
    )?;
    let terminal_created_at = terminal
        .get("created_at")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| VerificationFailure {
            code: "FSV-MISSING-FIELD",
            message: "missing required string field: created_at".to_string(),
            diagnostics: diagnostics.clone(),
            response: Some(response.clone()),
        })?;
    OffsetDateTime::parse(&terminal_created_at, &Rfc3339).map_err(|error| VerificationFailure {
        code: "FSV-TERMINAL-BINDING-MISMATCH",
        message: format!("terminal binding created_at is not RFC3339: {error}"),
        diagnostics: diagnostics.clone(),
        response: Some(response.clone()),
    })?;
    for key in [
        "response_to",
        "correlation_id",
        "run_id",
        "evidence_id",
        "policy_pack_id",
        "citation_bundle_id",
        "working_hash_after",
    ] {
        let expected = match key {
            "response_to" => &response_to,
            "correlation_id" => &correlation_id,
            "run_id" => &run_id,
            "evidence_id" => &evidence_id,
            "policy_pack_id" => &policy_pack_id,
            "citation_bundle_id" => &citation_bundle_id,
            "working_hash_after" => &working_hash_after,
            _ => unreachable!(),
        };
        expect_eq(
            &terminal,
            key,
            expected,
            "FSV-TERMINAL-BINDING-MISMATCH",
            &diagnostics,
            &response,
        )?;
    }

    verify_hashes(&evidence_dir, &hashes, &diagnostics, &response)?;
    let working_json_hash = raw_file_sha256(&cyrune_home.join("working").join("working.json"))
        .map_err(|message| VerificationFailure {
            code: "FSV-WORKING-MISSING",
            message,
            diagnostics: diagnostics.clone(),
            response: Some(response.clone()),
        })?;
    if working_json_hash != working_hash_after {
        return Err(VerificationFailure {
            code: "FSV-WORKING-HASH-MISMATCH",
            message: format!(
                "working/working.json hash mismatch: expected {working_hash_after}, got {working_json_hash}"
            ),
            diagnostics,
            response: Some(response),
        });
    }
    expect_eq(
        &terminal,
        "working_json_hash",
        &working_json_hash,
        "FSV-TERMINAL-BINDING-MISMATCH",
        &diagnostics,
        &response,
    )?;
    let evidence_manifest_hash =
        raw_file_sha256(&evidence_dir.join("manifest.json")).map_err(|message| {
            VerificationFailure {
                code: "FSV-EVIDENCE-HASH-MISMATCH",
                message,
                diagnostics: diagnostics.clone(),
                response: Some(response.clone()),
            }
        })?;
    let evidence_hashes_hash =
        raw_file_sha256(&evidence_dir.join("hashes.json")).map_err(|message| {
            VerificationFailure {
                code: "FSV-EVIDENCE-HASH-MISMATCH",
                message,
                diagnostics: diagnostics.clone(),
                response: Some(response.clone()),
            }
        })?;
    expect_eq(
        &terminal,
        "evidence_manifest_hash",
        &evidence_manifest_hash,
        "FSV-TERMINAL-BINDING-MISMATCH",
        &diagnostics,
        &response,
    )?;
    expect_eq(
        &terminal,
        "evidence_hashes_hash",
        &evidence_hashes_hash,
        "FSV-TERMINAL-BINDING-MISMATCH",
        &diagnostics,
        &response,
    )?;

    Ok(json!({
        "schema_version": SCHEMA_VERSION,
        "verified": true,
        "outcome": "accepted",
        "failure_code": null,
        "failure_message": null,
        "diagnostics": diagnostics,
        "public_first_success_input": PUBLIC_FIRST_SUCCESS_INPUT,
        "run_mode": RUN_MODE,
        "state_root": state_root.display().to_string(),
        "cyrune_home": cyrune_home.display().to_string(),
        "response": response,
        "response_to": response_to,
        "correlation_id": correlation_id,
        "run_id": run_id,
        "evidence_id": evidence_id,
        "policy_pack_id": policy_pack_id,
        "citation_bundle_id": citation_bundle_id,
        "working_hash_after": working_hash_after,
        "evidence_dir": evidence_dir_rel.to_string_lossy().to_string(),
        "terminal_binding_path": terminal_binding_rel.to_string_lossy().to_string(),
        "terminal_binding_schema_version": TERMINAL_BINDING_SCHEMA_VERSION,
        "terminal_binding_created_at": terminal_created_at,
        "evidence_manifest_hash": evidence_manifest_hash,
        "evidence_hashes_hash": evidence_hashes_hash,
        "working_json_hash": working_json_hash,
        "checked_at": checked_at(),
    }))
}

fn emit_failure_report(
    code: &'static str,
    message: impl Into<String>,
    diagnostics: Vec<String>,
    response: Option<Value>,
    state_root: Option<&Path>,
    cyrune_home: Option<&Path>,
) -> Result<i32, String> {
    let message = message.into();
    let response_to = optional_string(response.as_ref(), "response_to");
    let correlation_id = optional_string(response.as_ref(), "correlation_id");
    let run_id = optional_string(response.as_ref(), "run_id");
    let evidence_id = optional_string(response.as_ref(), "evidence_id");
    let policy_pack_id = optional_string(response.as_ref(), "policy_pack_id");
    let citation_bundle_id = optional_string(response.as_ref(), "citation_bundle_id");
    let working_hash_after = optional_string(response.as_ref(), "working_hash_after");
    let evidence_dir = evidence_id
        .as_ref()
        .map(|evidence_id| format!("ledger/evidence/{evidence_id}"));
    let terminal_binding_path = evidence_id
        .as_ref()
        .map(|evidence_id| format!("ledger/terminal-bindings/{evidence_id}.json"));
    let report = json!({
        "schema_version": SCHEMA_VERSION,
        "verified": false,
        "outcome": "rejected",
        "failure_code": code,
        "failure_message": message,
        "diagnostics": diagnostics,
        "response": response,
        "response_to": response_to,
        "correlation_id": correlation_id,
        "run_id": run_id,
        "evidence_id": evidence_id,
        "policy_pack_id": policy_pack_id,
        "citation_bundle_id": citation_bundle_id,
        "working_hash_after": working_hash_after,
        "evidence_dir": evidence_dir,
        "terminal_binding_path": terminal_binding_path,
        "state_root": state_root.map(|path| path.display().to_string()),
        "cyrune_home": cyrune_home.map(|path| path.display().to_string()),
        "checked_at": checked_at(),
    });
    println!("{}", render_json_payload(&report)?);
    eprintln!("{code}");
    Ok(1)
}

fn public_first_success_request() -> Result<RunRequest, String> {
    let request_stamp = OffsetDateTime::now_utc()
        .format(format_description!("[year][month][day]"))
        .map_err(|error| error.to_string())?;
    let now_nanos = OffsetDateTime::now_utc().unix_timestamp_nanos();
    let request_id = RequestId::parse(format!(
        "REQ-{}-{:04}",
        request_stamp,
        (now_nanos % 10_000) as i64
    ))
    .map_err(|error| error.to_string())?;
    let correlation_id = CorrelationId::parse(format!(
        "RUN-{}-{:04}",
        request_stamp,
        ((now_nanos / 1_000) % 10_000) as i64
    ))
    .map_err(|error| error.to_string())?;
    Ok(RunRequest {
        request_id,
        correlation_id,
        run_kind: RunKind::NoLlm,
        user_input: PUBLIC_FIRST_SUCCESS_INPUT.to_string(),
        policy_pack_id: DEFAULT_POLICY_PACK_ID.to_string(),
        binding_id: None,
        requested_capabilities: Vec::new(),
        io_mode: IoMode::Captured,
        adapter_id: None,
        argv: None,
        cwd: None,
        env_overrides: None,
    })
}

fn required_string(
    value: &Value,
    key: &str,
    diagnostics: &[String],
) -> Result<String, VerificationFailure> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| VerificationFailure {
            code: "FSV-MISSING-FIELD",
            message: format!("missing required string field: {key}"),
            diagnostics: diagnostics.to_vec(),
            response: Some(value.clone()),
        })
}

fn expect_eq(
    value: &Value,
    key: &str,
    expected: &str,
    code: &'static str,
    diagnostics: &[String],
    response: &Value,
) -> Result<(), VerificationFailure> {
    let actual = value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| VerificationFailure {
            code: "FSV-MISSING-FIELD",
            message: format!("missing required string field: {key}"),
            diagnostics: diagnostics.to_vec(),
            response: Some(response.clone()),
        })?;
    if actual == expected {
        Ok(())
    } else {
        Err(VerificationFailure {
            code,
            message: format!("{key} mismatch: expected {expected}, got {actual}"),
            diagnostics: diagnostics.to_vec(),
            response: Some(response.clone()),
        })
    }
}

fn read_json(path: &Path, missing_code: &'static str) -> Result<Value, VerificationFailure> {
    let bytes = fs::read(path).map_err(|error| VerificationFailure {
        code: missing_code,
        message: format!("failed to read {}: {error}", path.display()),
        diagnostics: Vec::new(),
        response: None,
    })?;
    serde_json::from_slice(&bytes).map_err(|error| VerificationFailure {
        code: "FSV-MISSING-FIELD",
        message: format!("failed to parse {}: {error}", path.display()),
        diagnostics: Vec::new(),
        response: None,
    })
}

fn verify_hashes(
    evidence_dir: &Path,
    hashes: &Value,
    diagnostics: &[String],
    response: &Value,
) -> Result<(), VerificationFailure> {
    let files = hashes
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| VerificationFailure {
            code: "FSV-MISSING-FIELD",
            message: "hashes.json files object missing".to_string(),
            diagnostics: diagnostics.to_vec(),
            response: Some(response.clone()),
        })?;
    let expected = [
        "manifest.json",
        "run.json",
        "policy.json",
        "citation_bundle.json",
        "rr.json",
        "working_delta.json",
        "stdout.log",
        "stderr.log",
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let actual = files.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(VerificationFailure {
            code: "FSV-EVIDENCE-HASH-MISMATCH",
            message: format!("hashes.json files set mismatch: {actual:?}"),
            diagnostics: diagnostics.to_vec(),
            response: Some(response.clone()),
        });
    }
    for file_name in expected {
        let expected_hash = files
            .get(file_name)
            .and_then(Value::as_str)
            .ok_or_else(|| VerificationFailure {
                code: "FSV-EVIDENCE-HASH-MISMATCH",
                message: format!("hashes.json missing hash for {file_name}"),
                diagnostics: diagnostics.to_vec(),
                response: Some(response.clone()),
            })?;
        let actual_hash = raw_file_sha256(&evidence_dir.join(file_name)).map_err(|message| {
            VerificationFailure {
                code: "FSV-EVIDENCE-MISSING",
                message,
                diagnostics: diagnostics.to_vec(),
                response: Some(response.clone()),
            }
        })?;
        if actual_hash != expected_hash {
            return Err(VerificationFailure {
                code: "FSV-EVIDENCE-HASH-MISMATCH",
                message: format!(
                    "evidence file hash mismatch for {file_name}: expected {expected_hash}, got {actual_hash}"
                ),
                diagnostics: diagnostics.to_vec(),
                response: Some(response.clone()),
            });
        }
    }
    Ok(())
}

fn raw_file_sha256(path: &Path) -> Result<String, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    Ok(format!("sha256:{out}"))
}

fn optional_string(value: Option<&Value>, key: &str) -> Option<String> {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn resolve_absolute(path: &Path) -> Result<PathBuf, String> {
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(_) if path.is_absolute() => Ok(path.to_path_buf()),
        Err(_) => std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|error| error.to_string()),
    }
}

fn checked_at() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .expect("current UTC time must be formattable as RFC3339")
}
