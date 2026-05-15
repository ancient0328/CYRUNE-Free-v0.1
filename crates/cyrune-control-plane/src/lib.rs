#![forbid(unsafe_code)]

pub mod citation;
pub mod execution_registry;
pub mod execution_result;
pub mod ledger;
pub mod memory;
pub mod policy;
pub mod request;
pub mod resolved_turn_context;
pub mod resolver;
pub mod retrieval;
pub mod sandbox;
pub mod turn;
pub mod working;

use cyrune_core_contract::crate_identity;

pub const CRATE_IDENTITY: &str = "cyrune-control-plane";

#[must_use]
pub fn control_plane_identity() -> (&'static str, &'static str) {
    (CRATE_IDENTITY, crate_identity())
}

#[cfg(test)]
mod tests {
    use super::control_plane_identity;

    #[test]
    fn control_plane_depends_on_core_contract() {
        assert_eq!(
            control_plane_identity(),
            ("cyrune-control-plane", "cyrune-core-contract")
        );
    }
}
