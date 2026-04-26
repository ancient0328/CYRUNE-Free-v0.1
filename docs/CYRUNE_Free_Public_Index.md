# CYRUNE Free v0.1 Public Index

**Status**: Current accepted public authority reference
**Subject**: CYRUNE Free v0.1 public-only corpus
**Purpose**: Let readers follow the current public truth, the first-success path, and the non-claim boundary without reading internal operational docs or historical drafts.

---

## 1. Role

This document is the public authority/reference map for CYRUNE Free v0.1.
It is not the product overview. The product-first entry points are the root `README.md` and `docs/GETTING_STARTED.md`.

This index points to current public truth and separated reference shelves. It does not make task roadmaps, raw proof payloads, historical drafts, or deferred tier documents authoritative for the public alpha.

Japanese companion material is available under `docs/ja/`. Japanese companion documents do not override this English public index.

## 2. Primary Reading Order

1. `docs/current/CYRUNE.md`
2. `docs/current/CYRUNE_ProblemStatement-En.md`
3. `docs/current/CYRUNE-Free_Canonical.md`
4. `docs/GETTING_STARTED.md`
5. `docs/FIRST_SUCCESS_EXPECTED.md`
6. `docs/USER_GUIDE.md`
7. `docs/ENGINEERING_SPEC.md`

The Japanese technical problem statement at `docs/current/CYRUNE_ProblemStatement-ja.md` is a companion document, not a line-by-line translation of `docs/current/CYRUNE_ProblemStatement-En.md`.

## 3. Supplementary References

Read these only when additional background is needed:

1. `docs/current/CYRUNE_Free_v0.1_Diagrams.html`
2. `docs/current/mermaid/`
3. `docs/historical/`
4. `docs/deferred/`
5. `free/v0.1/dev-docs/`

The `free/v0.1/dev-docs/` tree contains development history, evidence reports, and operational notes. It does not override the public alpha claim boundary, the repository publication model, or the primary reading order in this index.

## 4. Source-Side Path Boundary

On GitHub, the repository root is the public package root.
The source-side path `Distro/CYRUNE/public/free-v0.1/` is a private-workspace provenance path used before publication.
It is not a path a public GitHub reader needs to have locally.

## 5. Authoritative Public Truth

This public index treats the following as current public truth:

1. CYRUNE Free v0.1 is a public alpha repository for the single-user Free runtime.
2. The documented first-success path is `prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh`.
3. `cyr` is the user-facing entry command inside the prepared public-run state.
4. `BUNDLE_ROOT` is the runtime authority root.
5. `CYRUNE_HOME` is local state, not product authority.
6. Fail-closed behavior is part of the public alpha runtime shape.
7. Sandbox scope is sandbox specification normalization / validation, not OS-level process isolation.
8. Classification / MAC is product intent and public claim boundary, not enforcement-complete lattice / clearance governance in this alpha.
9. Concrete carrier URL / filename / size / SHA256 are operational pins in `scripts/prepare-public-run.sh`, not product identity truth.
10. GitHub `main` is the latest public repository surface.
11. SemVer tags are immutable snapshots; `v0.1.0` is the published CYRUNE Free v0.1 public alpha snapshot tag.
12. `v0.1` is a version marker / compatibility tag, not a branch name.
13. The Free public repository license is `MIT OR Apache-2.0` for first-party source unless a file or third-party notice states otherwise.

## 6. Non-Authority For This Public Alpha

The following must not be treated as current public truth authority:

1. task-level roadmaps
2. raw proof / raw validation payloads
3. historical / draft / superseded documents
4. deferred Pro / Pro+ / Enterprise / CITADEL tier scope
5. full Control OS product maturity
6. native distributable packaging
7. concrete reverse-DNS bundle identifier
8. concrete installer / archive filename
9. concrete signing identity value
10. concrete notarization provider value
11. signed update package delivery
12. a `v0.1` branch as the publication model
13. Pro / Pro+ / Enterprise / CITADEL product surfaces as part of the Free repository license grant

## 7. Shelf Meaning

`docs/current/` contains current public product and problem-statement references.

`docs/deferred/` contains documents that may be relevant to future publication decisions, but are not automatically adopted into Free v0.1 public alpha claims.

`docs/historical/` contains historical or non-authoritative material retained for background only.

`docs/ja/` contains Japanese companion documents.

## 8. Public Alpha Claim Boundary

CYRUNE Free v0.1 public alpha is a repository content surface that explains and executes the public first-success path.

This alpha claim does not include native distributable packaging, OS-level sandbox enforcement, enforcement-complete classification / MAC, Pro / Enterprise / CITADEL scope, signing, notarization, installer distribution, or signed update package delivery.

## 9. Repository Publication Model

The CYRUNE public repository uses `main` as the latest public surface and immutable SemVer tags as release snapshots.

The published CYRUNE Free v0.1 public alpha snapshot tag is `v0.1.0`.
The existing `v0.1` tag is a version marker / compatibility tag. A branch named `v0.1` is not used.

## 10. Summary

CYRUNE Free v0.1 public corpus is the publication surface that lets readers follow the current product truth, the public first-success path, and the non-claim boundary without internal operational material.
