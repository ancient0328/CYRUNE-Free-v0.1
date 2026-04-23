# CYRUNE Free v0.1: Evidence And Troubleshooting

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. この巻の役割

この巻は、現在何が証明済みであるか、どこまでを current accepted claim に採用しているか、失敗時にどう切り分けるかを standalone で説明する。

## 2. 証明の考え方

CYRUNE Free v0.1 では、成立主張は「動いた気がする」ではなく、再現可能な accepted / fail-closed / validation family で固定する。

証明の型は次の 3 つである。

1. **accepted family**
   - intended behavior が成立していること
2. **fail-closed family**
   - intended failure が silent success にならず、適切な surface に閉じること
3. **validation family**
   - fmt / clippy / build / test などの phase-end 検証が clean であること

## 3. current accepted proof coverage

### 3.1 core line

core line で証明済みなのは次である。

1. No-LLM accepted run
2. approved execution adapter accepted run
3. deny-by-default reject
4. binding resolution
5. Working limit reject
6. citation deny reject
7. ledger atomic commit

### 3.2 corrective line

corrective line で証明済みなのは次である。

1. shipping exact pin positive / negative proof
2. retention / ttl predicate-separated closure
3. `view policy --pack` routing correctness
4. reject surface normalization と no-leakage
5. evidence boundary closure
6. final reclose と D5 packaged alignment

### 3.3 D6

D6 line で証明済みなのは次である。

1. launcher accepted handoff
2. launcher-owned accepted family
3. preflight-invalid-override fail-closed
4. launcher-missing-terminal fail-closed
5. run-path-unresolved fail-closed
6. D6 workspace validation clean

### 3.4 D7

D7 line で証明済みなのは次である。

1. bundle identity conduit
2. notice / license / SBOM conduit
3. integrity / signature conduit
4. D5 / D6 inheritance guard
5. upstream intake judgment
6. productization failure surface
7. D7 proof-driver family
8. D7 workspace validation clean

## 4. current accepted closeout claim の範囲

現在の closeout claim に含めてよいのは次である。

1. core semantics が成立している
2. fixed problem 6 項目が corrective line で解消済み
3. D6 line が complete
4. D7 line が complete
5. active blocker が無い

現在の closeout claim に含めてはならないのは次である。

1. D7 の未採用 concrete external release detail
2. storage open/init failure surface 一般の no-leakage
3. future scope に属する release concretization detail

## 5. 失敗時の切り分け原則

失敗を見た時は、まず次の 4 軸で分類する。

1. **run-path failure か**
2. **preflight failure か**
3. **launcher failure か**
4. **productization failure か**

これに core reject family を加える。

5. `policy_denied`
6. `binding_unresolved`
7. `working_invalid`
8. `citation_denied`
9. `ledger_commit_failed`
10. `working_update_failed`

## 6. 代表的な failure と意味

### 6.1 `policy_denied`

要求 capability が pack で許可されていない、または capability family が不正である。  
execution 前に reject されるべきである。

### 6.2 `binding_unresolved`

binding または required resource が解決できない。  
packaged mode では bundle-root authority failure もここへ写像されうる。

### 6.3 `working_invalid`

Working limit 超過、required evidence 不足、deterministic rebuild failure など、Working 契約違反がある。

### 6.4 `citation_denied`

uncited claim または citation scope 超過がある。

### 6.5 `ledger_commit_failed`

Evidence の atomic commit が成立しない。  
この場合 accepted output を返してはならない。

### 6.6 `working_update_failed`

ledger commit 後に `working.json` 更新が失敗した。  
この場合も accepted output を返してはならない。

### 6.7 preflight failure

doctor / launch prerequisite が満たせない。  
RunRejected に丸めず、preflight surface で停止する。

### 6.8 launcher failure

outer launcher 側の統合失敗である。  
run-path reject や productization failure と混線させない。

### 6.9 productization failure

bundle assemble、branding resource、notice / SBOM / signature、upstream drift など、productization owner の failure である。  
runtime accepted path と混線させない。

## 7. troubleshooting 手順

### 7.1 症状が「実行できない」場合

1. request が Runtime に入っているか確認する
2. run-path reject か preflight failure かを分ける
3. binding / bundle resource / authority root の不整合がないか確認する
4. Working / Policy / Citation のどこで止まったかを確認する
5. ledger commit と working update まで到達しているか確認する

