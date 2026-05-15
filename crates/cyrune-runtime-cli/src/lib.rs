#![forbid(unsafe_code)]

pub mod cli;
pub mod doctor;
pub mod pack;
pub mod verify;
pub mod view;

pub const CRATE_IDENTITY: &str = "cyrune-runtime-cli";

#[must_use]
pub fn run() -> i32 {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    cli::run_with_args(&args).unwrap_or_else(|error| {
        eprintln!("{error}");
        1
    })
}

#[cfg(test)]
mod tests {
    use super::CRATE_IDENTITY;

    #[test]
    fn cli_identity_and_exit_code_are_stable() {
        assert_eq!(CRATE_IDENTITY, "cyrune-runtime-cli");
    }
}
