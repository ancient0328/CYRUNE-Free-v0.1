# CYRUNE Free v0.1 Public Index

**状態**: Current accepted public authority reference
**主語**: CYRUNE Free v0.1 public-only corpus の authority/reference
**目的**: 第三者が internal operational docs や historical docs を読まずに、current accepted public truth と separated reference shelves を辿れるようにする

---

## 1. この文書の役割

この文書は、CYRUNE Free v0.1 の **public-only corpus** を読むための authority/reference である。
ここで案内するのは current accepted public truth だけであり、task-level roadmap、gate operation、raw proof payload、historical / draft 文書は入口に含めない。

product-first の入口は root `README.md` と `GETTING_STARTED.md` である。
この文書自体は product overview ではなく、**public-safe authority list** である。

## 2. authority/reference 読順

1. `docs/current/CYRUNE.md`
2. `docs/current/CYRUNE_ProblemStatement-ja.md` または `docs/current/CYRUNE_ProblemStatement-En.md`
3. `docs/current/CYRUNE-Free_Canonical.md`
4. `docs/GETTING_STARTED.md`
5. `docs/FIRST_SUCCESS_EXPECTED.md`
6. `docs/USER_GUIDE.md`
7. `docs/ENGINEERING_SPEC.md`

## 3. 補助的に読む文書

必要に応じて次を読む。

1. `free/v0.1/dev-docs/00-TARGET_SYSTEM.md`
2. `free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md`
3. `docs/current/CYRUNE_Free_v0.1_Diagrams.html`
4. `docs/current/mermaid/` 配下の diagram source
5. `docs/ENGINEERING_SPEC.md`
6. `docs/USER_GUIDE.md`
7. `docs/historical/`
8. `docs/deferred/`
9. `free/v0.1/dev-docs/summary/00-SUMMARY_INDEX.md`
10. `free/v0.1/dev-docs/summary/01-SYSTEM_AND_SCOPE.md`
11. `free/v0.1/dev-docs/summary/02-ARCHITECTURE_AND_RUNTIME_LINES.md`
12. `free/v0.1/dev-docs/summary/03-CANONICAL_CONTRACTS_AND_DATA_MODELS.md`
13. `free/v0.1/dev-docs/summary/07-CURRENT_STATE_AND_OPERATIONAL_GUIDE.md`
14. `free/v0.1/dev-docs/90-reports/20260410-terminal-D6-native-outer-launcher-proof.md`
15. `free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md`
16. `free/v0.1/dev-docs/90-reports/20260412-terminal-EVID-D7RC1D-1-external-release-concretization-closeout.md`

上記 7-16 は supporting file list であり、current public authority surface の direct link set ではない。
linked dev-docs が classification / MAC、carrier fixed value、native packaging に触れる場合も、この文書の public alpha claim boundary を上書きしない。

## 4. この入口で authority として扱う truth

この入口で authority として扱ってよいのは次である。

1. CYRUNE Free v0.1 current accepted product truth
2. `cyr` single-entry
3. `BUNDLE_ROOT` single authority
4. `CYRUNE_HOME` non-authority
5. fail-closed family の存在
6. Free v0.1 public alpha is first-success capable through the documented script path
7. sandbox scope is sandbox specification normalization / validation, not OS-level process isolation
8. classification / MAC is product intent and public claim boundary, not enforcement-complete lattice / clearance governance in this alpha
9. concrete carrier URL / filename / size / SHA256 are operational pins in `scripts/prepare-public-run.sh`, not product identity truth
10. GitHub `main` is the latest public repository surface
11. SemVer tags are immutable snapshots; `v0.1.0` is the intended CYRUNE Free v0.1 public alpha snapshot tag
12. `v0.1` is a version marker / compatibility tag, not a branch name

## 5. この入口で authority として扱ってはいけないもの

次は current accepted public truth の authority として扱ってはならない。

1. task-level roadmap
2. current inventory の全量
3. exact manifest
4. raw proof / raw validation output
5. organization-owned variable handling detail
6. historical / draft / superseded 文書
7. full Control OS product maturity
8. Pro / Pro+ / Enterprise / CITADEL feature surface
9. native distributable packaging
10. concrete reverse-DNS bundle identifier
11. concrete installer / archive filename
12. concrete upstream revision
13. concrete signing identity value
14. concrete notarization provider value
15. a `v0.1` branch as the publication model

## 6. historical / non-authoritative 文書の扱い

次は current accepted public truth の入口に含めない。

1. `docs/historical/CYRUNE_Terminal.md`
2. `docs/historical/CYRUNE_Terminal_CanonicalDraft.md`
3. `docs/historical/CYRUNE_Free_v0.1_StructurePack.md`
4. `docs/historical/CYRUNE構図.md`
5. `docs/historical/CITADEL_CYRUNE.md`

これらは historical / non-authoritative corpus であり、背景や初期構想の参照に限定する。

## 7. deferred-publication 文書の扱い

次は current Free v0.1 public truth に自動採用しない。

1. `docs/deferred/CYRUNE_ProductTierCanonical.md`
2. `docs/deferred/CYRUNE-Pro_Canonical.md`
3. `docs/deferred/CYRUNE-Pro+_Canonical.md`
4. `docs/deferred/CYRUNE-Enterprise_Canonical.md`
5. `docs/deferred/CITADEL.md`
6. `docs/deferred/CITADEL_ThreatModel.md`
7. `docs/deferred/CYRUNE_TierReusableAssetInventory.md`
8. `docs/deferred/CRANE_3層メモリ.md`

これらは historical ではないが、別の publication decision が必要である。

## 8. public alpha claim boundary

CYRUNE Free v0.1 public alpha は、`prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh` の public first-success path を説明・実行できる repository content shape を主語にする。
この alpha claim は、native distributable、OS-level sandbox enforcement、enforcement-complete classification / MAC、Pro / Enterprise / CITADEL scope を含まない。

## 9. repository publication model

CYRUNE Free public repository は、`main` を latest public surface として提示し、SemVer tag を immutable snapshot として扱う。
CYRUNE Free v0.1 public alpha の snapshot tag は `v0.1.0` を使用する。既存の `v0.1` は version marker / compatibility tag として扱い、同名 branch は作成しない。

## 10. 現時点の一文結論

CYRUNE Free v0.1 public corpus は、第三者が internal operational docs を読まずに current accepted product truth、public first-success path、non-claim boundary を辿るための publication surface である。
この文書は、その authority/reference と separated shelf boundary を固定する。
