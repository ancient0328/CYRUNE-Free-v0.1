# Engineering Spec

This document explains the public repository package structure and execution surface for engineers.
It does not replace the authority surface. Start with the root `README.md`, `docs/GETTING_STARTED.md`, and `docs/CYRUNE_Free_Public_Index.md`.

Language stance: this English spec is primary. The Japanese companion is `docs/ENGINEERING_SPEC-ja.md`.

## 1. Scope

This spec covers the public repository root and the package-level implementation contract for:

- `README.md`
- `docs/`
- `scripts/`
- `free/v0.1/0/`

It is intended to make these points unambiguous:

1. the physical public package surface
2. the exact three-script sequence and observables
3. the generated `target/public-run/` state model
4. the paths and values that affect public-run behavior
5. the minimum maintenance checks
6. the runtime / product scope this public alpha does not claim

This spec does not redefine:

- current accepted public truth
- task-level roadmap
- native distributable packaging
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface
- signing, notarization, or signed update delivery

## 2. Package Surfaces

The public repository has four surfaces:

1. discovery / authority surface
   `README.md`, `docs/GETTING_STARTED.md`, `docs/FIRST_SUCCESS_EXPECTED.md`, `docs/TROUBLESHOOTING.md`, `docs/CYRUNE_Free_Public_Index.md`
2. current public docs
   `docs/current/` plus the root-level guide and engineering files under `docs/`
3. separated reference shelves
   `docs/historical/` and `docs/deferred/`
4. runnable source tree
   `free/v0.1/0/`

The GitHub repository root is the public package root. Source-side paths such as `Distro/CYRUNE/public/free-v0.1/` are provenance paths from the private source workspace and are not required for public use.

## 2.1 Repository Publication Model

GitHub publication uses `main` as the latest public repository surface and SemVer tags as immutable snapshots.

For CYRUNE Free v0.1 public alpha:

- `main` points to the latest public docs and package surface.
- `v0.1.0` is an immutable snapshot tag and release for the Free v0.1 public alpha.
- Existing `v0.1` is preserved as a version marker / compatibility tag.
- A `v0.1` branch is not used.
- Maintenance branches, if needed later, must avoid tag-name collision, for example `release/v0.1`.

Updating `main` after `v0.1.0` does not move the immutable `v0.1.0` tag or release.

## 3. Runtime Roots

The public scripts do not use the caller's current directory as authority.
They resolve roots from their own location:

- `SCRIPT_DIR`
- `PUBLIC_ROOT`
- `FREE_ROOT`
- `STATE_ROOT`
- `CYRUNE_HOME`

The generated state lives under:

```text
free/v0.1/0/target/public-run/
├── bin/
│   ├── cyr
│   └── cyrune-daemon
└── home/
```

Meaning:

1. `SCRIPT_DIR`: the executed script directory
2. `PUBLIC_ROOT`: repository root / public package root
3. `FREE_ROOT`: included runnable source tree
4. `STATE_ROOT`: public-run state root recreated by preparation
5. `CYRUNE_HOME`: runtime home used by `doctor` and `first-success`

## 4. Script Responsibilities

### 4.1 `scripts/prepare-public-run.sh`

Responsibilities:

- move into `free/v0.1/0/`
- recreate `target/public-run/`
- download the configured release carrier
- verify carrier filename, size, and SHA256
- reject unsafe tar members
- expand the carrier home template into `target/public-run/home/`
- run `cargo build --quiet --release`
- install `cyr` and `cyrune-daemon` into `target/public-run/bin/`

Minimum observables:

1. exit code is `0`
2. configured carrier filename, size, and SHA256 match
3. archive contains no absolute path, parent traversal, symlink, hardlink, or device file member
4. expected release manifest exists inside the carrier
5. `target/public-run/bin/cyr` exists
6. `target/public-run/bin/cyrune-daemon` exists
7. `target/public-run/home/` exists

### 4.2 `scripts/doctor.sh`

Responsibilities:

- fix `CYRUNE_HOME` to `target/public-run/home`
- run `cyr doctor`
- pass through the raw JSON object

Minimum observables:

1. exit code is `0`
2. stdout is raw JSON
3. stdout contains `"status":"healthy"`
4. no wrapper banner or synthetic success JSON is added

### 4.3 `scripts/first-success.sh`

Responsibilities:

- fix `CYRUNE_HOME` to `target/public-run/home`
- run `cyr run --no-llm --input "ship-goal public first success"`
- pass through the raw JSON object

Minimum observables:

1. exit code is `0`
2. stdout is raw JSON
3. `correlation_id`, `run_id`, `evidence_id`, and `policy_pack_id` are non-empty
4. `policy_pack_id` is `cyrune-free-default`

## 5. Expected Execution Order

`docs/GETTING_STARTED.md` owns the canonical sequence:

1. `prepare-public-run.sh`
2. `doctor.sh`
3. `first-success.sh`

This spec explains the sequence; it does not redefine it.

## 6. Maintenance Checks

After docs or public-envelope changes, run the checks that match the public CI:

```bash
bash -n scripts/prepare-public-run.sh scripts/doctor.sh scripts/first-success.sh
cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check
cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets
cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings
```

For docs-only changes, also check that generated artifacts are not tracked:

```bash
git ls-files | rg '(^|/)(target|__pycache__)(/|$)|\.pyc$|\.pyo$|\.DS_Store$|cyrune-free-v0\.1\.tar\.gz$'
```

The expected result is zero matches.

## 7. Operational Boundaries

The public package assumes:

- `bash`
- `curl`
- `python3`
- `tar`
- `cargo`
- `install`
- network access to the configured release carrier URL
- a local filesystem that preserves regular files and executable mode

The concrete carrier URL / filename / size / SHA256 in `scripts/prepare-public-run.sh` are operational pins.
They are not product identity truth and do not expand the public alpha claim boundary.

## 8. Not Included

The public alpha does not include:

- internal operational corpus
- private development repository contents outside this public root
- native installer / archive variation
- signing or notarization workflow
- signed update package delivery
- release-owner concrete value handling
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface

Product-wide docs may describe signed update packages or no-self-update discipline as a CYRUNE / CITADEL design direction. This public Free v0.1 alpha does not ship a signed updater or signed update channel.

## 9. Reading Order

Recommended engineering reading order:

1. `README.md`
2. `docs/GETTING_STARTED.md`
3. `docs/FIRST_SUCCESS_EXPECTED.md`
4. `docs/CYRUNE_Free_Public_Index.md`
5. `docs/current/CYRUNE-Free_Canonical.md`
6. `docs/current/CYRUNE.md`
7. `docs/TROUBLESHOOTING.md`
8. `free/v0.1/0/`

Use this document after those files as the package-level engineering explanation.
