#![forbid(unsafe_code)]

use crate::cli::{invoke_daemon_single, invoke_daemon_stream, render_json_payload};
use cyrune_core_contract::CorrelationId;
use cyrune_daemon::command::{
    ExplainPolicyPayload, GetEvidencePayload, ListEvidencePayload, TailPayload,
};
use cyrune_daemon::ipc::IpcCommand;
use std::thread;
use std::time::Duration;

const DEFAULT_WORKING_FOLLOW_INTERVAL_MS: u64 = 1_000;
const MIN_WORKING_FOLLOW_INTERVAL_MS: u64 = 100;
const MAX_WORKING_FOLLOW_INTERVAL_MS: u64 = 60_000;

#[derive(Debug, Clone, PartialEq, Eq)]
enum WorkingViewMode {
    Snapshot,
    Follow {
        interval_ms: u64,
        max_updates: Option<u64>,
    },
}

pub fn run_view(args: &[String]) -> Result<i32, String> {
    let Some(kind) = args.first().map(String::as_str) else {
        return Err("usage: cyr view <evidence|working|policy> [options]".to_string());
    };
    match kind {
        "evidence" => run_view_evidence(&args[1..]),
        "working" => run_view_working(&args[1..]),
        "policy" => run_view_policy(&args[1..]),
        other => Err(format!("unknown view target: {other}")),
    }
}

fn run_view_evidence(args: &[String]) -> Result<i32, String> {
    if let Some(index) = args.iter().position(|value| value == "--follow") {
        let correlation = args
            .get(index + 1)
            .ok_or_else(|| "--follow requires <correlation_id>".to_string())?;
        let correlation_id =
            CorrelationId::parse(correlation.clone()).map_err(|error| error.to_string())?;
        let chunks = invoke_daemon_stream(
            IpcCommand::Tail,
            serde_json::to_value(TailPayload { correlation_id })
                .map_err(|error| error.to_string())?,
        )?;
        for chunk in chunks {
            println!("{}", chunk.data.trim_end());
        }
        return Ok(0);
    }
    if let Some(index) = args.iter().position(|value| value == "--evidence-id") {
        let evidence_id = args
            .get(index + 1)
            .ok_or_else(|| "--evidence-id requires <evidence_id>".to_string())?;
        let response = invoke_daemon_single(
            IpcCommand::GetEvidence,
            serde_json::to_value(GetEvidencePayload {
                evidence_id: evidence_id.clone(),
            })
            .map_err(|error| error.to_string())?,
        )?;
        println!("{}", render_json_payload(&response)?);
        return Ok(0);
    }
    let response = invoke_daemon_single(
        IpcCommand::ListEvidence,
        serde_json::to_value(ListEvidencePayload {
            limit: Some(10),
            cursor: None,
        })
        .map_err(|error| error.to_string())?,
    )?;
    println!("{}", render_json_payload(&response)?);
    Ok(0)
}

fn run_view_working(args: &[String]) -> Result<i32, String> {
    match parse_working_view_mode(args)? {
        WorkingViewMode::Snapshot => {
            let snapshot = fetch_working_snapshot_pretty()?;
            println!("{snapshot}");
            Ok(0)
        }
        WorkingViewMode::Follow {
            interval_ms,
            max_updates,
        } => {
            let mut emitted_updates = 0_u64;
            let mut last_emitted: Option<String> = None;
            let snapshot = fetch_working_snapshot_pretty()?;
            if let Some(emission) = next_follow_emission(None, &snapshot) {
                print!("{emission}");
                emitted_updates += 1;
                last_emitted = Some(snapshot);
                if max_updates.is_some_and(|limit| emitted_updates >= limit) {
                    return Ok(0);
                }
            }
            loop {
                thread::sleep(Duration::from_millis(interval_ms));
                let snapshot = fetch_working_snapshot_pretty()?;
                if let Some(emission) = next_follow_emission(last_emitted.as_deref(), &snapshot) {
                    print!("{emission}");
                    emitted_updates += 1;
                    last_emitted = Some(snapshot);
                    if max_updates.is_some_and(|limit| emitted_updates >= limit) {
                        return Ok(0);
                    }
                }
            }
        }
    }
}

fn parse_working_view_mode(args: &[String]) -> Result<WorkingViewMode, String> {
    let mut follow = false;
    let mut interval_ms: Option<u64> = None;
    let mut max_updates: Option<u64> = None;
    let mut index = 0;

    while let Some(arg) = args.get(index).map(String::as_str) {
        match arg {
            "--follow" => {
                if follow {
                    return Err("duplicate option: --follow".to_string());
                }
                follow = true;
                index += 1;
            }
            "--interval-ms" => {
                if interval_ms.is_some() {
                    return Err("duplicate option: --interval-ms".to_string());
                }
                if !follow {
                    return Err("--interval-ms requires --follow".to_string());
                }
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--interval-ms requires <milliseconds>".to_string())?;
                interval_ms = Some(parse_follow_interval_ms(raw)?);
                index += 2;
            }
            "--max-updates" => {
                if max_updates.is_some() {
                    return Err("duplicate option: --max-updates".to_string());
                }
                if !follow {
                    return Err("--max-updates requires --follow".to_string());
                }
                let raw = args
                    .get(index + 1)
                    .ok_or_else(|| "--max-updates requires <count>".to_string())?;
                max_updates = Some(parse_follow_max_updates(raw)?);
                index += 2;
            }
            other => {
                return Err(format!("unknown working view option: {other}"));
            }
        }
    }
    if !follow {
        return Ok(WorkingViewMode::Snapshot);
    }

    Ok(WorkingViewMode::Follow {
        interval_ms: interval_ms.unwrap_or(DEFAULT_WORKING_FOLLOW_INTERVAL_MS),
        max_updates,
    })
}

