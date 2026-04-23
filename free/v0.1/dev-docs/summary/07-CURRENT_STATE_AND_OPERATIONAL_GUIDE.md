# CYRUNE Free v0.1: Current State And Operational Guide

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. current accepted state / ship-goal-side future scope

現在の accepted state は次である。

1. Free v0.1 core は complete
2. fixed problem corrective line は complete
3. D6 native outer launcher line は complete
4. D7 terminal bundle productization line は complete
5. ship-goal-side parent implementation lane は complete
6. current accepted scope 側の blocker は無い
7. next owner / task は `none / none` である
8. public package supplementary docs `docs/ENGINEERING_SPEC.md` と `docs/USER_GUIDE.md` は role-fulfillment closeout 済みであり、`CYRUNE_Free_Public_Index.md` の supplementary reading list から到達できる
9. ship goal complete は current source として成立している

### 1.1 separate future scope `GPC` の current truth

current accepted scope を reopen しない separate future scope `GPC` の current truth は次である。

1. `free/v0.1/0` は stand-alone complete root として扱う
2. `public/free-v0.1/free/v0.1/0` は GitHub publication branch として扱う
3. current operational implementation authority は `20260423-gpc-operational-completion-implementation-roadmap.md` である
4. current operational validation / closeout authority は `20260423-gpc-operational-completion-test-roadmap.md` である
5. prior narrow-purpose roadmap pair は narrow semantics / prior proof authority としてのみ参照し、operational completion authority と混同しない
6. current operational roadmap が採用する implementation completion conditions は stand-alone complete root、GitHub publication branch root、tracked GitHub branch、GitHub release carrier、public checkout preparation の全 operational surface である
7. exact tracked publication-branch surface と publication-branch state sync は remote publication branch surface の入力 state であり、remote publication の代替ではない
8. `20260415` roadmap pair と broader carrier / historical-bridge proof family は historical broader line として残すが、本 operational roadmap pair の authority を狭める根拠にしない
9. current accepted scope の next owner / task は引き続き `none / none` である

## 2. current closeout wording

現在の closeout wording として許可されるのは次である。

1. `blocker none`
2. `shipping memory line fully complete`
3. `baseline fully closed`
4. `D6 complete`
5. `D7 complete`

これらは current accepted scope と ship-goal-side current-source closeout の範囲では正しい。  
ただし native distributable や release-owner concrete value まで完了した、という意味ではない。

## 3. non-blocker residual

現在残っているが blocker ではないものは次である。

1. reverse-DNS bundle identifier concrete value
2. installer emitted name concrete value
3. upstream revision concrete value
4. concrete signing identity value
5. concrete notarization provider value
6. storage open/init failure surface 一般の no-leakage

これらは current accepted claim に採用していない。  
したがって current blocker でも current active task でもない。
ただし `D7-RC1` add-on scope により、signing identity / notarization provider の contract、input location、exact reason、publicization boundary、accepted / fail-closed artifact family 自体は既に fixed 済みである。  
また 2026-04-14 JST の parent redesign により `cyrune-free-v0.1.tar.gz` は selected ship channel asset filename として固定済みであり、archive filename は residual detail から外れている。  
現在 residual として残っているのは、release-owner concrete value と storage open/init failure surface 一般の no-leakage だけである。

## 4. 運用時に守るべき最重要不変条件

### 4.1 single-entry

`cyr` 単一入口を崩してはならない。

### 4.2 Control Plane first

実行の accept / reject は Control Plane を通らなければならない。

### 4.3 single immutable authority

packaged static authority は `BUNDLE_ROOT` のみである。

### 4.4 non-authority home

`CYRUNE_HOME` は mutable state / generated / materialized projection であり、static authority ではない。

### 4.5 fail-closed

不明・不足・未検証・未解決を success に落としてはならない。

### 4.6 evidence-first closeout

成立主張は accepted / fail-closed / validation family と Closed Gate Report を揃えて初めて許可される。

## 5. 現在の運用判断

### 5.1 既存 line の扱い

core、corrective、D6、D7 は current accepted line として凍結状態にある。  
これらを「なんとなく改善したい」理由で reopen してはならない。

### 5.2 既存 line を reopen してよい場合

