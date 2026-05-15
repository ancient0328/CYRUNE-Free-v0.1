#![forbid(unsafe_code)]

use crate::doctor::run_doctor;
use crate::pack::{default_cyrune_home, default_daemon_binary_path, ensure_terminal_config};
use crate::verify::run_verify;
use crate::view::run_view;
use cyrune_core_contract::{CorrelationId, IoMode, PathLabel, RequestId, RunKind, RunRequest};
use cyrune_daemon::ipc::{
    IPC_VERSION, IpcCommand, IpcStatus, RawRequestEnvelope, ResponseEnvelope, StreamChunkPayload,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

const REQUEST_ID_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]");
const DEFAULT_POLICY_PACK_ID: &str = "cyrune-free-default";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CapturedIpcResponse {
    pub payload: serde_json::Value,
    pub diagnostics: Vec<String>,
}

pub fn run_with_args(args: &[String]) -> Result<i32, String> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage().to_string());
    };
    match command {
        "shell" => run_shell(),
        "run" => run_command(&args[1..]),
        "verify" => run_verify(&args[1..]),
        "view" => run_view(&args[1..]),
        "doctor" => run_doctor(),
        _ => Err(usage().to_string()),
    }
}

pub fn invoke_daemon_single(command: IpcCommand, payload: Value) -> Result<Value, String> {
    let daemon_path = default_daemon_binary_path()?;
    let cyrune_home = default_cyrune_home()?;
    let message_id = next_message_id()?;
    let request = RawRequestEnvelope {
        version: IPC_VERSION.to_string(),
        message_id: message_id.clone(),
        command: command_name(&command).to_string(),
        payload,
    };
    let mut child = Command::new(daemon_path)
        .arg("serve-stdio")
        .env("CYRUNE_HOME", &cyrune_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    write_request_and_close_stdin(&mut child, &request)?;
    let mut stdout = String::new();
    child
        .stdout
        .as_mut()
        .ok_or_else(|| "daemon stdout is unavailable".to_string())?
        .read_to_string(&mut stdout)
        .map_err(|error| error.to_string())?;
    let status = child.wait().map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("daemon exited unsuccessfully: {status}"));
    }
    let response = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| "daemon produced no response".to_string())?;
    let envelope: ResponseEnvelope =
        serde_json::from_str(response).map_err(|error| error.to_string())?;
    match envelope.status {
        IpcStatus::Ok => Ok(envelope.payload),
        IpcStatus::Error => Err(envelope
            .payload
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("daemon error")
            .to_string()),
        other => Err(format!("unexpected daemon response status: {other:?}")),
    }
}

pub(crate) fn invoke_daemon_single_capture_stderr(
    command: IpcCommand,
    payload: Value,
) -> Result<CapturedIpcResponse, String> {
    let daemon_path = default_daemon_binary_path()?;
    let cyrune_home = default_cyrune_home()?;
    let message_id = next_message_id()?;
    let request = RawRequestEnvelope {
        version: IPC_VERSION.to_string(),
        message_id: message_id.clone(),
        command: command_name(&command).to_string(),
        payload,
    };
    let mut child = Command::new(daemon_path)
        .arg("serve-stdio")
        .env("CYRUNE_HOME", &cyrune_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;
    write_request_and_close_stdin(&mut child, &request)?;
    let mut stdout = String::new();
    child
        .stdout
        .as_mut()
        .ok_or_else(|| "daemon stdout is unavailable".to_string())?
        .read_to_string(&mut stdout)
        .map_err(|error| error.to_string())?;
    let mut stderr = String::new();
    child
        .stderr
        .as_mut()
        .ok_or_else(|| "daemon stderr is unavailable".to_string())?
        .read_to_string(&mut stderr)
        .map_err(|error| error.to_string())?;
    let diagnostics = stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let status = child.wait().map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("daemon exited unsuccessfully: {status}"));
    }
    let response = stdout
        .lines()
        .find(|line| !line.trim().is_empty())
        .ok_or_else(|| "daemon produced no response".to_string())?;
    let envelope: ResponseEnvelope =
        serde_json::from_str(response).map_err(|error| error.to_string())?;
    match envelope.status {
        IpcStatus::Ok => Ok(CapturedIpcResponse {
            payload: envelope.payload,
            diagnostics,
        }),
        IpcStatus::Error => Err(envelope
            .payload
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("daemon error")
            .to_string()),
        other => Err(format!("unexpected daemon response status: {other:?}")),
    }
}

