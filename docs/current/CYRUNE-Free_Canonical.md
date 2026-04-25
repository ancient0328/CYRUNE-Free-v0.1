# CYRUNE Free Canonical

**Status**: Canonical（Free）
**Scope**: Single-user Control OS Core
**Price**: $0

**Public Free v0.1 scope note**: この public alpha は Free v0.1 の first-success execution surface を公開する。classification / MAC は CYRUNE の canonical design concept として扱うが、この公開面は enforcement-complete classification lattice / clearance enforcement を主張しない。

---

# 1) 定義

> CYRUNE Free は、AI実行前に必ず通過する強制境界層を提供する単一ユーザー向けControl OSである。

Freeは機能削減版ではない。
思想と構造の最小単位である。

---

# 2) 不変原則（削除不可）

以下はFreeに常に含まれる：

* 三層メモリ（Working / Processing / Permanent）
* Working 10±2 強制
* context clear per turn
* Fail-closed Gate
* Citation-bound reasoning
* Evidence Ledger（append-only, atomic）
* Detachable LLM architecture
* No-LLM mode
* CLI canonical interface

Freeからこれらを削除してはならない。

---

# 3) Control Plane（最小構成）

Freeの中核はControl Planeである。

## 3.1 強制フロー（1ターン）

```

1. User input
2. Context clear
3. Working再構築
4. Classification boundary（target/design scope; public alpha does not claim enforcement-complete classification / MAC）
5. Policy pre-check
6. LLM（またはNo-LLM）実行
7. Citation validation
8. Fail-closed検証
9. Ledger atomic確定
10. Output

```

LLMは必ずControl Planeを通過する。

---

# 4) Working（10±2契約）

## 4.1 制約

* 最大12スロット（10±2）
* slot追加はsource_evidence必須
* 未紐付け更新は禁止
* 上限超過はfail-closed

## 4.2 slot種別（閉集合）

* decision
* constraint
* assumption
* todo
* definition
* context
* command

---

# 5) 三層メモリ契約

FreeはCRANEの三層契約を利用する。

* Working：即時判断材料
* Processing：中期保持（42日）
* Permanent：長期保存

Freeは実装詳細を固定しないが、
契約（容量・昇格・忘却）を守る。

---

# 6) Gate契約

## 6.1 Fail-Closed原則

以下の場合は出力を拒否する：

* 出典なき主張
* 未分類データ（target/design scope; public alpha does not claim executable classification / MAC rejection）
* Policy違反
* 未定義Capability使用
* Working未整合

検証不能 = 実行不能

---

# 7) Citation契約

* すべての主張はCitation Bundleに基づく
* uncited claimは禁止
* derivedはcitation範囲内に限定

モデルは権威ではない。

---

# 8) Ledger契約

## 8.1 Atomic確定

1. tmp生成
2. fsync
3. rename
4. index更新

確定不能はRun自体を失敗させる。

## 8.2 保存対象

* Query
* Citation Bundle ID
* Working hash
* Policy Pack ID
* Model ID
* RR（構造化推論記録）

Ledgerはログではなく証跡である。

---

# 9) No-LLM Mode

FreeはLLMを必須としない。

No-LLMでは：

* Retrievalのみ
* Extractive summaryのみ
* Gate + Ledgerは有効

LLMはdetachableである。

---

# 10) Capability Gating

Default = deny

Capability例：

* fs_read
* fs_write
* git_write
* net
* ci
* browser

Policy Packで明示許可されたもののみ使用可能。

---

# 11) Fail-Closed条件（Free専用）

以下は即時Block：

* Citation bundle未生成
* Working不整合
* Policy不整合
* atomic書き込み失敗
* 未定義Capability呼び出し

---

# 12) Freeの本質

Freeは：

* LLMラッパーではない
* IDEではない
* SaaSではない

Freeは：

> AI実行前に必ず通る強制境界層

である。

---

# 13) Proとの差異

Freeは：

* 推論差分を生成しない
* multi-agentを持たない
* 異種LLM実行を持たない
* CI Gateを持たない
* 組織統治を持たない

Freeは「制御の最小単位」である。

---

# 14) 戦略的意義

Freeは思想の武器である。

* 制御は無料で提供する
* 統治と合議は有料で提供する

制御を削ることは、CYRUNEの存在意義を削ることになる。

---

# 15) 最終定義

CYRUNE Free は、

AI能力を高めるための製品ではない。

AI実行を構造的に制御するための
最小構造Control OSである。