次のどちらかに限る。

1. current accepted claim を直接崩す current-state 矛盾が見つかった
2. existing accepted scope 自体を再定義する明示判断が必要になった

### 5.3 reopen してはいけない場合

1. current accepted claim に採用していない detail を blocker 化したいだけ
2. future release detail を retroactive に既存 line の未完として扱いたい
3. legacy catalog を completeness 条件に戻したい

### 5.4 public package supplementary docs の扱い

`docs/ENGINEERING_SPEC.md` と `docs/USER_GUIDE.md` は public package supplementary docs として current source に含まれる。  
これらは engineer-facing detailed spec と general-user usage manual を補うが、public entry、execution order、failure remediation の canonical owner を置換しない。  
公開入口は `CYRUNE_Free_Public_Index.md`、実行順序は `GETTING_STARTED.md`、失敗時 remediation は `TROUBLESHOOTING.md` が引き続き正である。

## 6. 新 scope を追加する正しい順序

新しい scope を追加する時は、次の順序を崩してはならない。

1. target を定義する
2. canonical を作る
3. executable roadmap を作る
4. exact test / proof manifest を作る
5. 実装する
6. accepted / fail-closed / validation family を採る
7. docs / inventory / gate を同期する
8. Closed Gate Report を作る

## 7. 新 scope に必要な最小設計項目

1. 主語 / 非主語
2. D5 / D6 / D7 との継承関係
3. authority root
4. generated / materialized path
5. accepted family
6. fail-closed family
7. validation family
8. blocker と non-blocker residual の切り分け

## 8. change review checklist

変更案を評価する時は次を確認する。

1. `cyr` 単一入口を壊していないか
2. Control Plane bypass が無いか
3. bundle-root authority を増殖させていないか
4. home copy を authority 化していないか
5. run-path / preflight / launcher / productization failure を混線させていないか
6. uncited claim や ledger-less success を許していないか
7. current accepted line に採用していない detail を retroactive blocker にしていないか

## 9. 第三者が最初に知るべき operational truth

1. CYRUNE Free v0.1 は current accepted scope では already complete である
2. ship goal complete は current source として成立している
3. D7 current executable line 自体は外部 release concretization concrete value を採用していないが、family-level concretization は `D7-RC1` add-on scope として完了している
4. 新しい作業は既存 line の continuation ではなく、separate future scope として起こすのが正しい
5. 現在の ship-goal-side whole-roadmap は parent `SGM / SGT-M` であり、`SGI / SGT` は `MGF-2` child lane である
6. separate future scope `GPC` の current operational roadmap authority は `20260423-gpc-operational-completion-implementation-roadmap.md` と `20260423-gpc-operational-completion-test-roadmap.md` であり、stand-alone complete root、GitHub publication branch root、tracked GitHub branch、GitHub release carrier、public checkout preparation の全 operational surface を user-fixed 2 目的の達成面として扱う
7. current GitHub publication executable は `publish_release_package_to_github.py` 1 本だけであり、obsolete repo-root publication script は retired 済みである

## 10. 現時点の次の一手

`D7RC1B-T2`、`D7RC1C-I1`、`D7RC1C-I2`、`D7RC1C-I3`、`D7RC1C-T1`、`D7RC1D-I1 / T1 / S1 / S2` は完了済みである。  
また publication design lane では、public / internal / historical boundary、public-only entry、internal-only entry、public corpus standalone audit、public-ready logical closeout まで完了している。  
さらに、2026-04-13 JST に ship goal master scope、GitHub publication topology / execution canonical、parent roadmap が追加され、child `SGI / SGT` pair は `MGF-2 public corpus publication-unit materialization` に再配置された。  
したがって、current accepted scope の次 owner / task は `none / none` のまま維持する。user-fixed 2 目的の operational lane は `20260423-gpc-operational-completion-implementation-roadmap.md` と `20260423-gpc-operational-completion-test-roadmap.md` を current authority とし、prior narrow-purpose roadmap pair や `20260415` broader roadmap pair を operational completion authority として読んではならない。

## 11. この corpus の operational conclusion

standalone summary corpus を読む第三者が、現時点で実務上覚えておくべき結論は次の 10 個で十分である。

