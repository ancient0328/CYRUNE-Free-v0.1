# ENGINEERING_SPEC

This document explains the engineering-facing structure and execution contract of the CYRUNE Free v0.1 public repository.

It does not replace the public authority surface. Start with the root `README.md`, `docs/GETTING_STARTED.md`, and `docs/CYRUNE_Free_Public_Index.md`.

Japanese companion material is available at `docs/ja/ENGINEERING_SPEC.md`.

## 1. Scope

This document covers the repository root contents and the package-level implementation contract for:

- `README.md`
- `docs/`
- `scripts/`
- repository-root source tree

It is meant to answer:

1. what physical surfaces are published,
2. what root each public script uses,
3. what state is generated under `target/public-run/`,
4. what changes can break public-run behavior,
5. what predicates should be checked during maintenance,
6. what runtime / product scope this public beta does not claim.

This document does not redefine current public truth, task roadmaps, native distribution, OS-level sandbox enforcement, completed classification / MAC, upper-tier features, signing, notarization, or installer distribution.

## 2. Public Surfaces

The public repository has four surfaces:

1. discovery / authority surface:
   `README.md`, `docs/GETTING_STARTED.md`, `docs/FIRST_SUCCESS_EXPECTED.md`, `docs/TROUBLESHOOTING.md`, `docs/CYRUNE_Free_Public_Index.md`
2. current public docs:
   `docs/current/` plus the guide and engineering docs under `docs/`
3. separated reference shelves:
   `docs/historical/`, `docs/deferred/`, `docs/ja/`
4. runnable source tree:
   repository root

## 3. Repository Publication Model

GitHub publication uses `main` as the latest public repository surface and immutable SemVer tags as snapshots.

For CYRUNE Free v0.1 public beta:

- `main` points to the latest public surface.
- `v0.1.0` is the published immutable snapshot tag for the Free v0.1 public alpha.
- `v0.1.1-beta.1` is the first public beta release-contract tag.
- Existing `v0.1` is a version marker / compatibility tag.
- A `v0.1` branch is not used.
- Future v0.1 maintenance, if needed, must use a non-conflicting branch name such as `release/v0.1`.

## 4. Topology

Top-level layout:

- `README.md`
- `README.ja.md`
- `Adapter/`
- `CRANE-Kernel/`
- `Cargo.toml`
- `Cargo.lock`
- `crates/`
- `docs/`
- `resources/`
- `scripts/`
- `tests/`

`docs/current/` contains current public truth references.
`docs/deferred/` contains future-publication or upper-tier material that is not adopted into Free v0.1 beta claims.
`docs/historical/` contains non-authoritative historical material.
`docs/ja/` contains Japanese companion documents.

The repository root contains the runnable source tree.

Important implementation families:

1. `crates/cyrune-runtime-cli/`: `cyr` command family and user-facing runtime surface.
2. `crates/cyrune-daemon/`: daemon / host execution surface.
3. `crates/cyrune-control-plane/`: request validation, Working rebuild, policy gate, citation validation, and ledger commit.
4. `crates/cyrune-core-contract/`: request / result / denial / ID contract types.
5. `resources/bundle-root/embedding/`: shipping embedding pin and static payload references.

## 5. Script Root Chain

All public scripts are called from the repository root:

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

They derive:

1. `SCRIPT_DIR`: `scripts/`
2. `PUBLIC_ROOT`: repository root
3. `FREE_ROOT`: repository root
4. `STATE_ROOT`: `target/public-run`
5. `CYRUNE_HOME`: `target/public-run/home`

## 6. prepare-public-run Contract

`scripts/prepare-public-run.sh` must:

1. recreate `target/public-run/`,
2. download the configured release carrier,
3. verify filename, size, and SHA256,
4. reject unsafe tar members,
5. require the expected carrier manifest,
6. extract the home template,
7. build `cyrune-runtime-cli` and `cyrune-daemon`,
8. install `cyr` and `cyrune-daemon` under `target/public-run/bin/`.

The concrete carrier URL / filename / size / SHA256 values are beta release-contract pins. They are not product identity authority.

## 7. doctor Contract

`scripts/doctor.sh` must run only against prepared public-run state.

Expected success:

- exit code `0`
- JSON output
- `"status": "healthy"`

If public-run state is missing or invalid, it must fail instead of constructing hidden fallback state.

## 8. first-success Contract

`scripts/first-success.sh` must run `cyr verify first-success` through the prepared `cyr` binary.

Expected success:

- exit code `0`
- verifier report JSON output
- `verified` is `true`
- `outcome` is `accepted`
- `policy_pack_id` is `cyrune-free-default`
- an `evidence_id` is returned
- evidence files exist under `CYRUNE_HOME/ledger/evidence/<evidence_id>/`
- `CYRUNE_HOME/working/working.json` exists
- `CYRUNE_HOME/ledger/terminal-bindings/<evidence_id>.json` exists and binds the response, evidence hashes, and visible working hash

## 9. Change Impact Map

Changes to the following affect public-run behavior directly:

- root resolution in the public scripts
- carrier URL / size / SHA256 pins
- tar member safety validation
- binary names and installation paths
- `CYRUNE_HOME` layout
- `cyr verify first-success` report contract
- evidence ledger paths
- terminal binding marker paths
- `working/working.json`

Changes to the following affect public-reader interpretation:

- root README claim boundary
- public index reading order
- current / deferred / historical shelf placement
- Japanese companion routing
- release/tag wording

## 10. Non-Claims

This public beta does not claim:

- production maturity
- native distributable release
- installer packaging
- concrete signing / notarization values
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface

## 11. Validation

The public CI checks:

- public shell scripts parse,
- beta release-contract static predicates,
- Rust formatting,
- Rust workspace check,
- Rust lint with warnings denied.

Runtime first-success validation is documented in `docs/FIRST_SUCCESS_EXPECTED.md` and produces local evidence under `target/public-run/home/`.

The beta release-contract criteria are documented in `docs/BETA_CRITERIA.md`.
