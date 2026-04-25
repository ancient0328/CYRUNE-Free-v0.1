# Public Repo Envelope Hardening Evidence

**Date (JST)**: 2026-04-26 00:25:14 JST
**Correlation ID**: EVID-PREH-1
**Reason**: PUBLIC_REPO_ENVELOPE_HARDENING
**Scope**: `Distro/CYRUNE/public/free-v0.1/` public repository content shape
**Operation**: Public Repo Envelope Hardening for CYRUNE Free v0.1 public alpha

## Target

Make `Distro/CYRUNE/public/free-v0.1/` itself read as a coherent public GitHub repository for CYRUNE Free v0.1 public alpha / first-success-capable use.

## In Scope

- root README public alpha framing
- public docs index role correction
- getting-started first-success claim boundary
- user guide / engineering spec link and prerequisite updates
- `docs/FIRST_SUCCESS_EXPECTED.md`
- root `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`
- root `.gitignore`
- `.github/workflows/public-ci.yml`
- docs shelf separation for current, historical, and deferred-publication materials

## Out Of Scope

- runtime feature changes
- Rust behavior changes
- remote visibility
- repository naming
- native distributable packaging
- signing / notarization packaging
- Pro / Pro+ / Enterprise / CITADEL feature surface

## Final Public Envelope Shape

- `README.md`
- `LICENSE`
- `LICENSE-MIT`
- `LICENSE-APACHE`
- `.gitignore`
- `.github/workflows/public-ci.yml`
- `docs/CYRUNE_Free_Public_Index.md`
- `docs/GETTING_STARTED.md`
- `docs/FIRST_SUCCESS_EXPECTED.md`
- `docs/TROUBLESHOOTING.md`
- `docs/USER_GUIDE.md`
- `docs/ENGINEERING_SPEC.md`
- `docs/current/`
- `docs/deferred/`
- `docs/historical/`
- `scripts/`
- `free/v0.1/0/`

## Static Validation Observed

Commands were run from `Distro/CYRUNE/public/free-v0.1/` unless stated otherwise.

1. Public scripts parse:
   - `bash -n scripts/prepare-public-run.sh`
   - `bash -n scripts/doctor.sh`
   - `bash -n scripts/first-success.sh`
   - Result: exit code `0`.

2. Rust formatting:
   - `cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check`
   - Result: exit code `0`.

3. Rust type/build check:
   - `cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets`
   - Result: exit code `0`.

4. Rust lint/warning check:
   - `cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings`
   - Result: exit code `0`.

5. Required envelope presence:
   - `test -f README.md`
   - `test -f LICENSE`
   - `test -f LICENSE-MIT`
   - `test -f LICENSE-APACHE`
   - `test -f .gitignore`
   - `test -f .github/workflows/public-ci.yml`
   - `test -f docs/FIRST_SUCCESS_EXPECTED.md`
   - `test -d docs/current`
   - `test -d docs/deferred`
   - `test -d docs/historical`
   - Result: exit code `0`.

6. Old root-level docs absence:
   - `test ! -e docs/CYRUNE.md`
   - `test ! -e docs/CYRUNE-Free_Canonical.md`
   - `test ! -e docs/CYRUNE_ProductTierCanonical.md`
   - `test ! -e docs/CYRUNE_Terminal.md`
   - `test ! -e docs/CITADEL.md`
   - Result: exit code `0`.

7. Old root-path reference scan:
   - `rg -n "docs/(CYRUNE\\.md|CYRUNE_ProblemStatement|CYRUNE-Free_Canonical|CYRUNE_Free_v0\\.1_Diagrams|mermaid/|CYRUNE_ProductTierCanonical|CYRUNE-Pro|CYRUNE-Enterprise|CITADEL|CYRUNE_Terminal|CYRUNE_Free_v0\\.1_StructurePack|CYRUNE構図|CRANE_3層メモリ)" README.md docs scripts .github -S`
   - Result: exit code `1`, no matches.

8. Claim-boundary scan:
   - `rg -n "classification|MAC|clearance|unclassified|sandbox|isolation|native distributable|CITADEL|Enterprise|Pro\\+|Pro /|carrier fixed value|native packaging" README.md docs/CYRUNE_Free_Public_Index.md docs/GETTING_STARTED.md docs/USER_GUIDE.md docs/ENGINEERING_SPEC.md docs/FIRST_SUCCESS_EXPECTED.md docs/current -S`
   - Result: exit code `0`; matches were reviewed as claim-boundary, non-goal, conceptual, historical-reference, or target-model wording. No runtime implementation claim was adopted from this scan.