1. current accepted scope は成立済みである
2. authority、single-entry、fail-closed は崩してはいけない
3. D6 と D7 は core semantics を変えずに add-on line として完了している
4. ship goal complete は current source として成立している
5. 次の executable 変更は parent `SGM / SGT-M` lane として起こし、`SGI / SGT` は child lane として使う
6. publication design lane は complete である
7. separate future scope `GPC` の current operational roadmap authority は `20260423-gpc-operational-completion-implementation-roadmap.md` と `20260423-gpc-operational-completion-test-roadmap.md` である
8. separate future scope `GPC` の current operational completion conditions は stand-alone complete root、GitHub publication branch root、tracked GitHub branch、GitHub release carrier、public checkout preparation の全 operational surface である
9. current GitHub publication executable は `publish_release_package_to_github.py` 1 本だけであり、obsolete repo-root publication script は retired 済みである
10. current accepted scope の next owner / task は `none / none` であり、native distributable に進めるなら別 scope が必要である

## 12. 6 Criteria / Closed Gate の current verdict

2026-04-12 JST 時点で、current accepted scope に対する 6 Criteria / Closed Gate はすべて `Strong Yes` である。

### 12.1 個別事案固定性

- 判定: `Strong Yes`
- 理由: current accepted closeout は `D7-RC1` family / contract / proof に限定され、concrete release value は residual detail として分離されている
- 崩れる条件: concrete release value を current accepted closeout 根拠へ混ぜた場合

### 12.2 fail-closed

- 判定: `Strong Yes`
- 理由: `release_preparation_failure` family は root invalidity と field invalidity を分離し、public payload を fixed message / no-raw-detail leakage に閉じている
- 崩れる条件: invalidity split を潰すか、raw detail を public surface に再露出させた場合

### 12.3 根拠の接続と範囲

- 判定: `Strong Yes`
- 理由: canonical、exact manifest、proof family、validation artifact、inventory sync の順で current truth が接続され、family-level 根拠で concrete value の成立まで主張していない
- 崩れる条件: family-level proof の根拠で concrete value の fixed completion まで主張した場合

### 12.4 構造・責務・意味論整合

- 判定: `Strong Yes`
- 理由: `cyr` single-entry、`BUNDLE_ROOT` authority、`CYRUNE_HOME` non-authority、D6 / D7 / D7-RC1 split は current truth として維持されている
- 崩れる条件: `D7-RC1` を D7 reopen に戻すか、`CYRUNE_HOME` を authority 化した場合

### 12.5 時間軸整合

- 判定: `Strong Yes`
- 理由: `D7-RC1 complete` は task-level proof と phase-end validation の採用後にのみ主張され、未完了 concrete value は future owner 側 residual detail として分離されている
- 崩れる条件: future owner の concrete value を retroactive に current complete claim へ戻した場合

### 12.6 未証明採用の不在

- 判定: `Strong Yes`
- 理由: current accepted closeout に採用しているのは adopted proof family と validation artifact だけであり、未実施の concrete value は採用していない
- 崩れる条件: proof / validation artifact を持たない項目を closeout 根拠へ採用した場合

## 13. initial findings が消滅した理由

`no findings` は、曖昧なまま観測を打ち切った結果ではない。  
初回 findings は、`D7-RC1` manifest wording と organization-owned canonical wording が `D7-RC1-D` closeout 後の current truth に完全追随していなかったことに限られていた。これらは closeout バッチ内で補正され、closeout report でも `補正後残存: 無し` として閉じている。  
したがって現在の `no findings` は、**初回 findings が current source に吸収・補正された結果**である。

## 14. 未完了だが正常なものの読み方

現在も concrete release value は残っているが、これは current accepted line の欠陥ではない。  
次の 5 件は、`release owner` が supply する別 owner の concrete value であり、current accepted source をまだ形成していないため、本文に採用していない。

1. concrete reverse-DNS bundle identifier
2. concrete installer emitted name
3. concrete upstream revision
4. concrete signing identity value
5. concrete notarization provider value

これらを「未完了だが正常」と呼んでよい理由は、次の 3 条件を満たすからである。

1. 今回の責務外である
2. owner が `release owner` で別である
3. current accepted closeout の成立主張に採用していない
