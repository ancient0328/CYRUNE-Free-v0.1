# Public-Run Smoke Evidence

**Status**: Public reproducibility evidence shelf
**Subject**: CYRUNE Free v0.1 public-run smoke observations
**Scope**: Specific repository commits, local environments, and first-success runs

This directory records public-run smoke observations for CYRUNE Free v0.1.
Each report is an environment-specific observation of the documented public path:

```text
bash scripts/prepare-public-run.sh
bash scripts/doctor.sh
bash scripts/first-success.sh
```

These reports are public reproducibility notes. They are not release closeout evidence, not a Closed Gate Report, and not a production maturity claim.

## Evidence Boundary

Each report must state:

1. repository commit
2. OS and architecture
3. `rustc` and `cargo` versions
4. command results for `prepare-public-run.sh`, `doctor.sh`, and `first-success.sh`
5. first-success `correlation_id`
6. first-success `evidence_id`
7. terminal binding existence
8. `working_hash_after`
9. failure notes, using the closed reason set below

## Failure Reason Set

Use exactly one of these values when recording the result:

```text
none
environment
carrier_download
carrier_integrity
build
doctor
first_success
manual_abort
```

Use `none` only when all three public-run commands exit with status `0` and the first-success report has `verified: true`.

## File Naming

Use this format:

```text
YYYYMMDD-<os>-<arch>.md
```

Example:

```text
20260516-macos-arm64.md
```
