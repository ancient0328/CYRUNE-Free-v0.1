# CYRUNE Free v0.1: System And Scope

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. 製品の定義

CYRUNE Free v0.1 は、CRANE-Kernel 上に成立する**単一ユーザー向け mandatory boundary distribution**である。
単なる CLI でも、単なる terminal skin でも、単なる LLM app でもない。
本体は Control Plane であり、ユーザー入力、Working 再構築、Policy、Execution、Citation、Ledger を turn 単位で強制する。

## 2. Mission

CYRUNE Free v0.1 の mission は、次の一文で表せる。

**CRANE-Kernel 上に、1 ターンごとに context clear、Working 10±2、citation-bound、fail-closed、atomic ledger を強制する単一ユーザー向け Control OS を成立させる。**

この mission は Free v0.1 の製品核であり、`minimum completion gate` の中心である。
ただし v0.1 の ship goal は、これを満たしたうえで、第三者が obtain / launch / first success まで進め、public corpus だけで製品の主語と current public truth を理解できる release-grade product を成立させることまで含む。
したがって current accepted scope complete は ship goal complete を意味しない。

## 3. Free v0.1 が握るもの

Free v0.1 が自前で握る責務は次である。

1. Control Plane
2. Working 10±2 の運用意味論
3. Policy / Gate
4. Citation-bound reasoning
5. Evidence Ledger
6. `cyr` を入口とする Runtime
7. approved execution adapter の許可と実行制御
8. packaged mode の authority / fail-closed split
9. D6 native outer launcher line
10. D7 terminal bundle productization line

## 4. Free v0.1 が握らないもの

Free v0.1 が握らない責務は次である。

1. CRANE-Kernel の用途非依存契約そのもの
2. Pro / Pro+ / Enterprise / CITADEL の差分機能
3. terminal emulator 自体の意味論
4. WezTerm 本体の端末責務
5. 組織依存の signing identity や notarization provider
6. multi-agent、consensus、cross-model diff
7. Desktop polish や UI 演出

## 5. 主要用語

### 5.1 Control Plane

1 ターンの accept / reject を最終決定する層である。
request 検証、binding 解決、Working rebuild、Gate、Execution、Citation validate、Ledger commit、Working 反映を握る。

### 5.2 Kernel adapter

CRANE-Kernel 契約を実装する差し替え可能な adapter 群である。
ストレージ、インデックス、埋め込みの実体を担う。
Free 独自の意味論は持たない。

### 5.3 Execution adapter

Control Plane に許可されたときだけ呼ばれる実行差分である。
No-LLM path と approved execution adapter path が current accepted scope である。

### 5.4 Working projection

`working.json` に現れる、そのターンで判断に使ってよい小さな作業集合である。
三層メモリの source of truth ではなく、Control Plane が確定した投影である。

## 6. In Scope

現在成立済みの In Scope は次である。

1. Control Plane 主導の単一 turn 実行モデル
2. Working / Processing / Permanent の三層意味論
3. Evidence Ledger による append-only / atomic な証跡化
4. `cyr` 単一入口の Runtime
5. No-LLM と単一 approved execution adapter の両 accepted path
6. deny-by-default capability Gate
7. citation-bound reasoning と uncited claim reject
8. packaged mode の single immutable bundle authority
9. D6 native outer launcher
10. D7 terminal bundle productization

## 7. Out Of Scope

現在の成立主張に含めないものは次である。

1. 上位 tier の差分機能
2. Free を IDE や SaaS として再定義すること
3. terminal emulator 本体の再実装
4. connector 群の大規模拡充
5. 複数 policy pack の本格運用
6. OS 強制 sandbox の高度化
7. D7 の未採用 concrete external release detail

## 8. Non-goals

Free v0.1 は次を目標にしない。

1. UI の見た目の完成
2. 長期運用に最適化された Permanent の快適性
3. 何でも実行できる convenience-first runtime
4. terminal 側への統制ロジック移植
5. fallback や best-effort を多用した fail-open 設計

## 9. 成立条件

Free v0.1 を「成立」と呼ぶ条件は、現在の accepted scope 上では次の 7 群である。

1. 境界と言葉が固定されている
2. 単一実行モデルが固定されている
3. Working 契約が固定されている
4. 三層メモリの意味論と既定 binding が固定されている
5. Policy / Gate / Citation の最小契約が固定されている
6. Evidence Ledger の最小契約が固定されている
7. 最低 2 経路の実行が成立している

現在はこれに加えて、corrective line、D6、D7 まで current accepted line として完了している。

## 10. 価値の中心

CYRUNE Free v0.1 の価値の中心は Control Plane である。
価値は次の性質の組み合わせにある。

1. turn ごとの context clear
2. Working 10±2 の小さな判断境界
3. deny-by-default Gate
4. citation-bound reasoning
5. append-only atomic Ledger
6. single-entry runtime
7. packaged mode でも崩れない fail-closed

## 11. Free の成立後に追加された line

### 11.1 corrective line

fixed problem 6 項目を、code / proof / inventory / gate / final closeout の範囲で解消した。
これにより `blocker none`、`shipping memory line fully complete`、`baseline fully closed` が current accepted claim として再許可された。

### 11.2 D6

native outer launcher を outer front として追加した。
ただし `cyr` 単一入口、`BUNDLE_ROOT` authority、`CYRUNE_HOME` non-authority projection、launcher / preflight / run-path split を崩していない。

### 11.3 D7

terminal bundle productization を追加した。
ただし D7 は runtime semantics を変えず、bundle identity、notice、integrity、upstream intake judgment、productization failure family だけを own している。

## 12. 現時点の製品状態

現在の製品状態は次のとおりである。

1. current active phase / task は `none / none`
2. current blocker は無い
3. current accepted next executable scope は `none` である
4. core、corrective、D6、D7、および `D7-RC1` add-on scope が current accepted source として成立している
5. ship goal complete は current source として成立している

## 13. 今後の変更原則

Free v0.1 に対して新 scope を追加する場合は、次を守らなければならない。

1. 既存 line を reopen しない
2. 新しい canonical で主語と非主語を固定する
3. roadmap で phase / task / gate を分解する
4. exact test / proof manifest を用意する
5. accepted family と fail-closed family を明示する
6. final closeout 前に inventory と gate を current line へ同期する
