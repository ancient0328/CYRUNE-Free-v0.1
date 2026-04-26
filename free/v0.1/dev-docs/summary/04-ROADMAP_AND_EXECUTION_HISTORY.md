# CYRUNE Free v0.1: Execution Lines And Completion Map

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 0. Public Repository Reading Caveat

This execution history is supplementary in the public GitHub repository. It keeps historical source-side publication wording for traceability.

For the public repository envelope, use the current model fixed by `docs/CYRUNE_Free_Public_Index.md`: `main` is the latest public surface; `v0.1.0` is the immutable public alpha snapshot tag / release; existing `v0.1` is a compatibility tag, not a branch; source-side paths such as `Distro/CYRUNE/public/free-v0.1/` are provenance paths, not public-user paths.

## 1. この巻の役割

この巻は、現在成立している execution line を整理し、それぞれが何を固定したかを説明する。
歴史を網羅することが目的ではない。
現在の構造を理解するために必要な line だけを残す。

## 2. current accepted lines

現在の accepted line は次の 4 本である。

1. Free v0.1 core line
2. fixed problem corrective line
3. D6 native outer launcher line
4. D7 terminal bundle productization line

加えて、current accepted source には line とは別カテゴリの **accepted add-on scope** として `D7-RC1 external release concretization` が存在する。
`D7-RC1` は D7 line の reopen ではなく、D7 closeout 後に family-level concretization だけを追加採用した post-v0.1 add-on scope である。

## 3. Free v0.1 core line

### 3.1 目的

Free v0.1 core line の目的は、製品の最小成立条件を満たすことだった。

### 3.2 固定したもの

1. Control Plane 主導の単一 turn model
2. Working 10±2、Processing 42 日、Permanent 手動昇格
3. deny-by-default Policy Gate
4. citation-bound accepted output
5. append-only atomic Evidence Ledger
6. `cyr` 単一入口の Runtime
7. No-LLM と approved execution adapter の両 accepted path

### 3.3 core line の成立結果

Free v0.1 core は current accepted で成立済みである。

## 4. fixed problem corrective line

### 4.1 目的

shipping memory line に残っていた fixed problem 6 項目を、code、proof、gate、inventory、closeout wording の範囲で解消することだった。

### 4.2 解消した fixed problem

1. shipping embedding exact pin mismatch
2. shipping reject normalization の raw binding 依存
3. `view policy --pack` miswire
4. packaged unresolved reject の path leakage
5. stale / self-contradictory inventory
6. retention / ttl claim の evidence boundary overclaim

### 4.3 corrective line が固定したもの

1. shipping exact pin positive / negative proof
2. reject surface normalization
3. view policy routing correctness
4. predicate-separated retention proof
5. gate / index / inventory の再同期
6. final reclose と D5 packaged observation alignment

### 4.4 corrective line の成立結果

corrective line は current accepted で完了している。
`blocker none`、`shipping memory line fully complete`、`baseline fully closed` が current accepted claim として再許可されている。

## 5. D6 native outer launcher line

### 5.1 目的

packaged baseline を崩さずに、outer launcher を追加することだった。

### 5.2 D6 の追加内容

1. native outer launcher の public surface
2. `launcher -> cyr -> daemon -> Control Plane` handoff
3. launcher-owned accepted family
4. launcher / preflight / run-path の failure split
5. D7 separation

### 5.3 D6 が変えていないもの

1. `cyr` 単一入口
2. Control Plane の意味論
3. `BUNDLE_ROOT` authority
4. `CYRUNE_HOME` non-authority projection rule
5. D7 bundle productization owner

### 5.4 D6 の成立結果

D6 は current accepted line として完了している。

## 6. D7 terminal bundle productization line

### 6.1 目的

WezTerm bundle productization を追加しつつ、runtime semantics を変えないことだった。

### 6.2 D7 の追加内容

1. bundle identity family
2. rebrand family
3. notice / license / SBOM conduit
4. integrity / signature conduit
5. upstream intake judgment family
6. productization failure family
7. D7-specific proof driver family

### 6.3 D7 が変えていないもの

