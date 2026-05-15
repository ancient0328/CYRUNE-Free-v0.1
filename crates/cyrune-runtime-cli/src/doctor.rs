#![forbid(unsafe_code)]

use crate::cli::{invoke_daemon_single, render_json_payload};
use cyrune_daemon::ipc::IpcCommand;

pub fn run_doctor() -> Result<i32, String> {
    let payload = serde_json::json!({});
    let response = invoke_daemon_single(IpcCommand::Health, payload)?;
    println!("{}", render_json_payload(&response)?);
    Ok(0)
}
