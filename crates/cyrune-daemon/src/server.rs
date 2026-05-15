#![forbid(unsafe_code)]

use crate::command::CommandContext;
use crate::ipc::{RequestEnvelope, ResponseEnvelope};
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Command(#[from] crate::command::CommandError),
    #[error(transparent)]
    Ipc(#[from] crate::ipc::IpcError),
}

pub fn serve_stdio_default() -> Result<(), ServerError> {
    let context = CommandContext::from_environment()?;
    let pid_path = context.cyrune_home().join("runtime").join("daemon.pid");
    fs::write(&pid_path, std::process::id().to_string())?;
    let result = serve_stdio_once(&context, io::stdin().lock(), &mut io::stdout());
    let _ = fs::remove_file(pid_path);
    result
}

pub fn serve_stdio<R: io::Read, W: Write>(
    context: &CommandContext,
    reader: R,
    writer: &mut W,
) -> Result<(), ServerError> {
    let buffered = BufReader::new(reader);
    for line in buffered.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        handle_request_line(context, &line, writer)?;
    }
    Ok(())
}

fn serve_stdio_once<R: io::Read, W: Write>(
    context: &CommandContext,
    reader: R,
    writer: &mut W,
) -> Result<(), ServerError> {
    let buffered = BufReader::new(reader);
    for line in buffered.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        handle_request_line(context, &line, writer)?;
        break;
    }
    Ok(())
}

fn handle_request_line<W: Write>(
    context: &CommandContext,
    line: &str,
    writer: &mut W,
) -> Result<(), ServerError> {
    let request = match RequestEnvelope::from_line(line) {
        Ok(request) => request,
        Err(error) => {
            let response =
                ResponseEnvelope::error("MSG-ERR-0001", "MSG-UNKNOWN", error.to_string());
            writer.write_all(response.to_line()?.as_bytes())?;
            writer.flush()?;
            return Ok(());
        }
    };
    let response_to = request.message_id.clone();
    let result = match context.execute(request.command, request.payload) {
        Ok(result) => result,
        Err(crate::command::CommandError::Public(message)) => {
            let response =
                ResponseEnvelope::error(format!("{response_to}-R01"), &response_to, message);
            writer.write_all(response.to_line()?.as_bytes())?;
            writer.flush()?;
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    match result {
        crate::command::CommandResult::Single(payload) => {
            let response =
                ResponseEnvelope::ok(format!("{response_to}-R01"), &response_to, payload);
            writer.write_all(response.to_line()?.as_bytes())?;
        }
        crate::command::CommandResult::Stream(chunks) => {
            for chunk in &chunks[..chunks.len().saturating_sub(1)] {
                let response = ResponseEnvelope::stream(
                    format!("{response_to}-S{:02}", chunk.sequence),
                    &response_to,
                    chunk,
                );
                writer.write_all(response.to_line()?.as_bytes())?;
            }
            if let Some(last) = chunks.last() {
                let response = ResponseEnvelope::end(
                    format!("{response_to}-E{:02}", last.sequence),
                    &response_to,
                    last,
                );
                writer.write_all(response.to_line()?.as_bytes())?;
            }
        }
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{serve_stdio, serve_stdio_once};
    use crate::command::{
        CommandContext, ExplainPolicyPayload, default_resolver_inputs, ensure_home_layout,
    };
    use crate::ipc::{IPC_VERSION, IpcStatus, RawRequestEnvelope, ResponseEnvelope};
    use serde_json::json;
    use tempfile::tempdir;

    #[test]
    fn unknown_ipc_command_is_rejected() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            default_resolver_inputs(temp.path()).unwrap(),
        );
        let request = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-1".to_string(),
            command: "Unknown".to_string(),
            payload: json!({}),
        })
        .unwrap();
        let mut output = Vec::new();
        serve_stdio(&context, format!("{request}\n").as_bytes(), &mut output).unwrap();
        let response: ResponseEnvelope =
            serde_json::from_slice(&output[..output.len() - 1]).unwrap();
        assert_eq!(response.status, crate::ipc::IpcStatus::Error);
    }

    #[test]
    fn serve_stdio_default_transport_processes_single_request_and_exits() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            default_resolver_inputs(temp.path()).unwrap(),
        );
        let first = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-1".to_string(),
            command: "Health".to_string(),
            payload: json!({}),
        })
        .unwrap();
        let second = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-2".to_string(),
            command: "Health".to_string(),
            payload: json!({}),
        })
        .unwrap();
        let mut output = Vec::new();
        serve_stdio_once(
            &context,
            format!("{first}\n{second}\n").as_bytes(),
            &mut output,
        )
        .unwrap();
        let rendered = String::from_utf8(output).unwrap();
        let lines: Vec<&str> = rendered.lines().collect();
        assert_eq!(lines.len(), 1);
        let response: ResponseEnvelope = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(response.response_to, "MSG-1");
    }

    #[test]
    fn serve_stdio_returns_error_response_for_unresolved_explain_policy() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            default_resolver_inputs(temp.path()).unwrap(),
        );
        let request = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-3".to_string(),
            command: "ExplainPolicy".to_string(),
            payload: serde_json::to_value(ExplainPolicyPayload {
                policy_pack: Some("missing-pack".to_string()),
                last_denial_id: None,
            })
            .unwrap(),
        })
        .unwrap();
        let mut output = Vec::new();
        serve_stdio_once(&context, format!("{request}\n").as_bytes(), &mut output).unwrap();
        let response: ResponseEnvelope =
            serde_json::from_slice(&output[..output.len() - 1]).unwrap();

        assert_eq!(response.status, IpcStatus::Error);
        let message = response
            .payload
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap();
        assert_eq!(message, "requested policy pack is unresolved: missing-pack");
        assert!(!message.contains("policy exact match not found"));
    }

    #[test]
    fn serve_stdio_propagates_non_public_command_error() {
        let temp = tempdir().unwrap();
        ensure_home_layout(temp.path()).unwrap();
        let context = CommandContext::from_parts(
            temp.path().to_path_buf(),
            default_resolver_inputs(temp.path()).unwrap(),
        );
        let request = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-4".to_string(),
            command: "ExplainPolicy".to_string(),
            payload: json!(1),
        })
        .unwrap();
        let mut output = Vec::new();
        let result = serve_stdio_once(&context, format!("{request}\n").as_bytes(), &mut output);

        assert!(result.is_err());
        assert!(output.is_empty());
    }
}
