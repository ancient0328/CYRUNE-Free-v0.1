# USER_GUIDE

This guide explains how to use the CYRUNE Free v0.1 public repository from the repository root.

It does not replace the root `README.md`, `GETTING_STARTED.md`, `FIRST_SUCCESS_EXPECTED.md`, or `TROUBLESHOOTING.md`.

Japanese companion guidance is available at `docs/ja/USER_GUIDE.md`.

## 1. What This Package Does

CYRUNE Free v0.1 is a single-user public beta package.

It lets a local host prepare public-run state from the pinned beta carrier, run `cyr doctor`, and execute the no-LLM first-success semantic verifier through the packaged Free v0.1 Control Plane path.

This public beta does not claim production maturity, native distribution, OS-level sandbox enforcement, enforcement-complete classification / MAC, Pro / Enterprise / CITADEL scope, signing, notarization, or installer distribution.

## 2. Repository Contents

The public repository root contains:

- `README.md`
- `docs/`
- `scripts/`
- `free/`

The normal user path uses:

- `docs/GETTING_STARTED.md`
- `docs/FIRST_SUCCESS_EXPECTED.md`
- `docs/BETA_CRITERIA.md`
- `docs/TROUBLESHOOTING.md`
- `scripts/prepare-public-run.sh`
- `scripts/doctor.sh`
- `scripts/first-success.sh`

## 3. Prerequisites

The host must provide:

- `bash`
- `curl`
- `python3`
- `tar`
- `cargo`
- `install`
- network access to the configured release carrier URL
- a local filesystem that preserves executable permissions

If a prerequisite is missing, `prepare-public-run.sh` must fail instead of fabricating success.

## 4. Start Sequence

Run the scripts in this exact order from the repository root:

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

Do not skip steps or change the sequence.

## 5. Step Meaning

### 5.1 prepare-public-run

This step downloads the configured release carrier, checks filename, size, SHA256, and tar member safety, extracts the home template into public-run state, builds the runtime binaries from the Free source tree, and installs `cyr` / `cyrune-daemon` into `target/public-run/bin/`.

### 5.2 doctor

This step runs `cyr doctor` against the prepared public-run state.

Expected result: a JSON object with `"status": "healthy"`.

### 5.3 first-success

This step runs the semantic verifier:

```bash
cyr verify first-success
```

Expected result: a JSON report containing `verified: true`, `outcome: "accepted"`, `correlation_id`, `run_id`, `evidence_id`, `policy_pack_id`, `state_root`, and `cyrune_home`.

Read `docs/FIRST_SUCCESS_EXPECTED.md` for the expected evidence paths and output fields.

## 6. Expected Local State

After a successful prepare step, the public-run state is under:

```text
target/public-run/
```

After a successful first-success step, inspect:

- `target/public-run/home/ledger/manifests/index.jsonl`
- `target/public-run/home/ledger/evidence/<evidence_id>/`
- `target/public-run/home/ledger/terminal-bindings/<evidence_id>.json`
- `target/public-run/home/working/working.json`

## 7. Failure Handling

If `prepare-public-run.sh` fails, do not continue to `doctor.sh`.

If `doctor.sh` fails, rerun `prepare-public-run.sh` first.

If `first-success.sh` fails, rerun `prepare-public-run.sh`, confirm `doctor.sh` passes, then rerun `first-success.sh`.

## 8. Non-Claims

Successful first-success means the C5 verifier accepted the response, evidence bundle, terminal binding marker, and visible working projection for the documented no-LLM Free v0.1 flow. The full beta release contract additionally requires the source, carrier asset, CI, public docs, and Closed Gate evidence defined in `docs/BETA_CRITERIA.md`.

It does not prove:

- native distributable release
- installer packaging
- signing or notarization
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL functionality
