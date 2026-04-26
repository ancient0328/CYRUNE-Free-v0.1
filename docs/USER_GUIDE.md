# User Guide

This guide explains how to use the CYRUNE Free v0.1 public repository as a public alpha package.
It does not replace the root `README.md`, `docs/GETTING_STARTED.md`, or `docs/CYRUNE_Free_Public_Index.md`.

Language stance: this English guide is primary. The Japanese companion is `docs/USER_GUIDE-ja.md`.

## 1. What This Package Does

CYRUNE Free v0.1 is a single-user public alpha package.
It prepares local public-run state on the host, then uses `cyr` to reach the no-LLM first-success path.

This public alpha does not claim native distribution, OS-level sandbox enforcement, enforcement-complete classification / MAC, Pro / Enterprise / CITADEL scope, signing, notarization, or a signed update mechanism.

## 2. Package Contents

The repository root contains:

- `README.md`
- `docs/`
- `scripts/`
- `free/`

Most users should start with:

- `docs/GETTING_STARTED.md`
- `docs/FIRST_SUCCESS_EXPECTED.md`
- `docs/TROUBLESHOOTING.md`
- `scripts/prepare-public-run.sh`
- `scripts/doctor.sh`
- `scripts/first-success.sh`

## 3. Before You Start

Run the package from the repository root in a local terminal.
The host must provide:

- `bash`
- `curl`
- `python3`
- `tar`
- `cargo`
- `install`
- network access to the configured release carrier URL
- a local filesystem that preserves executable permissions

Before running scripts, confirm that `README.md`, `docs/`, `scripts/`, and `free/` are visible at the repository root.

## 4. Start Sequence

The canonical execution order is defined by `docs/GETTING_STARTED.md`.
The user-facing sequence has exactly three steps:

1. prepare local public-run state with `prepare-public-run.sh`
2. diagnose the prepared state with `doctor.sh`
3. verify the no-LLM first-success path with `first-success.sh`

From the repository root:

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

Do not skip steps or change the order.

## 5. Step Meaning

### 5.1 `prepare-public-run.sh`

This step downloads the release carrier, checks filename / size / SHA256, rejects unsafe tar members, expands the carrier home template, builds the required local binaries from the included Free source tree, and prepares local public-run state.

Successful preparation creates:

- `free/v0.1/0/target/public-run/bin/cyr`
- `free/v0.1/0/target/public-run/bin/cyrune-daemon`
- `free/v0.1/0/target/public-run/home/`

### 5.2 `doctor.sh`

This step runs `cyr doctor` against the prepared public-run state.
Successful output is a raw JSON object whose `"status"` is `"healthy"`.

### 5.3 `first-success.sh`

This step runs `cyr run --no-llm` against the same public-run state.
Successful output is a raw JSON object containing non-empty `correlation_id`, `run_id`, `evidence_id`, and `policy_pack_id`.
The expected `policy_pack_id` is `cyrune-free-default`.

## 6. How To Read Success

This guide does not define the accepted predicate.
For users, success means all of the following are true:

1. `prepare-public-run.sh` returns to the prompt and creates `target/public-run/bin/` and `target/public-run/home/`
2. `doctor.sh` returns a raw JSON object with `"status":"healthy"`
3. `first-success.sh` returns a raw JSON object with `correlation_id`, `run_id`, `evidence_id`, and `policy_pack_id`

If any item is missing, treat the run as failed and use `docs/TROUBLESHOOTING.md`.
For expected first-success files and output shape, read `docs/FIRST_SUCCESS_EXPECTED.md`.

## 7. Failure Handling

The canonical remediation source is `docs/TROUBLESHOOTING.md`.
The minimum user rule is:

1. if `doctor.sh` fails, rerun `prepare-public-run.sh` first
2. if `first-success.sh` fails, rerun `prepare-public-run.sh`, then `doctor.sh`, then `first-success.sh`
3. if host prerequisites are missing, fix them before rerunning

Failure includes:

- a command stops with a non-zero exit code
- `doctor.sh` does not show `"status":"healthy"`
- `first-success.sh` does not show the required IDs and policy pack ID

## 8. Not Included

This public package does not include:

- native distributable release
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- concrete signing / notarization values
- signed update package delivery
- Pro / Pro+ / Enterprise / CITADEL feature surface
- private development / internal operational corpus
- organization-specific operational workflow

## 9. Next Documents

Recommended reading order:

1. `README.md`
2. `docs/GETTING_STARTED.md`
3. `docs/FIRST_SUCCESS_EXPECTED.md`
4. `docs/TROUBLESHOOTING.md`
5. `docs/CYRUNE_Free_Public_Index.md`
6. `docs/ENGINEERING_SPEC.md`