1. `cyr` public command family
2. D6 launcher-owned handoff
3. `BUNDLE_ROOT` authority
4. `CYRUNE_HOME` projection rule
5. run-path / preflight / launcher failure split

### 6.4 D7 の current accepted closeout

current accepted D7 closeout に採用したのは family-level productization behavior である。
一方で次の concrete external release detail は intentionally 未採用である。

1. reverse-DNS bundle identifier
2. signing identity
3. notarization provider
4. installer emitted name
5. upstream revision

### 6.5 D7 の成立結果

D7 は current accepted line として完了している。
ただし上記 5 detail を future release concretization に残している。

## 7. accepted line と non-blocker residual の違い

現在の理解で重要なのは、次を混同しないことである。

1. line が complete であること
2. line の外に別 owner の detail が残っていること

D7 は complete だが、未採用 concrete external release detail が残る。
これは D7 complete claim の否定ではない。

## 8. current completion map

| line | current status | 何が確立されたか |
|------|----------------|------------------|
| core | complete | Free v0.1 最小成立条件 |
| corrective | complete | fixed problem 6 項目の current accepted 解消 |
| D6 | complete | outer launcher line |
| D7 | complete | terminal bundle productization line |

## 9. current accepted scope / separate future scope

current ship-goal-side parent implementation lane と parent validation lane は complete であり、next owner / task は `none / none` である。
`D7RC1B-T2` は 2026-04-12 JST に完了し、rule-fixed family の accepted / fail-closed artifact と phase-end validation が current accepted add-on scope source になった。
同日、`D7RC1C-I1` は docs-only gate として完了し、signing identity の organization-owned owner、top-level `RELEASE_PREPARATION.json.signing_identity` input location、`release_preparation_failure` family への validation contract が current accepted add-on scope source になった。
同日、`D7RC1C-I2` は docs-only gate として完了し、notarization provider の organization-owned owner、top-level `RELEASE_PREPARATION.json.notarization_provider` input location、`release_preparation_failure` family への validation contract が current accepted add-on scope source になった。
同日、`D7RC1C-I3` は docs-only gate として完了し、organization-owned variable family の exact reason を `signing_identity_invalid` / `notarization_provider_invalid` に固定し、publicization boundary を fixed message / no-raw-detail leakage まで current accepted add-on scope source に同期した。
同日、`D7RC1C-T1` は accepted fixture artifact、field-level fail-closed artifact、root metadata invalidity split、phase-end validation を採用し、`D7-RC1-C` phase complete claim を current accepted add-on scope source に同期した。
同日、`D7RC1D-I1 / T1 / S1 / S2` は closeout family として完了し、`D7-RC1` final proof report、workspace validation、roadmap / inventory / summary sync を current accepted add-on scope source に同期した。
同日、`20260413-ship-goal-master-scope-and-boundary.md`、`SHIP_GOAL_MASTER_SCOPE_CANONICAL.md`、`SHIP_GOAL_MASTER_PHASE_FAMILY_CANONICAL.md`、`SHIP_GOAL_GITHUB_PUBLICATION_TOPOLOGY_CANONICAL.md`、`SHIP_GOAL_GITHUB_PUBLICATION_EXECUTION_CANONICAL.md`、`20260413-ship-goal-master-implementation-roadmap.md`、`20260413-ship-goal-master-test-roadmap.md` が追加され、ship goal side whole-roadmap は parent `SGM / SGT-M` lane として再構成された。
既存 `20260412-ship-goal-implementation-roadmap.md` と `20260412-ship-goal-test-roadmap.md` は `MGF-2 public corpus publication-unit materialization` の child lane に再配置された。
同日、all-agent brush-up cycle により parent global sync ownership、child closeout adoption boundary、shared canonical authority、residual 6 項目 comparator、current-state wording residual が解消し、master roadmap set は planning set として 6 Gate `Yes` に到達した。
2026-04-14 JST には `SGMD-I1 / SGMD-I2` blocker を受けた parent redesign により selected ship channel が `GitHub public release package channel` に再固定され、tracked public branch surface と GitHub-hosted non-tracked carrier の二面 publication contract へ進む parent ship-channel root が固定された。
同日、`publish_release_package_to_github.py` の actual execution により repository root tracked set `README.md` / `docs/`、release tag `v0.1`、release title `CYRUNE Free v0.1`、exact asset URL `cyrune-free-v0.1.tar.gz`、third-party reachability が materialize され、`SGM-D` は完了した。
同日、`ENGINEERING_SPEC.md` と `USER_GUIDE.md` は public package supplementary docs として role-fulfillment 修正が入り、`CYRUNE_Free_Public_Index.md` の supplementary reading list から到達可能になった。formal close root は `20260414-docs-EVID-public-package-docs-role-fulfillment-closeout.md` に固定され、この差分は ship-goal-side next owner / task を変更しない。
2026-04-15 JST には `SGTMA-T1` parent scope / topology consistency proof が完了し、parent ADR、master scope canonical、master phase family canonical の selected ship channel、master fixed problem family、child mapping 一致が検証された。
同日、`SGTMA-T2` child lane non-substitution proof が完了し、child `SGI / SGT` pair は `MGF-2 public corpus publication-unit materialization` の child lane としてだけ参照され、ship goal whole-roadmap の代替として扱われていないことが検証された。
同日、child `SGT-A` publication unit exactness proof も完了し、publication unit root 一意性、then-current execution canonical 準拠 manifest、non-adopted family 非混入、stale purge が formal proof として固定された。
同日、child `SGT-B` public authority surface proof も完了し、authority surface 4 文書の exact body template / heading set / direct-link set 一致と、historical / deferred-publication family の non-navigable plain text mention only が formal proof として固定された。
同日、child `SGT-C` ship-grade user journey proof が完了し、public package 3 script の accepted / fail-closed contract、success metrics、failure fixture が formal proof として固定された。
同日、child `SGT-D` publication validation closeout が完了し、current publication unit comparator は `expected = 129 / actual = 129 / missing = 0 / extra = 0 / byte mismatch = 0 / mode mismatch = 0`、authority-surface comparator、runtime comparator を pass した。
同日、parent `SGT-MC` GitHub release package publication execution proof が完了し、selected ship channel repository metadata、release tag / asset identity、then-current tracked publication surface、third-party obtain root reachability が formal proof として固定された。
同日、parent `SGT-MD` ship-goal parent closeout が完了し、child proof set、selected ship channel proof set、master closeout comparator、current-state docs sync により ship goal complete current-source claim と `next owner / task = none / none` が固定された。
同日、obsolete `publish_public_corpus_to_github.py` は live source / public payload から retired され、old-script rerun 手順を含む `20260413-public-EVID-SGMD-1-2-github-publication-blocker.md` は `証跡失効` として `99-archive/90-reports/` へ退避された。current live publication artifact は `publish_release_package_to_github.py` 1 本に再固定された。
同日、separate future scope `GPC` が fixed problem として追加され、`free/v0.1/0` stand-alone complete root、`public/free-v0.1/free/v0.1/0` GitHub publication branch、tracked public branch surface と GitHub-hosted non-tracked carrier の exact boundary を再固定する ADR / implementation roadmap / test roadmap が authority として追加された。
同日、`GPCI-A` と `GPCI-B` は完了し、split semantics と GitHub publication carrier contract authority が current source へ同期した。
同日、`GPCI-C` と `GPCI-D` は完了し、tracked public branch materialization、carrier-aware publish path、implementation-state docs sync が実装 lane として complete に到達した。
したがって、現時点の current accepted scope 側 next owner / task は `none / none` であり、child `SGI-D / SGID-S1` と child `SGT-D / SGTD-T4` は closed child targets として参照し、reopen しない。user-fixed 2 目的の operational lane では `20260423-gpc-operational-completion-implementation-roadmap.md` と `20260423-gpc-operational-completion-test-roadmap.md` を current roadmap authority として扱う。prior narrow-purpose roadmap pair は narrow semantics / prior proof authority としてのみ参照し、tracked GitHub branch、GitHub release carrier、public checkout preparation を operational completion condition から外す根拠にしてはならない。

