# GETTING_STARTED

Run the three scripts in order from the public repository root. `prepare-public-run.sh` downloads and validates the required carrier, normalizes it into `free/v0.1/0/target/public-run/`, builds the local runtime binaries, and prepares local runtime state. Do not skip steps or change the sequence.

This is a public alpha first-success path. It is expected to prepare local state, pass `doctor`, and return an accepted first-success JSON response for the packaged Free v0.1 no-LLM path. It does not prove native distribution, OS-level sandbox isolation, enforcement-complete classification / MAC, or Pro / Enterprise / CITADEL scope.

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

After running the sequence, read [FIRST_SUCCESS_EXPECTED.md](FIRST_SUCCESS_EXPECTED.md) to understand the expected JSON fields, generated evidence paths, and claim boundary.
