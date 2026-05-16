# Public-Run Smoke Evidence: <OS> <ARCH>

**Status**: Public reproducibility observation
**Subject**: CYRUNE Free v0.1 public-run smoke
**Recorded at**: `<YYYY-MM-DD HH:MM:SS JST>`
**Repository commit**: `<commit>`
**Failure reason**: `<none|environment|carrier_download|carrier_integrity|build|doctor|first_success|manual_abort>`

## Environment

| Field | Value |
| --- | --- |
| OS | `<os>` |
| Kernel | `<kernel>` |
| Architecture | `<arch>` |
| rustc | `<rustc --version>` |
| cargo | `<cargo --version>` |

## Commands

| Step | Command | Exit | Result |
| --- | --- | ---: | --- |
| 1 | `bash scripts/prepare-public-run.sh` | `<exit>` | `<result>` |
| 2 | `bash scripts/doctor.sh` | `<exit>` | `<result>` |
| 3 | `bash scripts/first-success.sh` | `<exit>` | `<result>` |

## First-Success Report

| Field | Value |
| --- | --- |
| report path | `target/public-run/first-success-report.json` |
| schema_version | `<schema_version>` |
| checked_at | `<checked_at>` |
| run_mode | `<run_mode>` |
| verified | `<true|false>` |
| outcome | `<accepted|rejected>` |
| correlation_id | `<correlation_id>` |
| run_id | `<run_id>` |
| evidence_id | `<evidence_id>` |
| evidence_dir | `<evidence_dir>` |
| terminal_binding_path | `<terminal_binding_path>` |
| terminal binding exists | `<yes|no>` |
| working_hash_after | `<sha256:...>` |
| evidence_manifest_hash | `<sha256:...>` |
| evidence_hashes_hash | `<sha256:...>` |

## Notes

`<notes>`

## Boundary

This observation records one public-run smoke result for one commit and one local environment. It does not replace the beta release contract, release closeout evidence, or a Closed Gate Report.