## 10. public boundary publication design lane

current accepted product truth を public / internal / historical の 3 層へ落とす publication design lane は、product executable scope とは別に進行している。

現時点で完了済みなのは次である。

1. `PB-A` logical boundary freeze
2. `PB-B` family-level export manifest
3. `PBC-I1` historical / non-authoritative authority-state label 整流
4. `PBC-I2` public-only entry 追加
5. `PBC-I3` internal-only entry 追加
6. `PBC-T1` public corpus standalone audit
7. `PBC-S1` docs / index / summary sync
8. `PBD-I1` public-ready exact adoption set
9. `PBD-T1` public-ready logical closeout
10. `PBD-T2` read-only consistency check
11. `PBD-S1` current state / operational wording sync

この lane は publication design であり、`current accepted next executable scope` を変えない。
`PB-D` は完了済みであり、publication design lane 自体は complete である。physical publication に進む場合は別 scope が必要である。

## 11. ship goal executable lane

`ship goal` は `minimum completion gate` の後段にある別軸であり、current accepted scope closeout を維持したまま別 lane で進める。

現時点で追加された lane は次である。

1. `20260413-ship-goal-master-implementation-roadmap.md`
2. `20260413-ship-goal-master-test-roadmap.md`
3. `20260412-ship-goal-implementation-roadmap.md`
4. `20260412-ship-goal-test-roadmap.md`

