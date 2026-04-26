# CYRUNE Free v0.1 Public Index

**Status**: Current accepted public authority reference
**Subject**: CYRUNE Free v0.1 public-only corpus authority / reference
**Purpose**: let a reader follow the current public truth, public first-success path, and separated reference shelves without reading internal operational docs.

---

## 1. Role

This document is the authority / reference index for the CYRUNE Free v0.1 public-only corpus.
It points to the current public truth and separates supporting, deferred, and historical material.

The product-first entry is the root `README.md` and `docs/GETTING_STARTED.md`.
This index is not a product overview and does not replace those files.

## 2. Language Policy

English is primary for the public GitHub entry path.
Japanese documents are companion material where explicitly present.

Current maintained Japanese companion documents:

1. `docs/current/CYRUNE_ProblemStatement-ja.md`
2. `docs/USER_GUIDE-ja.md`
3. `docs/ENGINEERING_SPEC-ja.md`

The English problem statement is primary for the public GitHub reading order.
The Japanese problem statement is a companion for the same structural problem framing, not a separate authority surface.

## 3. Authority / Reference Reading Order

Read in this order:

1. `docs/current/CYRUNE.md`
2. `docs/current/CYRUNE_ProblemStatement-En.md`
3. `docs/current/CYRUNE_ProblemStatement-ja.md`
4. `docs/current/CYRUNE-Free_Canonical.md`
5. `docs/GETTING_STARTED.md`
6. `docs/FIRST_SUCCESS_EXPECTED.md`
7. `docs/USER_GUIDE.md`
8. `docs/ENGINEERING_SPEC.md`

## 4. Supplementary Reading

Read these only when needed:

1. `docs/current/CYRUNE_Free_v0.1_Diagrams.html`
2. `docs/current/mermaid/`
3. `docs/USER_GUIDE-ja.md`
4. `docs/ENGINEERING_SPEC-ja.md`
5. `docs/historical/`
6. `docs/deferred/`
7. `free/v0.1/dev-docs/00-TARGET_SYSTEM.md`
8. `free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md`
9. `free/v0.1/dev-docs/summary/00-SUMMARY_INDEX.md`
10. `free/v0.1/dev-docs/summary/01-SYSTEM_AND_SCOPE.md`
11. `free/v0.1/dev-docs/summary/02-ARCHITECTURE_AND_RUNTIME_LINES.md`
12. `free/v0.1/dev-docs/summary/03-CANONICAL_CONTRACTS_AND_DATA_MODELS.md`
13. `free/v0.1/dev-docs/summary/07-CURRENT_STATE_AND_OPERATIONAL_GUIDE.md`
14. `free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md`
15. `free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md`
16. `free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md`

Items 5-16 are supplementary reference files, not the direct public authority entry path.
If supplementary dev-docs mention earlier source-side paths, publication branches, `v0.1` release wording, native packaging, classification / MAC completion, or carrier fixed values, those statements do not override this index.

## 5. Source-Side Path Boundary

On GitHub, the repository root is the public package root.
The source-side path `Distro/CYRUNE/public/free-v0.1/` is a private-workspace provenance path used before publication.
It is not a path a public GitHub reader needs to have locally.

## 6. Authority Truth For This Public Entry

This public entry may treat only the following as current public authority:

1. CYRUNE Free v0.1 current accepted public product truth
2. `cyr` single-entry
3. `BUNDLE_ROOT` single authority
4. `CYRUNE_HOME` non-authority
5. fail-closed family existence
6. Free v0.1 public alpha is first-success capable through the documented script path
7. sandbox scope is sandbox specification normalization / validation, not OS-level process isolation
8. classification / MAC is product intent and public claim boundary, not enforcement-complete lattice / clearance governance in this alpha
9. concrete carrier URL / filename / size / SHA256 are operational pins in `scripts/prepare-public-run.sh`, not product identity truth
10. GitHub `main` is the latest public repository surface
11. SemVer tags are immutable snapshots; `v0.1.0` is the CYRUNE Free v0.1 public alpha snapshot tag
12. existing `v0.1` is a version marker / compatibility tag, not a branch name

## 7. Not Authority For This Public Entry

Do not treat the following as current public authority:

1. task-level roadmap
2. current inventory in full
3. raw proof / raw validation output
4. organization-owned variable handling detail
5. historical / draft / superseded documents
6. full Control OS product maturity
7. Pro / Pro+ / Enterprise / CITADEL feature surface
8. native distributable packaging
9. concrete reverse-DNS bundle identifier
10. concrete installer / archive filename
11. concrete upstream revision
12. concrete signing identity value
13. concrete notarization provider value
14. signed update package delivery
15. a `v0.1` branch as the publication model

## 8. Historical Shelf

The following are not part of the current public truth entry:

1. `docs/historical/CYRUNE_Terminal.md`
2. `docs/historical/CYRUNE_Terminal_CanonicalDraft.md`
3. `docs/historical/CYRUNE_Free_v0.1_StructurePack.md`
4. `docs/historical/CYRUNE構図.md`
5. `docs/historical/CITADEL_CYRUNE.md`

They are retained only for background and history.

## 9. Deferred-Publication Shelf

The following are not automatically adopted into current Free v0.1 public truth:

1. `docs/deferred/CYRUNE_ProductTierCanonical.md`
2. `docs/deferred/CYRUNE-Pro_Canonical.md`
3. `docs/deferred/CYRUNE-Pro+_Canonical.md`
4. `docs/deferred/CYRUNE-Enterprise_Canonical.md`
5. `docs/deferred/CITADEL.md`
6. `docs/deferred/CITADEL_ThreatModel.md`
7. `docs/deferred/CYRUNE_TierReusableAssetInventory.md`
8. `docs/deferred/CRANE_3層メモリ.md`

These are not historical by default, but they require a separate publication decision before they can become current public truth.

## 10. Public Alpha Claim Boundary

CYRUNE Free v0.1 public alpha is scoped to a repository content shape that documents and can execute the public first-success path:

```text
prepare-public-run.sh -> doctor.sh -> first-success.sh
```

This alpha claim does not include native distributable packaging, OS-level sandbox enforcement, enforcement-complete classification / MAC, Pro / Enterprise / CITADEL scope, signing / notarization workflow, or signed update package delivery.

## 11. Repository Publication Model

`main` is the latest public surface.
SemVer tags are immutable snapshots.
The `v0.1.0` tag and release are the immutable CYRUNE Free v0.1 public alpha snapshot.
Updating `main` after `v0.1.0` does not move that tag or release.

## 12. One-Sentence Summary

The CYRUNE Free v0.1 public corpus is a publication surface that lets readers follow current public product truth, the public first-success path, language boundaries, and non-claim boundaries without reading internal operational docs.
