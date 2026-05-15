# FIRST_SUCCESS_EXPECTED

This document explains what `scripts/first-success.sh` is expected to show in the CYRUNE Free v0.1 public beta release contract.

It is a public-facing interpretation guide. It does not replace `GETTING_STARTED.md`, `TROUBLESHOOTING.md`, or `CYRUNE_Free_Public_Index.md`.

## 1. Command

Run this only after `prepare-public-run.sh` and `doctor.sh` have succeeded.

```bash
./scripts/first-success.sh
```

The script fixes `CYRUNE_HOME` to:

```text
target/public-run/home
```

Then it runs:

```bash
cyr verify first-success
```

## 2. Expected Stdout Shape

On success, stdout is a verifier report JSON object. It must include:

- `schema_version` with value `cyrune.free.first-success-verifier-report.v1`
- `verified` with value `true`
- `outcome` with value `accepted`
- `correlation_id`
- `run_id`
- `evidence_id`
- `policy_pack_id`
- `working_hash_after`
- `evidence_dir`
- `terminal_binding_path`
- `state_root`
- `cyrune_home`

The expected policy pack is:

```text
cyrune-free-default
```

The report contains the full accepted response under `response`.

## 3. Expected Local Evidence

When `evidence_id` is returned, the corresponding local evidence directory is expected under:

```text
target/public-run/home/ledger/evidence/<evidence_id>/
```

For an accepted run, the evidence directory is expected to contain:

- `manifest.json`
- `run.json`
- `policy.json`
- `citation_bundle.json`
- `rr.json`
- `working_delta.json`
- `stdout.log`
- `stderr.log`
- `hashes.json`

The working projection is expected under:

```text
target/public-run/home/working/working.json
```

The terminal binding marker is expected under:

```text
target/public-run/home/ledger/terminal-bindings/<evidence_id>.json
```

The verifier accepts the run only when the response, manifest, evidence files, terminal binding marker, and `working/working.json` raw hash agree.

## 4. What This Result Means

This first-success path means the local public beta repository has performed these steps in the prepared public-run state:

1. prepare local public-run state
2. run `cyr doctor` against that state
3. execute one no-LLM run through `cyr verify first-success`
4. return a verifier report with `outcome: "accepted"`
5. commit local evidence, update local working projection, and write the terminal binding marker

For the public beta release contract, this result is one required evidence item. The full beta criteria are defined in `docs/BETA_CRITERIA.md`.

## 5. What This Does Not Prove

This first-success path does not prove:

- native distributable release
- installer packaging
- concrete signing or notarization
- OS-level sandbox process isolation
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature scope
- production maturity

## 6. Failure Reading

Treat the result as failed if any of the following are true:

- the script exits non-zero
- stdout is not a JSON object
- `verified` is not `true` or `outcome` is not `accepted`
- `correlation_id`, `run_id`, `evidence_id`, `policy_pack_id`, `state_root`, or `cyrune_home` is missing or empty
- `policy_pack_id` is not `cyrune-free-default`
- the evidence directory for the returned `evidence_id` is missing
- any expected accepted-run evidence file listed in section 3 is missing
- `working/working.json` is missing after a reported success
- the terminal binding marker for the returned `evidence_id` is missing

For remediation, follow `TROUBLESHOOTING.md`.

## 7. Rerun Semantics

Each `first-success.sh` invocation must be evaluated independently.

If `first-success.sh` is rerun without rerunning `prepare-public-run.sh`, do not assume the same `run_id` or `evidence_id`. Use the IDs returned by that specific run and inspect the matching evidence directory.

If `prepare-public-run.sh` is rerun, it recreates `target/public-run/`. Treat evidence under the previous public-run state root as non-current for the new run.