この lane の固定内容は次である。

1. parent lane は `SGM / SGT-M` とする
2. child lane `SGI / SGT` は `MGF-2 public corpus publication-unit materialization` に限定する
3. publication unit root は `Distro/CYRUNE/public/free-v0.1/` のみ
4. implementation roadmap と test roadmap は parent / child の両方で分離する
5. native distributable と concrete release owner value finalization は本 lane に含めない
6. ship-goal-side implementation lane と parent validation lane は complete であり、next owner / task は `none / none`

### 11.1 GitHub publication carrier recanonicalization future scope

fixed problem `GPC` は、`free/v0.1/0` を stand-alone complete root、`public/free-v0.1/free/v0.1/0` を GitHub publication branch として再固定する separate future scope である。

現時点の到達点は次である。

1. current operational implementation authority は `20260423-gpc-operational-completion-implementation-roadmap.md`
2. current operational validation / closeout authority は `20260423-gpc-operational-completion-test-roadmap.md`
3. `20260423-gpc-narrow-fixed-problem-implementation-roadmap.md` と `20260423-gpc-narrow-fixed-problem-test-roadmap.md` は prior narrow-purpose roadmap pair としてのみ参照し、operational completion authority と混同しない
4. current operational roadmap が採用する implementation completion conditions は `free/v0.1/0` stand-alone semantics、`public/.../0` publication branch semantics、tracked GitHub branch、GitHub release carrier、public checkout preparation の全 operational surface である
5. exact tracked publication-branch surface と publication-branch state sync は、remote publication branch surface の入力 state として扱い、remote publication の代替として扱わない
6. non-tracked carrier、remote publication execution、exact asset URL、public checkout preparation は user-fixed 2 目的の operational surface として採用し、目的外へ退避しない
7. current accepted scope `none / none` は維持しつつ、user-fixed 2 目的の operational lane は本 operational roadmap pair に従う

## 12. 新 scope を起こす条件

新 scope を起こすべきなのは次の場合である。

1. current accepted line に含めていない detail を新たに固定したい
2. 既存 owner とは別の責務を持つ line を追加したい
3. current proof family とは別の accepted / fail-closed family が必要になった

## 13. 新 scope を起こしてはいけない場合

次のような理由で current line を reopen してはならない。

1. current accepted line に採用していない detail を retroactive blocker にする
2. standalone summary で外した legacy を completeness 条件に戻す
3. D6 / D7 の追加 family を core semantics へ逆流させる

## 14. 現在の最終結論

この巻の current accepted 結論は次である。

1. core は complete
2. corrective line は complete
3. D6 は complete
4. D7 は complete
5. current accepted scope の next owner / task は `none / none` である
6. ship goal executable lane は parent `SGM / SGT-M` と child `SGI / SGT` の二層で固定されている
7. separate future scope `GPC` の current operational roadmap authority は `20260423-gpc-operational-completion-implementation-roadmap.md` と `20260423-gpc-operational-completion-test-roadmap.md` である
8. publication design lane は complete である
9. native distributable に進む場合は別 scope が必要である