### 7.2 症状が「packaged mode だけ壊れる」場合

1. `BUNDLE_ROOT` 解決が一意か
2. explicit whole-root override だけを使っているか
3. home 側 copy を authority として読んでいないか
4. doctor / launch / run-path split が崩れていないか

### 7.3 症状が「D6 で壊れる」場合

1. launcher が `cyr` を bypass していないか
2. launcher failure を run-path reject に丸めていないか
3. outer launcher が authority root を変更していないか
4. D7 の productization family を D6 に混ぜていないか

### 7.4 症状が「D7 で壊れる」場合

1. bundle identity / notice / integrity / upstream intake のどの family かを分ける
2. productization failure を run-path / preflight / launcher に丸めていないか
3. unsigned / notice 欠落 / metadata invalid を success にしていないか
4. D5 / D6 inheritance guard が崩れていないか

## 8. 何を根拠に complete と言ってよいか

ある line を complete と言ってよいのは、少なくとも次が揃った時だけである。

1. accepted family
2. fail-closed family
3. validation family
4. docs / index / inventory sync
5. Closed Gate Report

どれか 1 つでも欠ければ complete claim を出してはならない。

## 9. current accepted failures の読み方

fail-closed artifact は「壊れている証拠」ではない。  
期待した失敗面に適切に閉じることを示す証拠である。  
したがって、failure artifact の存在は current accepted line の否定ではなく、fail-closed 成立の一部である。

## 10. current evidence conclusion

現在の accepted claim を支える証明は次である。

1. core accepted / reject family
2. corrective accepted / fail-closed / final reclose family
3. D6 accepted / fail-closed / validation family
4. D7 accepted / fail-closed / validation family

この 4 群が揃っているため、current active blocker は無いと判断してよい。

### 10.1 public package supplementary docs closeout

`ENGINEERING_SPEC.md` と `USER_GUIDE.md` の role-fulfillment closeout は、current accepted proof coverage 4 群の一部ではない。  
これは accepted / fail-closed / validation family ではなく、public package supplementary docs が engineer-facing detailed spec と general-user usage manual として十分成立するよう補正され、`CYRUNE_Free_Public_Index.md` の supplementary reading list、source/public exact mapping、Closed Gate Report まで揃ったことを固定する docs closeout root である。

### 10.1.1 obsolete repo-root publication script retirement closeout

obsolete `publish_public_corpus_to_github.py` の retirement closeout も current accepted proof coverage 4 群の一部ではない。  
これは accepted / fail-closed / validation family ではなく、release package channel redesign 後の current live publication artifact singularity を回復する maintenance closeout root であり、obsolete executable の source/public 除去、old-script rerun 手順を含む stale blocker report の archive、current live publication artifact の `publish_release_package_to_github.py` 1 本化を固定する。

### 10.2 `SGTMA-T1` parent scope / topology consistency proof

`SGTMA-T1` closeout も current accepted proof coverage 4 群の一部ではない。  
これは ship-goal-side parent validation lane に属する proof root であり、parent ADR、master scope canonical、master phase family canonical が selected ship channel、master fixed problem family、child mapping の 3 点で一致していることを固定する。`ship goal complete` や child validation proof set adoption は、この proof 単独ではまだ主張しない。

### 10.3 `SGTMA-T2` child lane non-substitution proof

`SGTMA-T2` closeout も current accepted proof coverage 4 群の一部ではない。  
これは ship-goal-side parent validation lane に属する proof root であり、child `SGI / SGT` pair が `MGF-2 public corpus publication-unit materialization` の child lane としてだけ参照され、ship goal whole-roadmap の代替として扱われていないことを固定する。`ship goal complete` や child validation proof set adoption は、この proof 単独ではまだ主張しない。

### 10.4 child `SGT-A` publication unit exactness proof

child `SGT-A` closeout も current accepted proof coverage 4 群の一部ではない。  
これは `MGF-2` validation child lane に属する proof root であり、publication unit root 一意性、then-current execution canonical 準拠 manifest、non-adopted family 非混入、stale purge を固定する。child validation lane complete や ship goal complete は、この proof 単独ではまだ主張しない。

### 10.5 child `SGT-B` public authority surface proof

child `SGT-B` closeout も current accepted proof coverage 4 群の一部ではない。  
これは `MGF-2` validation child lane に属する proof root であり、authority surface 4 文書の exact body template / heading set / direct-link set 一致と、historical / deferred-publication family の plain text mention only / non-navigable exposure を固定する。child validation lane complete や ship goal complete は、この proof 単独ではまだ主張しない。

