#![forbid(unsafe_code)]

pub mod command;
pub mod ipc;
pub mod server;

pub const CRATE_IDENTITY: &str = "cyrune-daemon";

#[must_use]
pub fn run() -> i32 {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    run_with_args(&args)
}

#[must_use]
pub fn run_with_args(args: &[String]) -> i32 {
    match args {
        [] => server::serve_stdio_default().map_or_else(report_error, |_| 0),
        [command] if command == "serve-stdio" => {
            server::serve_stdio_default().map_or_else(report_error, |_| 0)
        }
        _ => {
            eprintln!("usage: cyrune-daemon [serve-stdio]");
            2
        }
    }
}

fn report_error(error: server::ServerError) -> i32 {
    eprintln!("{error}");
    1
}

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, run_with_args};

    #[test]
    fn daemon_identity_and_exit_code_are_stable() {
        assert_eq!(CRATE_IDENTITY, "cyrune-daemon");
        assert_eq!(run_with_args(&["--invalid".to_string()]), 2);
    }
}