## Changed-File Inventory

No Git worktree covers `Distro/CYRUNE/public/free-v0.1/`; this inventory is source-bound to pre/post filesystem observations and the file operations performed in this patch phase.

Base path: `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1`

### Added

- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/.github/workflows/public-ci.yml`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/.gitignore`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/LICENSE`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/LICENSE-APACHE`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/LICENSE-MIT`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/FIRST_SUCCESS_EXPECTED.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/README.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/README.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/README.md`

### Edited In Place

- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/README.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_Free_Public_Index.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/GETTING_STARTED.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/USER_GUIDE.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/ENGINEERING_SPEC.md`

### Moved Then Edited Or Reclassified

- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/CYRUNE.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE-Free_Canonical.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/CYRUNE-Free_Canonical.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_ProblemStatement-En.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/CYRUNE_ProblemStatement-En.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_ProblemStatement-ja.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/CYRUNE_ProblemStatement-ja.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_Free_v0.1_Diagrams.html` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/CYRUNE_Free_v0.1_Diagrams.html`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/mermaid/` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/current/mermaid/`

### Moved To Deferred Shelf

- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_ProductTierCanonical.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CYRUNE_ProductTierCanonical.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE-Pro_Canonical.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CYRUNE-Pro_Canonical.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE-Pro+_Canonical.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CYRUNE-Pro+_Canonical.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE-Enterprise_Canonical.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CYRUNE-Enterprise_Canonical.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CITADEL.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CITADEL.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CITADEL_ThreatModel.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CITADEL_ThreatModel.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_TierReusableAssetInventory.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CYRUNE_TierReusableAssetInventory.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CRANE_3層メモリ.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/deferred/CRANE_3層メモリ.md`

### Moved To Historical Shelf

- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_Terminal.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/CYRUNE_Terminal.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_Terminal_CanonicalDraft.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/CYRUNE_Terminal_CanonicalDraft.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE_Free_v0.1_StructurePack.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/CYRUNE_Free_v0.1_StructurePack.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CYRUNE構図.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/CYRUNE構図.md`
- `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/CITADEL_CYRUNE.md` -> `/Users/ancient0328/Development/GitHub/CRANE-project/Distro/CYRUNE/public/free-v0.1/docs/historical/CITADEL_CYRUNE.md`

## Claim-Boundary Calibration

| Source lines | Match class | Category | Allowed reading | Invalidated if |
| --- | --- | --- | --- | --- |
| `README.md:5`, `README.md:40-42` | native / upper tier / sandbox / MAC | Non-goal boundary | These are exclusions from Free v0.1 public alpha. | The text says the public alpha provides any excluded item. |
| `README.md:24-26` | sandbox / classification / MAC / CITADEL | Claim boundary | Sandbox is normalization / validation; classification/MAC is intended model, not enforcement-complete. | The text drops "does not claim" or names OS isolation / complete MAC as current implementation. |
| `docs/GETTING_STARTED.md:5` | native / sandbox / MAC / upper tier | Non-goal boundary | First-success path scope is limited and explicitly excludes those items. | The path is described as proving native, OS sandbox, MAC completion, or upper-tier scope. |
| `docs/FIRST_SUCCESS_EXPECTED.md:84-89` | native / sandbox / MAC / upper tier | Non-goal boundary | First-success result does not prove those items. | These items move into expected success meaning. |
| `docs/USER_GUIDE.md:12`, `docs/USER_GUIDE.md:120-124` | native / sandbox / MAC / upper tier | Non-goal boundary | User-facing package exclusions. | The guide treats these as included package features. |
| `docs/ENGINEERING_SPEC.md:29-32`, `docs/ENGINEERING_SPEC.md:247-249` | native / sandbox / MAC / upper tier | Non-goal boundary | Engineering non-goals and operational boundaries. | The spec makes these implementation requirements for this public alpha. |
| `docs/CYRUNE_Free_Public_Index.md:49` | classification / MAC / carrier / native packaging | Override boundary | Linked dev-docs cannot override public alpha claim boundary. | Linked dev-docs are restored to primary public authority or override this boundary. |
| `docs/CYRUNE_Free_Public_Index.md:61-62`, `docs/CYRUNE_Free_Public_Index.md:114` | sandbox / classification / MAC / native / upper tier | Authority boundary | Current public authority narrows these claims. | Section 4 or 8 adopts OS isolation, enforcement-complete MAC, native, or upper-tier claims. |
| `docs/CYRUNE_Free_Public_Index.md:76-77`, `docs/CYRUNE_Free_Public_Index.md:92`, `docs/CYRUNE_Free_Public_Index.md:102-105` | upper tier / CITADEL / native | Shelf boundary | These are non-authority, deferred, or historical paths. | Deferred/historical documents are moved back into current truth. |
| `docs/current/CYRUNE-Free_Canonical.md:7`, `docs/current/CYRUNE-Free_Canonical.md:49`, `docs/current/CYRUNE-Free_Canonical.md:104` | classification / MAC / clearance / unclassified | Scope note and target/design boundary | Classification/MAC and unclassified-data rejection are target/design scope; executable public-alpha enforcement is not claimed. | The target/design qualifier is removed or the public alpha adopts executable classification/MAC rejection without matching source. |
| `docs/current/CYRUNE.md:6`, `docs/current/CYRUNE.md:88` | CITADEL / classification / MAC / clearance | Product overview boundary | Product overview is not public-alpha implementation claim. | The overview is used as current implementation proof. |
| `docs/current/CYRUNE.md:18`, `docs/current/CYRUNE.md:72`, `docs/current/CYRUNE.md:81`, `docs/current/CYRUNE.md:94-106` | classification / MAC / clearance | Target model concept | These describe CYRUNE target model, constrained by line 88. | The target model text is read without the line 88 public-alpha scope note. |
| `docs/current/CYRUNE.md:202-213`, `docs/current/CYRUNE_ProblemStatement-En.md:199` | CITADEL | Relationship / deferred tier | CITADEL appears only as relationship or hardening concept. | CITADEL is presented as Free v0.1 public alpha scope. |
| `docs/current/CYRUNE_Free_v0.1_Diagrams.html:132`, `docs/current/mermaid/free-v0.1-boundary-enforcement.mmd:8` | classification | Boundary wording | Diagram uses "classification boundary", not completed MAC enforcement. | Diagram text changes back to completed classification/MAC enforcement. |
| `docs/current/CYRUNE_ProblemStatement-En.md:3`, `docs/current/CYRUNE_ProblemStatement-En.md:45`, `docs/current/CYRUNE_ProblemStatement-En.md:114`, `docs/current/CYRUNE_ProblemStatement-En.md:201` | classification / upper tier | Problem statement with scope note | Describes AI risk and needed boundary, not current implementation proof. | The scope note is removed or the problem statement is used as implementation evidence. |
| `docs/current/CYRUNE_ProblemStatement-ja.md:3` | classification / sandbox / upper tier | Problem statement with scope note | Narrows the technical problem statement away from public-alpha implementation claims. | The scope note is removed or contradicted. |

## Branch And Tag Publication Strategy Observation

Observed at 2026-04-26 08:34:01 JST to 2026-04-26 08:52 JST.

- Remote URL used for publication worktree: `https://github.com/ancient0328/CYRUNE.git`
- Remote refs observed before push work:
  - `refs/heads/main`
  - `refs/tags/v0.1`
- `refs/tags/v0.1.0` was not present before tag creation gate evaluation.
- `README.md` now states:
  - `main` is the latest public repository surface.
  - SemVer tags such as `v0.1.0` are immutable snapshots of this public repository content.
  - `v0.1` is treated as a version marker / compatibility tag, not as a branch name.
- `docs/CYRUNE_Free_Public_Index.md` now states:
  - `main` is the latest public repository surface.
  - SemVer tags are immutable snapshots.
  - `v0.1.0` is the intended CYRUNE Free v0.1 public alpha snapshot tag.
  - a `v0.1` branch is not the publication model.
- `.gitignore` excludes Python generated artifacts:
  - `__pycache__/`
  - `*.py[cod]`
- Local source validation after branch/tag wording update:
  - `bash -n scripts/prepare-public-run.sh scripts/doctor.sh scripts/first-success.sh`: exit code `0`
  - `cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check`: exit code `0`
  - `cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets`: exit code `0`
  - `cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings`: exit code `0`
- Publication worktree validation after final sync:
  - `bash -n scripts/prepare-public-run.sh scripts/doctor.sh scripts/first-success.sh`: exit code `0`
  - `cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check`: exit code `0`
  - `cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets`: exit code `0`
  - `cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings`: exit code `0`

## Non-Assertions

This report does not assert:

- runtime first-success execution
- GitHub remote visibility state
- repository naming
- publication approval
- native distributable packaging
- signing or notarization
- Pro / Pro+ / Enterprise / CITADEL feature surface
- Closed Gate Report status