pub fn invoke_daemon_stream(
    command: IpcCommand,
    payload: Value,
) -> Result<Vec<StreamChunkPayload>, String> {
    let daemon_path = default_daemon_binary_path()?;
    let cyrune_home = default_cyrune_home()?;
    let message_id = next_message_id()?;
    let request = RawRequestEnvelope {
        version: IPC_VERSION.to_string(),
        message_id,
        command: command_name(&command).to_string(),
        payload,
    };
    let mut child = Command::new(daemon_path)
        .arg("serve-stdio")
        .env("CYRUNE_HOME", &cyrune_home)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| error.to_string())?;
    write_request_and_close_stdin(&mut child, &request)?;
    let mut stdout = String::new();
    child
        .stdout
        .as_mut()
        .ok_or_else(|| "daemon stdout is unavailable".to_string())?
        .read_to_string(&mut stdout)
        .map_err(|error| error.to_string())?;
    let status = child.wait().map_err(|error| error.to_string())?;
    if !status.success() {
        return Err(format!("daemon exited unsuccessfully: {status}"));
    }
    let mut chunks = Vec::new();
    for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
        let envelope: ResponseEnvelope =
            serde_json::from_str(line).map_err(|error| error.to_string())?;
        match envelope.status {
            IpcStatus::Stream | IpcStatus::End => {
                chunks.push(
                    serde_json::from_value::<StreamChunkPayload>(envelope.payload)
                        .map_err(|error| error.to_string())?,
                );
            }
            IpcStatus::Error => {
                return Err(envelope
                    .payload
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("daemon error")
                    .to_string());
            }
            IpcStatus::Ok => {}
        }
    }
    Ok(chunks)
}

pub fn render_json_payload(payload: &Value) -> Result<String, String> {
    serde_json::to_string_pretty(payload).map_err(|error| error.to_string())
}

fn run_shell() -> Result<i32, String> {
    let cyrune_home = default_cyrune_home()?;
    ensure_terminal_config(&cyrune_home)?;
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let status = Command::new(shell)
        .env("CYRUNE_HOME", &cyrune_home)
        .status()
        .map_err(|error| error.to_string())?;
    Ok(status.code().unwrap_or(1))
}

fn run_command(args: &[String]) -> Result<i32, String> {
    let request = build_run_request(args)?;
    let payload = build_run_payload(&request)?;
    let response = invoke_daemon_single(IpcCommand::Run, payload)?;
    println!("{}", render_json_payload(&response)?);
    Ok(0)
}

fn build_run_request(args: &[String]) -> Result<RunRequest, String> {
    if args.is_empty() {
        return Err(
            "usage: cyr run (--no-llm | --adapter <id>) --input <text> [--binding <id>] [options] [-- <argv...>]"
                .to_string(),
        );
    }
    let mut run_kind = None;
    let mut adapter_id = None;
    let mut user_input = None;
    let mut policy_pack_id = DEFAULT_POLICY_PACK_ID.to_string();
    let mut binding_id = None;
    let mut requested_capabilities = Vec::new();
    let mut io_mode = IoMode::Captured;
    let mut cwd = None;
    let mut env_overrides = BTreeMap::new();
    let mut argv = Vec::new();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--" => {
                argv.extend_from_slice(&args[index + 1..]);
                break;
            }
            "--no-llm" => {
                run_kind = Some(RunKind::NoLlm);
                index += 1;
            }
            "--adapter" => {
                let value = args.get(index + 1).ok_or_else(|| {
                    "--adapter requires <approved-execution-adapter-id>".to_string()
                })?;
                run_kind = Some(RunKind::ExecutionAdapter);
                adapter_id = Some(value.clone());
                index += 2;
            }
            "--input" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--input requires <text>".to_string())?;
                user_input = Some(value.clone());
                index += 2;
            }
            "--policy-pack" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--policy-pack requires <id>".to_string())?;
                policy_pack_id = value.clone();
                index += 2;
            }
            "--binding" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--binding requires <id>".to_string())?;
                binding_id = Some(value.clone());
                index += 2;
            }
            "--cap" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--cap requires <capability>".to_string())?;
                requested_capabilities.push(value.clone());
                index += 2;
            }
            "--cwd" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--cwd requires <path>".to_string())?;
                cwd = Some(PathLabel::parse(value.clone()).map_err(|error| error.to_string())?);
                index += 2;
            }
            "--env" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--env requires KEY=VALUE".to_string())?;
                let (key, item_value) = value
                    .split_once('=')
                    .ok_or_else(|| "--env requires KEY=VALUE".to_string())?;
                env_overrides.insert(key.to_string(), item_value.to_string());
                index += 2;
            }
            "--io-mode" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--io-mode requires captured|quiet".to_string())?;
                io_mode = match value.as_str() {
                    "captured" => IoMode::Captured,
                    "quiet" => IoMode::Quiet,
                    _ => return Err("--io-mode requires captured|quiet".to_string()),
                };
                index += 2;
            }
            other => return Err(format!("unknown run option: {other}")),
        }
    }

    let run_kind = run_kind.ok_or_else(|| "--no-llm or --adapter is required".to_string())?;
    let user_input = user_input.ok_or_else(|| "--input is required".to_string())?;
    let request_id = RequestId::parse(format!(
        "REQ-{}-{:04}",
        OffsetDateTime::now_utc()
            .format(REQUEST_ID_FORMAT)
            .map_err(|error| error.to_string())?,
        (OffsetDateTime::now_utc().unix_timestamp_nanos() % 10_000) as i64
    ))
    .map_err(|error| error.to_string())?;
    let correlation_id = CorrelationId::parse(format!(
        "RUN-{}-{:04}",
        OffsetDateTime::now_utc()
            .format(REQUEST_ID_FORMAT)
            .map_err(|error| error.to_string())?,
        ((OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000) % 10_000) as i64
    ))
    .map_err(|error| error.to_string())?;

    Ok(RunRequest {
        request_id,
        correlation_id,
        run_kind,
        user_input,
        policy_pack_id,
        binding_id,
        requested_capabilities,
        io_mode,
        adapter_id,
        argv: if argv.is_empty() { None } else { Some(argv) },
        cwd,
        env_overrides: if env_overrides.is_empty() {
            None
        } else {
            Some(env_overrides)
        },
    })
}