### 10.6 child `SGT-C` ship-grade user journey proof

child `SGT-C` closeout も current accepted proof coverage 4 群の一部ではない。  
これは `MGF-2` validation child lane に属する proof root であり、public package `prepare-public-run.sh`、`doctor.sh`、`first-success.sh` の accepted / fail-closed contract、success metrics、failure fixture を固定する。child validation lane complete や ship goal complete は、この proof 単独ではまだ主張しない。

### 10.7 child `SGT-D` publication validation closeout

child `SGT-D` closeout も current accepted proof coverage 4 群の一部ではない。  
これは `MGF-2` validation child lane に属する proof root であり、current publication-unit exactness comparator `expected = 129 / actual = 129 / missing = 0 / extra = 0 / byte mismatch = 0 / mode mismatch = 0`、authority-surface comparator、runtime comparator を通した child close root を固定する。child validation lane complete はこの root で成立するが、ship goal complete はこの root 単独ではまだ主張しない。

### 10.8 `SGT-MC` GitHub release package publication execution proof

`SGT-MC` closeout は current accepted proof coverage 4 群の一部ではない。  
これは ship-goal-side parent validation lane に属する proof root であり、selected ship channel repository metadata、release tag / asset identity、repository root discovery / docs surface topology、third-party obtain root reachability を固定する。ship goal complete は、この proof 単独ではまだ主張しない。

### 10.9 `SGT-MD` ship-goal parent closeout

`SGT-MD` closeout は current accepted proof coverage 4 群の一部ではない。  
これは ship-goal-side parent validation / closeout lane に属する proof root であり、master scope canonical、master phase-family canonical、GitHub publication topology / execution canonical、child proof set、selected ship channel proof set、current-state docs を入力とした parent closeout comparator pass と `ship goal complete / next owner = none` 同期を固定する。

### 10.10 `GPCT-C` GitHub publication execution proof

`GPCT-C` closeout は current accepted proof coverage 4 群の一部ではない。  
これは `2026-04-15/16` broader `GPC` validation lane に属する historical proof root であり、remote tracked publication branch exactness、release `v0.1` / exact asset `cyrune-free-v0.1.tar.gz` / exact asset URL、public branch scripts の obtainability / launchability / health、carrier-aware docs / auxiliary non-substitution を固定する。current narrow-purpose roadmap authority には採用しない。

### 10.11 `GPCT-D` fixed problem closeout

`GPCT-D` closeout も current accepted proof coverage 4 群の一部ではない。  
これは `2026-04-15/16` broader `GPC` validation / closeout lane に属する historical proof root であり、split semantics、carrier semantics、tracked / non-tracked boundary、implementation-state docs、historical bridge を入力とした comparator pass と closeout wording を固定する。current narrow-purpose roadmap authority には採用しない。

## 11. 2026-04-12 JST 時点の 6 Gate 再評価

current accepted scope に対する最新の 6 Criteria / Closed Gate 判定は、すべて `Strong Yes` である。

1. 個別事案固定性
2. fail-closed
3. 根拠の接続と範囲
4. 構造・責務・意味論整合
5. 時間軸整合
6. 未証明採用の不在

この判定は、何も見ずに `no findings` としたものではない。  
初回に残っていた findings は、`D7-RC1` manifest wording と organization-owned canonical wording が current truth に完全追随していなかった点に限られていた。これらは closeout 前に補正され、現在は closeout report、inventory、operational guide まで整合している。

この 6 Gate を支える current evidence の読み方は次である。

1. `D7-RC1` closeout は `D7RC1B-T2` と `D7RC1C-T1` の実測 proof family、workspace validation、docs sync だけを採用している
2. rule-fixed family と organization-owned family は、どちらも `release_preparation_failure` split と no-raw-detail leakage を満たしている
3. concrete reverse-DNS value、concrete installer / archive filename、concrete upstream revision、concrete signing identity value、concrete notarization provider value は residual detail として本文不採用である
4. residual detail は `release owner` が supply する別 owner の concrete value であり、current accepted source をまだ形成していない

したがって、この巻における `証明済み` とは、concrete release value まで固定したという意味ではなく、**family / contract / proof / validation の current accepted closeout が成立している**という意味である。
