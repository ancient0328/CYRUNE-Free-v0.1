# FIRST_SUCCESS_EXPECTED

This document explains what `scripts/first-success.sh` is expected to show in the CYRUNE Free v0.1 public alpha.

It is a public-facing interpretation guide. It does not replace `GETTING_STARTED.md`, `TROUBLESHOOTING.md`, or `CYRUNE_Free_Public_Index.md`.

## 1. Command

Run this only after `prepare-public-run.sh` and `doctor.sh` have succeeded.

```bash
./scripts/first-success.sh
```

The script fixes `CYRUNE_HOME` to:

```text
free/v0.1/0/target/public-run/home
```

Then it runs:

```bash
cyr run --no-llm --input "ship-goal public first success"
```

## 2. Expected Stdout Shape

On success, stdout is a raw JSON object from `cyr run`. It must include non-empty values for:

- `correlation_id`
- `run_id`
- `evidence_id`
- `policy_pack_id`

The expected policy pack is:

```text
cyrune-free-default
```

The response may include additional fields such as `response_to`, `output`, `citation_bundle_id`, and `working_hash_after`.

## 3. Expected Local Evidence

When `evidence_id` is returned, the corresponding local evidence directory is expected under:

```text
free/v0.1/0/target/public-run/home/ledger/evidence/<evidence_id>/
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
free/v0.1/0/target/public-run/home/working/working.json
```

## 4. What This Result Means

This first-success path means the local public alpha repository has performed these steps in the prepared public-run state:

1. prepare local public-run state
2. run `cyr doctor` against that state
3. execute one no-LLM `cyr run`
4. return an accepted JSON response
5. commit local evidence and update local working projection

## 5. What This Does Not Prove

This first-success path does not prove:

- native distributable release
- installer packaging
- concrete signing or notarization
- OS-level sandbox process isolation
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature scope
- production or beta maturity

## 6. Failure Reading

Treat the result as failed if any of the following are true:

- the script exits non-zero
- stdout is not a JSON object
- `correlation_id`, `run_id`, `evidence_id`, or `policy_pack_id` is missing or empty
- `policy_pack_id` is not `cyrune-free-default`
- the evidence directory for the returned `evidence_id` is missing
- any expected accepted-run evidence file listed in section 3 is missing
- `working/working.json` is missing after a reported success

For remediation, follow `TROUBLESHOOTING.md`.

## 7. Rerun Semantics

Each `first-success.sh` invocation must be evaluated independently.

If `first-success.sh` is rerun without rerunning `prepare-public-run.sh`, do not assume the same `run_id` or `evidence_id`. Use the IDs returned by that specific run and inspect the matching evidence directory.

If `prepare-public-run.sh` is rerun, it recreates `free/v0.1/0/target/public-run/`. Treat evidence under the previous public-run state root as non-current for the new run.
