# CYRUNE Free v0.1

CYRUNE Free v0.1 is a **public alpha** repository for the single-user CYRUNE Free runtime. It is shaped for the first-success path: prepare the public-run state, run `cyr doctor`, then run one no-LLM `cyr run` through the packaged Control Plane path.

This repository is a public-facing Free v0.1 publication unit. It is not a native installer, not a signed desktop distribution, and not the Pro / Enterprise / CITADEL product surface.

## Versioning

- `main` is the latest public repository surface.
- SemVer tags, such as `v0.1.0`, are immutable snapshots of this public repository content.
- `v0.1.0` is the published immutable CYRUNE Free v0.1 public alpha snapshot tag.
- `v0.1` is treated as a version marker / compatibility tag, not as a branch name.
- This repository does not use a `v0.1` branch. A maintenance branch, if ever needed, should use a non-conflicting name such as `release/v0.1`.
- Version tags do not expand the public alpha claim boundary described below.

## Language

English is the primary language for the public repository entry path.

Japanese companion documents are available for readers who need them:

- [README.ja.md](README.ja.md)
- [docs/ja/](docs/ja/)
- [Japanese technical problem statement](docs/current/CYRUNE_ProblemStatement-ja.md)

Japanese companion documents do not override the English public claim boundary in this README, [Getting Started](docs/GETTING_STARTED.md), or [Public Index](docs/CYRUNE_Free_Public_Index.md).

## Start Here

1. [Getting Started](docs/GETTING_STARTED.md)
2. [First Success Expected Result](docs/FIRST_SUCCESS_EXPECTED.md)
3. [Troubleshooting](docs/TROUBLESHOOTING.md)
4. [Public Index](docs/CYRUNE_Free_Public_Index.md)
5. [Free Source](free/v0.1/0/)

## What This Alpha Provides

- The public repository contains the Free v0.1 source surface and the public scripts needed for the first-success flow.
- `prepare-public-run.sh` prepares local state under `free/v0.1/0/target/public-run/`.
- `doctor.sh` checks that the prepared state is diagnosable.
- `first-success.sh` runs a no-LLM request and returns the accepted JSON response when the path succeeds.

## Current Claim Boundary

- Sandbox scope: this alpha documents and uses sandbox specification normalization / validation. It does not claim OS-level process isolation.
- Classification / MAC scope: CYRUNE product docs describe the intended classification and MAC model. This Free v0.1 public alpha does not claim enforcement-complete classification lattice or clearance governance.
- Evidence scope: first-success creates local runtime evidence for the no-LLM path. It does not prove native distribution, signing, notarization, Pro features, Enterprise governance, or CITADEL hardening.
- Signed update scope: product-wide docs may describe signed update or no-self-update discipline as a design direction. This Free v0.1 public alpha does not ship a signed updater or signed update channel.

## Repository Contents

- `README.md`: public product entry
- `docs/`: public documentation, expected first-success output, and separated reference shelves
- `scripts/`: public entry scripts
- `free/v0.1/0/`: runnable Free source tree

## License

CYRUNE Free v0.1 first-party source is licensed under either MIT or Apache-2.0, at your option. See [LICENSE](LICENSE), [LICENSE-MIT](LICENSE-MIT), and [LICENSE-APACHE](LICENSE-APACHE).

Third-party notices for redistributed model/tokenizer resources are tracked in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). This Free repository license does not license Pro / Pro+ / Enterprise / CITADEL product surfaces.

## Not Included

- Native distributable release
- Installer packaging
- Concrete signing / notarization values
- Signed update package delivery
- OS-level sandbox enforcement
- Enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface
- Private development / internal operational corpus
