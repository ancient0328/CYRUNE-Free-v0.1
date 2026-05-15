#![forbid(unsafe_code)]

use cyrune_runtime_cli::pack::launch_packaged_terminal_with_distribution_root_override;
use serde_json::json;
use std::env;
use std::path::{Path, PathBuf};

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
        "launch" => run_launch(&args[1..]),
        _ => Err(usage().to_string()),
    }
}

fn run_launch(args: &[String]) -> Result<i32, String> {
    let mut terminal_binary = None;
    let mut cyrune_home = None;
    let mut distribution_root = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--terminal-binary" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--terminal-binary requires <path>".to_string())?;
                terminal_binary = Some(PathBuf::from(value));
                index += 2;
            }
            "--cyrune-home" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--cyrune-home requires <path>".to_string())?;
                cyrune_home = Some(PathBuf::from(value));
                index += 2;
            }
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

    let terminal_binary =
        terminal_binary.ok_or_else(|| "--terminal-binary requires <path>".to_string())?;
    let cyrune_home = cyrune_home.ok_or_else(|| "--cyrune-home requires <path>".to_string())?;
    let distribution_root = distribution_root.as_deref();

    match launch_packaged_terminal_with_distribution_root_override(
        terminal_binary.as_path(),
        cyrune_home.as_path(),
        distribution_root,
    ) {
        Ok(execution) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "launched",
                    "exit_code": execution.exit_code,
                    "plan": {
                        "cyrune_home": path_display(&execution.plan.cyrune_home),
                        "distribution_root": path_display(&execution.plan.distribution_root),
                        "bundle_root": path_display(&execution.plan.bundle_root),
                        "terminal_config_path": path_display(&execution.plan.terminal_config_path),
                        "runtime_pid_path": path_display(&execution.plan.runtime_pid_path),
                        "distribution_root_override": execution
                            .plan
                            .distribution_root_override
                            .as_ref()
                            .map(|path| path_display(path)),
                    },
                    "invocation": {
                        "program": path_display(&execution.invocation.program),
                        "args": execution.invocation.args,
                        "env": execution.invocation.env,
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

fn path_display(path: &Path) -> String {
    path.display().to_string()
}

fn usage() -> &'static str {
    "usage: d6-proof-driver launch --terminal-binary <path> --cyrune-home <path> [--distribution-root <path>]"
}
