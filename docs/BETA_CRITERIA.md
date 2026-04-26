# CYRUNE Free v0.1 Public Beta Criteria

**Status**: Current public beta release-contract criteria
**Subject**: `main` and `v0.1.1-beta.1` for the CYRUNE Free v0.1 public repository surface

This document defines what the public repository means by **public beta**.

CYRUNE Free v0.1 public beta is not a label-only wording change. It is a release contract that binds:

1. tracked public source,
2. verified carrier archive,
3. immutable beta tag / release asset,
4. public CI,
5. public documentation,
6. first-success runtime evidence,
7. a matching Closed Gate Report.

## 1. Beta Release Line

- `main` is the latest public repository surface.
- `v0.1.0` remains the immutable public alpha snapshot.
- `v0.1.1-beta.1` is the first public beta release-contract line.
- A beta tag must not be moved after publication. If a beta release is withdrawn or superseded, the next attempt must use a new tag such as `v0.1.1-beta.2`.
- The Closed Gate Report is post-release closeout evidence on `main`. It is required for public beta closeout, but it is not required to be inside the immutable release tag snapshot because the report depends on release and CI evidence created after the tag exists.

## 2. Required Beta Evidence

A public beta claim requires all of the following evidence to exist for the same beta release line:

1. source commit SHA for the beta candidate,
2. `v0.1.1-beta.1` tag pointing at that commit,
3. GitHub release named for the beta tag,
4. one release asset named `cyrune-free-v0.1.1-beta.1.tar.gz`,
5. exact asset size and SHA256,
6. carrier `RELEASE_MANIFEST.json` matching the beta asset name and package root,
7. CI success for the beta candidate,
8. fresh `prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh` result,
9. returned `evidence_id` and expected accepted-run evidence files,
10. public docs consistency scan,
11. license / third-party notice boundary check,
12. Closed Gate Report under `free/v0.1/dev-docs/90-reports/`.

If any required evidence is missing or fails, the beta claim is not established.

## 3. Public Runtime Path

The public beta runtime path remains:

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

`prepare-public-run.sh` must download the configured beta carrier, verify filename, size, SHA256, and tar member safety, require `RELEASE_MANIFEST.json`, prepare local state, build runtime binaries from source, and install `cyr` / `cyrune-daemon` into `free/v0.1/0/target/public-run/bin/`.

`doctor.sh` and `first-success.sh` must run only against the prepared public-run state.

## 4. Public Beta Claim Boundary

This public beta claims a repeatable public repository release surface for the no-LLM Free v0.1 first-success path.

It does not claim:

- production maturity
- native distributable release
- installer packaging
- signed desktop distribution
- signed update channel
- concrete signing / notarization values
- OS-level sandbox process isolation
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface
- private development or internal operational corpus

## 5. Failure Semantics

The release contract is fail-closed.

Treat the beta candidate as not established if any of the following occurs:

- the beta tag is missing,
- the beta tag points at the wrong commit,
- the release asset is missing,
- asset size or SHA256 does not match the public pin,
- carrier `RELEASE_MANIFEST.json` does not match the beta asset,
- CI fails,
- `prepare-public-run.sh`, `doctor.sh`, or `first-success.sh` exits non-zero,
- first-success output omits required IDs,
- expected evidence files are missing,
- public docs imply an unsupported product scope,
- the Closed Gate Report is missing or has any Gate below `Yes`.

## 6. Version Continuity

The beta line keeps the Free v0.1 product subject while separating release maturity:

- `v0.1.0`: immutable public alpha snapshot
- `v0.1.1-beta.1`: public beta release contract

This avoids moving historical tags and prevents alpha evidence from being reused as beta evidence.
