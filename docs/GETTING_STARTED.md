# GETTING_STARTED

Run the three scripts in order from the public repository root. `prepare-public-run.sh` downloads and validates the required carrier, normalizes it into `target/public-run/`, builds the local runtime binaries, and prepares local runtime state. Do not skip steps or change the sequence.

This is the public beta first-success path for the `v0.1.1-beta.1` release contract. It is expected to prepare local state from the pinned beta carrier, pass `doctor`, and return a verifier report with `verified: true` and `outcome: "accepted"` for the packaged Free v0.1 no-LLM path. It does not prove production maturity, native distribution, OS-level sandbox isolation, enforcement-complete classification / MAC, or broader product-line scope.

## Prerequisites

The host must provide:

- `bash`
- `curl`
- `python3`
- `tar`
- `cargo`
- `install`
- network access to the configured release carrier URL
- a local filesystem that preserves executable permissions

If any prerequisite is missing, `prepare-public-run.sh` must fail instead of fabricating a success state.

## 1. prepare-public-run.sh

```bash
./scripts/prepare-public-run.sh
```

## 2. doctor.sh

```bash
./scripts/doctor.sh
```

## 3. first-success.sh

```bash
./scripts/first-success.sh
```

## 4. Read The Expected Result

After running the sequence, read [FIRST_SUCCESS_EXPECTED.md](FIRST_SUCCESS_EXPECTED.md) to understand the expected verifier report, terminal binding marker, generated evidence paths, and claim boundary.

The beta release-contract criteria are defined in [BETA_CRITERIA.md](BETA_CRITERIA.md).
