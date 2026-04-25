# CYRUNE Pro+ Canonical

**Status**: Canonical（Pro+）
**Scope**: 異種LLM multi-agent / Cross-Model Diff / Consensus Engine / Stability & Drift

---

# 1) 定義

> Pro+ は、異種LLM間で推論を構造的に比較・合議し、推論安定性を保証する層である。

Proは単一LLM内部での構造検査である。
Pro+は複数LLM間の構造比較と合議確定を行う。

---

# 2) Proとの差異（決定的定義）

| 項目 | Pro | Pro+ |
|------|-----|------|
| 時間軸推論差分（D1〜D5） | ✔ | ✔ |
| Single-LLM Multi-Agent | ✔ | ✔ |
| 異種LLM実行 | ✖ | ✔ |
| Cross-Model Diff | ✖ | ✔ |
| Consensus Engine | ✖ | ✔ |
| Stability Score | ✖ | ✔ |
| Model Drift Detection | ✖ | ✔ |
| Conflict Auto-Block | ✖ | ✔ |

> Proは「検査」
> Pro+は「合議」

---

# 3) 実行単位（Run Group Model）

Pro+では複数モデル実行を1つの単位で束ねる。

## 3.1 Task Group Schema

```json
{
  "task_id": "T-20260301-001",
  "run_group_id": "RG-0001",
  "models": ["claude-code", "codex"],
  "citation_bundle_id": "CB-001",
  "policy_pack_id": "default"
}
````

* 各モデルの実行は独立Evidenceとして保存される
* run_group_idで束ねる

---

# 4) Cross-Model Diff（空間軸Diff）

ProのDiff Taxonomy（D1〜D5）をモデル間比較に拡張する。

## 4.1 CM-Diff Taxonomy（閉集合）

1. CM-D1: Citation Divergence
2. CM-D2: Claim Divergence
3. CM-D3: Dependency Graph Divergence
4. CM-D4: Policy Event Divergence
5. CM-D5: Working Impact Divergence

文章diffは禁止。

---

# 5) Consensus Engine（合議確定機構）

Pro+の中核。

## 5.1 入力

* RR_A（Model A）
* RR_B（Model B）
* Citation Bundle
* Policy Pack
* Working

## 5.2 合議ルール（v0.1固定）

1. 両者一致Claimは採用
2. citation一致率が高い側を優先
3. Policy違反を含む側は棄却
4. Claim divergenceが重大な場合 → severity = warn
5. citation無し主張が存在 → severity = block

## 5.3 出力

```json
{
  "consensus_rr": {...},
  "divergence_report": {...},
  "severity": "info|warn|block"
}
```

---

# 6) Inference Stability Score

```
stability = 1 - divergence_ratio
```

divergence_ratio は：

* Claim差分
* Citation差分
* Dependency差分

を重み付きで算出。

## 6.1 出力例

```json
{
  "stability_score": 0.82,
  "major_divergence": ["claim:c9"],
  "severity": "warn"
}
```

---

# 7) Conflict Escalation Mode

以下条件で auto-block：

* CM-D2 重大 divergence
* CM-D1 divergence > threshold
* Policy結果がモデル間で矛盾

BlockはLedgerに保存される。

---

# 8) Model Drift Detection（時間×空間拡張）

同一task_idを再実行。

比較対象：

* 旧 run_group
* 新 run_group
* model hash
* model version

## 8.1 Drift Record

```json
{
  "drift_detected": true,
  "dimension": "claim",
  "model_id": "claude-code",
  "previous_hash": "...",
  "current_hash": "..."
}
```

---

# 9) Ledger拡張

Evidence構造に追加：

```json
{
  "run_group_id": "RG-0001",
  "model_results": [
    {"model": "claude-code", "evidence_id": "EVID-120"},
    {"model": "codex", "evidence_id": "EVID-121"}
  ],
  "consensus_evidence": "EVID-122",
  "stability_score": 0.82
}
```

---

# 10) Fail-Closed条件（Pro+専用）

以下は合議失敗としてBlock：

* RR構造化不能
* citation divergence > threshold
* consensus生成不能
* driftが重大かつ説明不能
* atomic確定失敗

---

# 11) 実行CLI仕様

APIは使用しない。CLIアダプタとして実行。

```
cyr proplus run --models claude-code,codex --task T-001
```

内部処理：

1. citation bundle生成
2. 並列実行
3. RR生成
4. Cross-model diff
5. Consensus engine
6. Ledger atomic確定

---

# 12) 価値定義

Pro+は：

* 推論の安定性を定量化する
* モデル依存性を可視化する
* 合議不能時に停止する

これは単なる「モデル追加」ではない。

> 推論の構造的安定性保証層である。