fn parse_follow_interval_ms(raw: &str) -> Result<u64, String> {
    let value = raw
        .parse::<u64>()
        .map_err(|_| "--interval-ms must be an integer between 100 and 60000".to_string())?;
    if !(MIN_WORKING_FOLLOW_INTERVAL_MS..=MAX_WORKING_FOLLOW_INTERVAL_MS).contains(&value) {
        return Err("--interval-ms must be an integer between 100 and 60000".to_string());
    }
    Ok(value)
}

fn parse_follow_max_updates(raw: &str) -> Result<u64, String> {
    let value = raw
        .parse::<u64>()
        .map_err(|_| "--max-updates must be an integer greater than or equal to 1".to_string())?;
    if value == 0 {
        return Err("--max-updates must be an integer greater than or equal to 1".to_string());
    }
    Ok(value)
}

fn fetch_working_snapshot_pretty() -> Result<String, String> {
    let response = invoke_daemon_single(IpcCommand::GetWorking, serde_json::json!({}))?;
    render_json_payload(&response)
}

fn next_follow_emission(last_emitted: Option<&str>, current: &str) -> Option<String> {
    match last_emitted {
        None => Some(format!("{current}\n")),
        Some(previous) if previous == current => None,
        Some(_) => Some(format!("---\n{current}\n")),
    }
}

fn run_view_policy(args: &[String]) -> Result<i32, String> {
    let pack = if let Some(index) = args.iter().position(|value| value == "--pack") {
        Some(
            args.get(index + 1)
                .ok_or_else(|| "--pack requires <policy_pack>".to_string())?
                .clone(),
        )
    } else {
        None
    };
    let response = invoke_daemon_single(
        IpcCommand::ExplainPolicy,
        serde_json::to_value(ExplainPolicyPayload {
            policy_pack: pack,
            last_denial_id: None,
        })
        .map_err(|error| error.to_string())?,
    )?;
    println!("{}", render_json_payload(&response)?);
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_WORKING_FOLLOW_INTERVAL_MS, WorkingViewMode, next_follow_emission,
        parse_working_view_mode,
    };

    #[test]
    fn working_view_mode_defaults_to_snapshot() {
        let args: Vec<String> = vec![];

        let mode = parse_working_view_mode(&args).expect("working mode should parse");

        assert_eq!(mode, WorkingViewMode::Snapshot);
    }

    #[test]
    fn working_view_mode_accepts_follow_defaults() {
        let args = vec!["--follow".to_string()];

        let mode = parse_working_view_mode(&args).expect("working mode should parse");

        assert_eq!(
            mode,
            WorkingViewMode::Follow {
                interval_ms: DEFAULT_WORKING_FOLLOW_INTERVAL_MS,
                max_updates: None,
            }
        );
    }

    #[test]
    fn working_view_mode_rejects_missing_interval_value() {
        let args = vec!["--follow".to_string(), "--interval-ms".to_string()];

        let error = parse_working_view_mode(&args).expect_err("working mode should fail");

        assert_eq!(error, "--interval-ms requires <milliseconds>");
    }

    #[test]
    fn working_view_mode_rejects_interval_below_minimum() {
        let args = vec![
            "--follow".to_string(),
            "--interval-ms".to_string(),
            "99".to_string(),
        ];

        let error = parse_working_view_mode(&args).expect_err("working mode should fail");

        assert_eq!(
            error,
            "--interval-ms must be an integer between 100 and 60000"
        );
    }

    #[test]
    fn working_view_mode_rejects_max_updates_below_one() {
        let args = vec![
            "--follow".to_string(),
            "--max-updates".to_string(),
            "0".to_string(),
        ];

        let error = parse_working_view_mode(&args).expect_err("working mode should fail");

        assert_eq!(
            error,
            "--max-updates must be an integer greater than or equal to 1"
        );
    }

    #[test]
    fn next_follow_emission_formats_first_snapshot() {
        let emission = next_follow_emission(None, "{\"version\":1}");

        assert_eq!(emission, Some("{\"version\":1}\n".to_string()));
    }

    #[test]
    fn next_follow_emission_suppresses_identical_snapshot() {
        let emission = next_follow_emission(Some("{\"version\":1}"), "{\"version\":1}");

        assert_eq!(emission, None);
    }

    #[test]
    fn next_follow_emission_formats_changed_snapshot_after_separator() {
        let emission = next_follow_emission(Some("{\"version\":1}"), "{\"version\":2}");

        assert_eq!(emission, Some("---\n{\"version\":2}\n".to_string()));
    }
}
