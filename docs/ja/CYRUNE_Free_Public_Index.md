# CYRUNE Free v0.1 Public Index

**Status**: Japanese companion for the current public authority reference
**Subject**: CYRUNE Free v0.1 public-only corpus
**Purpose**: internal operational docs や historical drafts を読まずに、current public truth、first-success path、non-claim boundary を辿れるようにする。

---

## 1. 役割

この文書は、`docs/CYRUNE_Free_Public_Index.md` の日本語 companion です。

product-first entry point は root `README.md` と `docs/GETTING_STARTED.md` です。この日本語 companion は英語版 Public Index を上書きしません。

## 2. Primary Reading Order

公開リポジトリを読む順序は次です。

1. `docs/current/CYRUNE.md`
2. `docs/current/CYRUNE_ProblemStatement-En.md`
3. `docs/current/CYRUNE-Free_Canonical.md`
4. `docs/GETTING_STARTED.md`
5. `docs/FIRST_SUCCESS_EXPECTED.md`
6. `docs/USER_GUIDE.md`
7. `docs/ENGINEERING_SPEC.md`

日本語の technical problem statement は `docs/current/CYRUNE_ProblemStatement-ja.md` にあります。これは companion document であり、`docs/current/CYRUNE_ProblemStatement-En.md` の line-by-line translation ではありません。

## 3. Supplementary References

追加背景が必要な場合にのみ、次を参照します。

1. `docs/current/CYRUNE_Free_v0.1_Diagrams.html`
2. `docs/current/mermaid/`
3. `docs/historical/`
4. `docs/deferred/`
5. `free/v0.1/dev-docs/`

`free/v0.1/dev-docs/` は development history、evidence reports、operational notes を含みます。それらは public alpha claim boundary、repository publication model、primary reading order を上書きしません。

## 4. Authoritative Public Truth

この public index が current public truth として扱うものは次です。

1. CYRUNE Free v0.1 は single-user Free runtime の public alpha repository です。
2. documented first-success path は `prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh` です。
3. `cyr` は prepared public-run state 内の user-facing entry command です。
4. `BUNDLE_ROOT` は runtime authority root です。
5. `CYRUNE_HOME` は local state であり、product authority ではありません。
6. fail-closed behavior は public alpha runtime shape の一部です。
7. sandbox scope は sandbox specification normalization / validation であり、OS-level process isolation ではありません。
8. classification / MAC は product intent と public claim boundary であり、この alpha における enforcement-complete lattice / clearance governance ではありません。
9. concrete carrier URL / filename / size / SHA256 は `scripts/prepare-public-run.sh` の operational pins であり、product identity truth ではありません。
10. GitHub `main` は latest public repository surface です。
11. SemVer tags は immutable snapshots であり、`v0.1.0` は published CYRUNE Free v0.1 public alpha snapshot tag です。
12. `v0.1` は version marker / compatibility tag であり、branch 名ではありません。
13. Free public repository license は、別ファイルまたは third-party notice が異なる条件を示す場合を除き、first-party source について `MIT OR Apache-2.0` です。

## 5. Non-Authority For This Public Alpha

次は、この public alpha の current public truth authority ではありません。

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
11. `v0.1` branch as the publication model
12. Pro / Pro+ / Enterprise / CITADEL product surfaces が Free repository license grant に含まれること

## 6. Shelf Meaning

`docs/current/` は current public product and problem-statement references を含みます。

`docs/deferred/` は future publication decisions に関係し得るが、Free v0.1 public alpha claims へ自動採用されない文書を含みます。

`docs/historical/` は background のためだけに保持される historical / non-authoritative material を含みます。

`docs/ja/` は Japanese companion documents を含みます。

## 7. Public Alpha Claim Boundary

CYRUNE Free v0.1 public alpha は、current product truth、public first-success path、non-claim boundary を公開する repository content surface です。

この alpha claim は、native distributable packaging、OS-level sandbox enforcement、enforcement-complete classification / MAC、Pro / Enterprise / CITADEL scope、signing、notarization、installer distribution を含みません。

## 8. Repository Publication Model

CYRUNE public repository は、`main` を latest public surface、immutable SemVer tags を release snapshots として扱います。

published CYRUNE Free v0.1 public alpha snapshot tag は `v0.1.0` です。
既存の `v0.1` tag は version marker / compatibility tag です。`v0.1` branch は使用しません。

## 9. Summary

CYRUNE Free v0.1 public corpus は、internal operational material を読まなくても、current product truth、public first-success path、non-claim boundary を追跡できるようにする公開面です。
