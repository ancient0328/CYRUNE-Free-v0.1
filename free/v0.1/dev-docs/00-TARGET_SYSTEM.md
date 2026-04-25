# 最終到達目標（CYRUNE Free v0.1 の"構築するもの"）

**作成日時 (JST)**: 2026-03-14

この文書は、`Distro/CYRUNE/free/v0.1/dev-docs/`（正典）を読んだ人が「CYRUNE Free v0.1 は最終的に何を作るのか」を迷わず理解できる状態を作るための**単一の定義**です。

---

## 0. CYRUNE Free v0.1 とは何か（他プロジェクトとの境界）

CYRUNE Free v0.1 は、CRANE-Kernel 上に載る**単一ユーザー向け mandatory boundary distribution**である。

- CRANE-Kernel が握るもの:
  - 3層メモリ、検索、埋め込み、忘却、昇格降格、計装の用途非依存契約
- CYRUNE Free が握るもの:
  - Control Plane
  - Working 10±2 の意味論
  - Policy / Gate
  - Citation-bound reasoning
  - Evidence Ledger
  - `cyr` を入口とする Runtime
- CYRUNE Free が握らないもの:
  - Pro / Pro+ / Enterprise / CITADEL の差分機能
  - Terminal emulator 自体の責務
  - Kernel 契約の再定義

---

## 1. 一文で言うと（Mission）

**CRANE-Kernel 上に、1ターンごとに context clear、Working 10±2、citation-bound、fail-closed、atomic ledger を強制する単一ユーザー向け Control OS を成立させる。**

この一文は Free v0.1 の製品核であり、`minimum completion gate` の中心を表す。
ただし Free v0.1 の最終ゴールはこの一文だけでは閉じず、ship goal まで含めて読む。

---

## 2. goal hierarchy

### 2.1 minimum completion gate

`minimum completion gate` は、Free v0.1 を「最初に成立」と呼ぶための前提条件である。
`02-decisions/20260314-cyrune-free-v0_1-minimum-completion-definition.md` がこの gate の単一ソースであり、current accepted scope の complete claim はこの gate を含む。

### 2.2 ship goal

Free v0.1 の ship goal は、`minimum completion gate` を満たした製品核を、第三者が取得・起動・理解できる release-grade product として成立させることである。
少なくとも、次を含む。

1. public entry から正しい取得先に辿れること
2. obtain / launch / first success の導線が単一であること
3. fail-closed のまま理解可能な公開面と recovery 導線を持つこと
4. public corpus が internal operational docs なしで self-consistent に読めること
5. public corpus の physical 実体が存在すること
6. 選択した ship channel に必要な release artifact が concrete に確定していること

### 2.3 現時点の読み方

current accepted scope の complete は、`minimum completion gate` と current accepted line / add-on scope / logical publication design lane の closeout を意味する。
これは ship goal complete を意味しない。
ship goal 側の差分は、新しい fixed problem / roadmap scope として切り出して解く。

---

## 3. 主要な用語の定義（このプロジェクト内での意味）

| 用語 | 定義 |
|------|------|
| Control Plane | Free の本体。1ターンの検証、拒否、証跡確定、Working 更新を握る層。 |
| Kernel adapter | CRANE 契約を実装する差し替え可能実装。ストレージ、インデックス、埋め込みなど。 |
| Execution adapter | Control Plane が Policy 通過後に呼び出す実行プラグイン。No-LLM、Local LLM、approved connector / executor を含む。 |
| Working projection | `working.json` に表現される現ターンの Working 集合。Kernel 契約の代替ではなく、CYRUNE の運用意味論の投影。 |

---

## 4. システム境界（System Boundary）

このバージョンで "提供するもの" は次です。

### 4.1 提供するもの（In Scope）

1. **Control Plane**
   - `cyr` を入口とし、context clear、Working 再構築、Policy pre-check、Execution、Citation validate、Fail-Closed、Ledger 確定までを強制する。
2. **三層メモリの Free 意味論**
   - Working 10±2、Processing 42日、Permanent 手動昇格を Free の意味論として固定する。
3. **Evidence Ledger**
   - append-only、atomic commit、Working 更新と証跡の紐付けを成立させる。
4. **最小 Runtime**
   - `cyr`、viewer、daemon 接続、Terminal への起動統合を提供する。
5. **No-LLM と単一 approved execution adapter**
   - 少なくとも 2 経路の実行を通し、どちらも Gate と Ledger を通過させる。
   - approved execution adapter の承認条件、公開面、pin 要件は `04-implementation-notes/EXECUTION_ADAPTER_APPROVAL_CANONICAL.md` を正とする。

### 4.2 提供しないもの（Out of Scope）

1. **上位ティア機能**
   - 推論差分、multi-agent、cross-model diff、consensus、組織統治、WORM / Airgap 強制は Free v0.1 の範囲外とする。
2. **Terminal emulator の本体実装**
   - WezTerm の責務を再実装しない。Terminal は投影層と配布統合に限定する。
3. **快適な長期運用完成度**
   - Permanent の快適運用、複数 policy pack、本格 connector 群、OS 強制 sandbox は後続へ送る。

---

## 5. 非目標（Non-goals）

このバージョンの範囲で、以下を「やらない」ものとして固定します。

- Free を IDE、チャットボット、SaaS として設計しない。
- Terminal polish を本体完成条件にしない。
- Kernel 契約へ Policy / Gate / Citation / Ledger を押し込まない。
- `Adapter` を曖昧語のまま運用しない。
- Pro 以上の価値を Free の成立条件へ混入させない。

---

## 6. 受け入れ条件（合否の単一ソース）

CYRUNE Free v0.1 の合否は階層で読む。

1. `minimum completion gate`:
   `01-roadmap/ROADMAP.md` の工程順チェックリストと `02-decisions/20260314-cyrune-free-v0_1-minimum-completion-definition.md` の Completion Definition を単一ソースとし、完了根拠は `90-reports/` の実測ログとして残す。
2. `ship goal`:
   本書、`02-decisions/20260412-free-v0_1-goal-hierarchy-and-ship-goal.md`、`02-decisions/20260413-ship-goal-master-scope-and-boundary.md` を基準に、ship-grade user journey、physical publication、selected ship channel の release artifact を別 scope として閉じる。

現時点で current accepted source により close 済みなのは `minimum completion gate`、current accepted scope、ship goal の 3 つである。

- ロードマップ: `01-roadmap/ROADMAP.md`
- Completion Definition: `02-decisions/20260314-cyrune-free-v0_1-minimum-completion-definition.md`
- Goal hierarchy / ship goal: `02-decisions/20260412-free-v0_1-goal-hierarchy-and-ship-goal.md`
- 実測ログ: `90-reports/`
