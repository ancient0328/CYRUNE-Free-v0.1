# CYRUNE Free v0.1 Standalone Summary Corpus

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. この corpus の役割

この `summary/` corpus は、CYRUNE Free v0.1 の**現在成立している内容**を、第三者がゼロから理解できるようにまとめた standalone 説明書である。

この corpus の目的は次の 3 つである。

1. CYRUNE Free v0.1 が何であり、何を成立させたのかを一読で理解できるようにする
2. 実装・運用・検証・拡張の判断に必要な構造、責務、意味論、fail-closed 条件を本文だけで把握できるようにする
3. 新しい人間または AI agent が、外部文書を先に読まずに current accepted state を理解できるようにする

この corpus は、現時点で成立している内容だけを扱う。
未採用 detail、将来 scope、未固定の組織依存値は本文に混ぜない。

## 2. 読み方

最短で全体像を掴む順序は次のとおりである。

1. 本巻
2. 巻 1: 製品定義、境界、完成条件
3. 巻 2: アーキテクチャ、authority、runtime line
4. 巻 3: 契約、データモデル、fail-closed 規則
5. 巻 7: 現在状態、運用上の判断、次 scope の切り方

必要に応じて次を読む。

- 巻 4: どの execution line が何を確立したか
- 巻 5: 実装の物理配置
- 巻 6: 何が証明済みで、失敗時にどう切り分けるか

## 3. 現時点の一文要約

CYRUNE Free v0.1 は、CRANE-Kernel 上に構築された単一ユーザー向け Control OS であり、`cyr` 単一入口、Control Plane 主導、Working 10±2、citation-bound reasoning、fail-closed、append-only atomic evidence ledger、packaged bundle-root authority、D6 native outer launcher、D7 terminal bundle productization まで current accepted scope として成立している。
ただしこれは `minimum completion gate` と current accepted scope 側の成立を意味し、ship goal complete を意味しない。

## 4. 現在成立しているもの

現時点で成立済みとして扱うものは次である。

1. Free v0.1 core
2. fixed problem corrective line
3. D5 packaged mode baseline
4. D6 native outer launcher line
5. D7 terminal bundle productization line

これらは、文書だけでなく、実装、proof family、workspace validation、current-state summary 同期まで current accepted scope で閉じている。

## 5. 現在成立していないもの

現時点で成立主張に含めていないものは次である。

1. 上位 tier 機能
2. 組織依存の release concretization detail
3. D7 で intentionally 未採用の concrete external release value
4. storage open/init failure surface 一般の no-leakage

これらは current blocker ではない。
今回の成立主張に採用していないだけである。

## 6. 8 分冊の責務

| 巻 | 役割 |
|----|------|
| 0 | corpus の入口、読み順、現在状態の圧縮要約 |
| 1 | CYRUNE Free v0.1 が何であり、何を握り、何を握らないか |
| 2 | 層構造、authority graph、runtime line、execution family |
| 3 | 契約、データモデル、fail-closed 条件、禁止事項 |
| 4 | 現在成立している execution line ごとの役割と完成範囲 |
| 5 | 実装の物理配置、crate / binary / script / artifact の責務 |
| 6 | 証明済み範囲、証明の意味、失敗時の切り分け |
| 7 | current accepted state、運用判断、変更時の進め方 |

## 7. 本 corpus で扱う主語

この corpus では、主語を次のように固定する。

1. **CYRUNE Free v0.1**
   - 製品と current accepted scope の主語
2. **Control Plane**
   - turn 実行、Gate、Citation、Ledger、Working rebuild の主語
3. **Runtime**
   - `cyr`、daemon、view、pack、terminal integration の主語
4. **BUNDLE_ROOT**
   - packaged static resource の唯一の authority root
5. **CYRUNE_HOME**
   - mutable state、generated path、materialized projection の root
6. **D6**
   - native outer launcher line の主語
7. **D7**
   - terminal bundle productization line の主語

## 8. 何を「理解した」とみなすか

第三者が次を説明できれば、この corpus は目的を果たしている。

