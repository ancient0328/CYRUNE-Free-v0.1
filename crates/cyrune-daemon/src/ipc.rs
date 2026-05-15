#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

pub const IPC_VERSION: &str = "cyrune.free.ipc.v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IpcCommand {
    Run,
    Cancel,
    Tail,
    GetEvidence,
    ListEvidence,
    GetWorking,
    ExplainPolicy,
    Health,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcStatus {
    Ok,
    Error,
    Stream,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamKind {
    Stdout,
    Stderr,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawRequestEnvelope {
    pub version: String,
    pub message_id: String,
    pub command: String,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestEnvelope {
    pub message_id: String,
    pub command: IpcCommand,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub version: String,
    pub message_id: String,
    pub response_to: String,
    pub status: IpcStatus,
    pub payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamChunkPayload {
    pub stream_kind: StreamKind,
    pub sequence: u64,
    pub eof: bool,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(String),
}

impl RequestEnvelope {
    pub fn from_line(line: &str) -> Result<Self, IpcError> {
        let raw: RawRequestEnvelope = serde_json::from_str(line)?;
        if raw.version != IPC_VERSION {
            return Err(IpcError::Invalid(format!(
                "unknown ipc version: {}",
                raw.version
            )));
        }
        if raw.message_id.trim().is_empty() {
            return Err(IpcError::Invalid(
                "message_id must not be empty".to_string(),
            ));
        }
        let command = parse_command(&raw.command)?;
        Ok(Self {
            message_id: raw.message_id,
            command,
            payload: raw.payload,
        })
    }
}

impl ResponseEnvelope {
    #[must_use]
    pub fn ok(
        message_id: impl Into<String>,
        response_to: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            message_id: message_id.into(),
            response_to: response_to.into(),
            status: IpcStatus::Ok,
            payload,
        }
    }

    #[must_use]
    pub fn error(
        message_id: impl Into<String>,
        response_to: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            message_id: message_id.into(),
            response_to: response_to.into(),
            status: IpcStatus::Error,
            payload: serde_json::to_value(ErrorPayload {
                message: message.into(),
            })
            .expect("error payload must serialize"),
        }
    }

    #[must_use]
    pub fn stream(
        message_id: impl Into<String>,
        response_to: impl Into<String>,
        payload: &StreamChunkPayload,
    ) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            message_id: message_id.into(),
            response_to: response_to.into(),
            status: IpcStatus::Stream,
            payload: serde_json::to_value(payload).expect("stream payload must serialize"),
        }
    }

    #[must_use]
    pub fn end(
        message_id: impl Into<String>,
        response_to: impl Into<String>,
        payload: &StreamChunkPayload,
    ) -> Self {
        Self {
            version: IPC_VERSION.to_string(),
            message_id: message_id.into(),
            response_to: response_to.into(),
            status: IpcStatus::End,
            payload: serde_json::to_value(payload).expect("end payload must serialize"),
        }
    }

    pub fn to_line(&self) -> Result<String, IpcError> {
        let mut line = serde_json::to_string(self)?;
        line.push('\n');
        Ok(line)
    }
}

fn parse_command(value: &str) -> Result<IpcCommand, IpcError> {
    match value {
        "Run" => Ok(IpcCommand::Run),
        "Cancel" => Ok(IpcCommand::Cancel),
        "Tail" => Ok(IpcCommand::Tail),
        "GetEvidence" => Ok(IpcCommand::GetEvidence),
        "ListEvidence" => Ok(IpcCommand::ListEvidence),
        "GetWorking" => Ok(IpcCommand::GetWorking),
        "ExplainPolicy" => Ok(IpcCommand::ExplainPolicy),
        "Health" => Ok(IpcCommand::Health),
        unknown => Err(IpcError::Invalid(format!("unknown ipc command: {unknown}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{IPC_VERSION, IpcError, IpcStatus, RawRequestEnvelope, RequestEnvelope};
    use serde_json::json;

    #[test]
    fn unknown_command_is_rejected() {
        let line = serde_json::to_string(&RawRequestEnvelope {
            version: IPC_VERSION.to_string(),
            message_id: "MSG-1".to_string(),
            command: "Unknown".to_string(),
            payload: json!({}),
        })
        .unwrap();
        let error = RequestEnvelope::from_line(&line).unwrap_err();
        assert!(
            matches!(error, IpcError::Invalid(message) if message.contains("unknown ipc command"))
        );
    }

    #[test]
    fn response_line_round_trips() {
        let response = super::ResponseEnvelope::ok("MSG-2", "MSG-1", json!({"ok": true}));
        let line = response.to_line().unwrap();
        let parsed: super::ResponseEnvelope = serde_json::from_str(line.trim_end()).unwrap();
        assert_eq!(parsed.status, IpcStatus::Ok);
    }
}