pub(crate) fn build_run_payload(request: &RunRequest) -> Result<Value, String> {
    serde_json::to_value(request).map_err(|error| error.to_string())
}

fn command_name(command: &IpcCommand) -> &'static str {
    match command {
        IpcCommand::Run => "Run",
        IpcCommand::Cancel => "Cancel",
        IpcCommand::Tail => "Tail",
        IpcCommand::GetEvidence => "GetEvidence",
        IpcCommand::ListEvidence => "ListEvidence",
        IpcCommand::GetWorking => "GetWorking",
        IpcCommand::ExplainPolicy => "ExplainPolicy",
        IpcCommand::Health => "Health",
    }
}

fn write_request_and_close_stdin(
    child: &mut Child,
    request: &RawRequestEnvelope,
) -> Result<(), String> {
    let request_line = serde_json::to_string(request).map_err(|error| error.to_string())? + "\n";
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "daemon stdin is unavailable".to_string())?;
    stdin
        .write_all(request_line.as_bytes())
        .map_err(|error| error.to_string())?;
    stdin.flush().map_err(|error| error.to_string())?;
    drop(stdin);
    Ok(())
}

fn next_message_id() -> Result<String, String> {
    Ok(format!(
        "MSG-{}",
        OffsetDateTime::now_utc().unix_timestamp_nanos()
    ))
}

fn usage() -> &'static str {
    "usage: cyr <shell|run|verify|view|doctor> ..."
}

#[cfg(test)]
mod tests {
    use super::{build_run_payload, build_run_request};
    use cyrune_core_contract::RunKind;

    #[test]
    fn run_payload_is_raw_run_request() {
        let request = build_run_request(&[
            "--no-llm".to_string(),
            "--input".to_string(),
            "Summarize this.".to_string(),
        ])
        .unwrap();
        assert_eq!(request.run_kind, RunKind::NoLlm);
        let payload = build_run_payload(&request).unwrap();
        let reparsed: cyrune_core_contract::RunRequest =
            serde_json::from_value(payload.clone()).unwrap();
        assert_eq!(reparsed, request);
        assert_eq!(payload, serde_json::to_value(&request).unwrap());
        assert_eq!(request.policy_pack_id, "cyrune-free-default");
        assert_eq!(request.binding_id, None);
    }

    #[test]
    fn run_request_accepts_explicit_binding_selection() {
        let request = build_run_request(&[
            "--no-llm".to_string(),
            "--input".to_string(),
            "Summarize this.".to_string(),
            "--binding".to_string(),
            "cyrune-free-shipping.v0.1".to_string(),
        ])
        .unwrap();
        assert_eq!(request.run_kind, RunKind::NoLlm);
        assert_eq!(
            request.binding_id.as_deref(),
            Some("cyrune-free-shipping.v0.1")
        );
    }
}