1. CYRUNE Free v0.1 の mission と boundary
2. Control Plane、Runtime、Kernel、Execution Adapter の責務分離
3. Working / Processing / Permanent の意味論
4. Policy Gate、Citation Bundle、Evidence Ledger の必須規則
5. packaged mode の authority root と fail-closed split
6. D6 と D7 が core semantics を変えずに何を追加したか
7. 現在の complete / out-of-scope / non-blocker の境界
8. 新しい scope を追加する時に何を先に固定すべきか

## 9. 本 corpus の制限

この corpus は code walkthrough を主目的にしない。
ただし、実装判断に必要な物理配置と責務分離は説明する。

また、この corpus は legacy の全履歴を保存する歴史書ではない。
過去の経路のうち、現在の成立内容を理解するために必要なものだけを残す。

## 10. 現在の簡易結論

- CYRUNE Free v0.1 は current accepted scope で成立している
- ship goal complete は current source として成立している
- ship-goal-side parent implementation lane と parent validation lane は complete
- current accepted scope 側の blocker は無い
- next owner / task は `none / none`
- obsolete repo-root GitHub publication script は retired 済みであり、stale blocker report は archive 済みである
- public package supplementary docs `docs/ENGINEERING_SPEC.md` と `docs/USER_GUIDE.md` は role-fulfillment closeout 済みであり、`CYRUNE_Free_Public_Index.md` の supplementary reading list から到達できる
- publication design lane は `PB-A-PB-D` まで完了し、public-ready logical closeout まで current source に同期されている
- ship goal whole-roadmap は parent `SGM / SGT-M` と child `SGI / SGT` の二層で固定されている
- native distributable と concrete release owner value finalization は本 lane に含めない

## 11. 最新の 6 Gate 再評価

2026-04-12 JST 時点の再評価では、current accepted scope に対する 6 Criteria / Closed Gate はすべて `Strong Yes` である。

1. 個別事案固定性
2. fail-closed
3. 根拠の接続と範囲
4. 構造・責務・意味論整合
5. 時間軸整合
6. 未証明採用の不在

この判定は、「曖昧なまま no findings にした」ものではない。
初回に残っていた local findings は、child roadmap pair を ship goal whole-roadmap の代替として読めてしまう点と、master fixed problem family が未定義だった点に限られていた。これらは master ADR / canonical / roadmap 追加により補正済みであり、その後の all-agent brush-up cycle でも parent global sync ownership、child adoption boundary、dual authority、residual 6 項目 comparator、current-state wording の残差が解消した。さらに 2026-04-14 JST の parent redesign と `SGM-D` actual publication execution により selected ship channel は `GitHub public release package channel` として materialize され、2026-04-15 JST の `SGTMA-T1` parent scope / topology consistency proof、`SGTMA-T2` child lane non-substitution proof、child `SGT-A`-`SGT-D` validation proof set、parent `SGT-MC` selected ship channel proof、`SGT-MD` ship-goal parent closeout により parent ADR、master scope canonical、master phase family canonical、master roadmaps、child pair、index / summary / roadmap readme、publication unit manifest、authority surface contract、selected ship channel remote state、final closeout wording が一致し、ship goal complete と `next owner / task = none / none` が current-source で一致した。child `SGI / SGT` pair は `MGF-2 public corpus publication-unit materialization` の child lane としてのみ読まれる。

6 Gate の強い成立を支える読み方は次である。

1. `D7-RC1` closeout は family / contract / proof を採用し、concrete release value は採用していない
2. fail-closed family は `release_preparation_failure` split と no-raw-detail leakage に閉じている
3. 根拠は canonical、exact manifest、proof family、workspace validation、inventory sync に順序立てて接続している
4. `cyr` single-entry、`BUNDLE_ROOT` authority、`CYRUNE_HOME` non-authority、D6 / D7 / D7-RC1 の責務分離は崩れていない
5. residual detail は future owner に残し、current complete claim に逆流させていない
6. 未証明の concrete value を closeout 根拠へ混ぜていない

この巻だけ読めば、現時点で「current accepted scope は成立している」「ship goal complete は current source として成立している」「ship-goal-side implementation / validation lane は complete であり、next owner / task は `none / none` である」「既存 line の内部を直す段階ではない」ことが分かる。
