# CYRUNE Free v0.1 Public Index

**Status**: Current accepted public authority reference
**Subject**: CYRUNE Free v0.1 public-only corpus
**Purpose**: Let readers follow the current public truth, the beta release contract, the first-success path, and the non-claim boundary without reading internal operational docs or historical drafts.

---

## 1. Role

This document is the public authority/reference map for CYRUNE Free v0.1.
It is not the product overview. The product-first entry points are the root `README.md` and `docs/GETTING_STARTED.md`.

This index points to current public truth and separated reference shelves. It does not make task roadmaps, raw proof payloads, or internal operational documents authoritative for the public beta.

Japanese companion material is available under `docs/ja/`. Japanese companion documents do not override this English public index.

## 2. Primary Reading Order

1. `docs/current/CYRUNE.md`
2. `docs/current/CYRUNE_ProblemStatement-En.md`
3. `docs/current/CYRUNE-Free_Canonical.md`
4. `docs/GETTING_STARTED.md`
5. `docs/FIRST_SUCCESS_EXPECTED.md`
6. `docs/BETA_CRITERIA.md`
7. `docs/USER_GUIDE.md`
8. `docs/ENGINEERING_SPEC.md`

The Japanese technical problem statement at `docs/current/CYRUNE_ProblemStatement-ja.md` is a companion document, not a line-by-line translation of `docs/current/CYRUNE_ProblemStatement-En.md`.

## 3. Supplementary References

Read these only when additional background is needed:

1. `docs/current/CYRUNE_Free_v0.1_Diagrams.html`
2. source-side public dev-docs at `Distro/CYRUNE/free/public/v01/dev-docs/`

The source-side public dev-docs tree contains development history, evidence reports, and operational notes. It does not override the public beta claim boundary, the repository publication model, or the primary reading order in this index.

## 4. Source-Side Path Boundary

On GitHub, the repository root is the public package root.
The source-side path `Distro/CYRUNE/free/public/v01/0/` is the local private-workspace mirror of that public package root.
It is not a path a public GitHub reader needs to have locally.

## 5. Authoritative Public Truth

This public index treats the following as current public truth:

1. CYRUNE Free v0.1 is a public beta repository for the single-user Free runtime.
2. The documented first-success path is `prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh`, where `first-success.sh` emits the `cyr verify first-success` report.
3. `cyr` is the user-facing entry command inside the prepared public-run state.
4. `BUNDLE_ROOT` is the runtime authority root.
5. `CYRUNE_HOME` is local state, not product authority.
6. Fail-closed behavior is part of the public beta runtime shape.
7. Sandbox scope is sandbox specification normalization / validation, not OS-level process isolation.
8. Classification / MAC is product intent and public claim boundary, not enforcement-complete lattice / clearance governance in this beta.
9. Concrete carrier URL / filename / size / SHA256 are release-contract pins in `scripts/prepare-public-run.sh`, not product identity truth.
10. GitHub `main` is the latest public repository surface.
11. SemVer tags are immutable snapshots; `v0.1.0` is the published CYRUNE Free v0.1 public alpha snapshot tag.
12. `v0.1.1-beta.1` is the first CYRUNE Free v0.1 public beta release-contract tag.
13. `v0.1` is a version marker / compatibility tag, not a branch name.
14. The Free public repository license is `MIT OR Apache-2.0` for first-party source unless a file or third-party notice states otherwise.

## 6. Non-Authority For This Public Beta

The following must not be treated as current public truth authority:

1. task-level roadmaps
2. raw proof / raw validation payloads
3. draft / superseded documents
4. broader product-line scope outside this Free v0.1 public beta
5. full Control OS product maturity
6. native distributable packaging
7. concrete reverse-DNS bundle identifier
8. concrete installer / archive filename
9. concrete signing identity value
10. concrete notarization provider value
11. signed update package delivery
12. a `v0.1` branch as the publication model
13. broader product-line surfaces as part of the Free repository license grant

## 7. Shelf Meaning

`docs/current/` contains current public product and problem-statement references.

`docs/ja/` contains Japanese companion documents.

## 8. Public Beta Claim Boundary

CYRUNE Free v0.1 public beta is a release-contract repository surface that explains and executes the public first-success path.

This beta claim does not include production maturity, native distributable packaging, OS-level sandbox enforcement, enforcement-complete classification / MAC, broader product-line scope, signing, notarization, installer distribution, or signed update package delivery.

## 9. Repository Publication Model

The CYRUNE public repository uses `main` as the latest public surface and immutable SemVer tags as release snapshots.

The published CYRUNE Free v0.1 public alpha snapshot tag is `v0.1.0`.
The first CYRUNE Free v0.1 public beta release-contract tag is `v0.1.1-beta.1`.
The existing `v0.1` tag is a version marker / compatibility tag. A branch named `v0.1` is not used.

## 10. Summary

CYRUNE Free v0.1 public corpus is the publication surface that lets readers follow the current product truth, the beta release contract, the public first-success path, and the non-claim boundary without internal operational material.
