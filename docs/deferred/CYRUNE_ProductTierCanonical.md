# CYRUNE Product Tier Canonical

**Status**: Canonical
**Date**: 2026-03-01
**Scope**: Free / Pro / Pro+ / Enterprise / CITADEL の構造的差分確定

---

# 1. 全体構造（レイヤー関係）

```
CRANE (OSS Kernel)
        ↓
CYRUNE Free
        ↓
CYRUNE Pro
        ↓
CYRUNE Pro+
        ↓
CYRUNE Enterprise
        ↓
CITADEL
```

                    ┌──────────────────────────┐
                    │         CRANE            │
                    │  Contract Kernel (OSS)   │
                    │  - Memory contract       │
                    │  - Query contract        │
                    │  - Lifecycle contract    │
                    └──────────────┬───────────┘
                                   │
                                   ▼
┌────────────────────────────────────────────────────┐
│                    CYRUNE Free                     │
│----------------------------------------------------│
│  Mandatory Boundary Layer                          │
│  - 3-layer memory                                  │
│  - Fail-closed Gate                                │
│  - Citation enforcement                            │
│  - Ledger (atomic)                                 │
│  - No-LLM mode                                     │
│                                                    │
│  対象：個人                                        │
└───────────────────────────────┬────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────┐
│                    CYRUNE Pro                      │
│----------------------------------------------------│
│  Inference Inspection Layer                        │
│  - Time-axis Diff (D1-D5)                          │
│  - Single-LLM Multi-Agent                          │
│  - CI Gate                                         │
│                                                    │
│  対象：個人 / 小規模チーム                         │
└───────────────────────────────┬────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────┐
│                   CYRUNE Pro+                      │
│----------------------------------------------------│
│  Multi-Model Consensus Layer                       │
│  - Cross-Model Diff (CM-D1-D5)                     │
│  - Consensus Engine                                │
│  - Stability Score                                 │
│  - Drift Detection                                 │
│                                                    │
│  対象：高信頼開発者 / 研究者                       │
└───────────────────────────────┬────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────┐
│                 CYRUNE Enterprise                  │
│----------------------------------------------------│
│  Organizational Governance Layer                   │
│  - Shared Ledger                                   │
│  - RBAC                                            │
│  - SSO                                             │
│  - Organization Policy Enforcement                 │
│  - Vault Segmentation                              │
│                                                    │
│  対象：組織                                         │
└───────────────────────────────┬────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────┐
│                    CITADEL                         │
│----------------------------------------------------│
│  Hardened Enforcement Layer                        │
│  - WORM Ledger                                     │
│  - Airgap                                          │
│  - Supply Chain Lock                               │
│  - Hardware Key                                    │
│                                                    │
│  対象：防衛 / 高機密環境                           │
└────────────────────────────────────────────────────┘


* CRANEは契約Kernel（OSS）
* CYRUNEはControl OS
* CITADELはHardened Distribution

---

* CRANEは契約Kernel（OSS）
* CYRUNEはControl OS
* CITADELはHardened Distribution

---

# 2. 不変原則（全ティア共通）

以下は削らない。

* 三層メモリ（Working / Processing / Permanent）
* Working 10±2（modified Miller’s）
* context clear per turn
* Fail-closed Gate
* Citation-bound reasoning
* Evidence Ledger（append-only, atomic）
* Detachable LLM architecture
* No-LLM mode

**Freeであっても本質は削らない。**

---

# 3. CYRUNE Free

## 3.1 定義

> Single-user Control OS（完全体）

思想の武器。
機能削減版ではない。

---

## 3.2 含まれる機能

### Control Core

* context clear
* Working 10±2強制
* Processingログ保存
* Permanent昇格（手動）
* Hybrid search（BM25 + lightweight embedding）

### Gate

* 出典なし断定拒否
* 未定義語検知
* 境界侵食検知
* deny-by-default capability

### Ledger

* atomic確定
* ハッシュ保存
* クエリ→根拠束→出力→更新差分保存
* 構造化推論記録（RR: Reasoning Record）の保存（Diff生成は含まない）

### LLM

* No-LLM mode
* Local LLM
* LLMはdetachable adapter

### Runtime

* CLI canonical
* WezTermベースTerminal統合
* Cross-platform

  * macOS: strong enforcement
  * Linux (Debian/RHEL系): strong enforcement
  * Windows: scoped-to-cyrune enforcement

### 配布

* GitHub Actions canonical CI
* SBOM生成
* SHA256生成
* macOS adhoc署名（v0.1）
* Linux/Windowsはハッシュ担保

---

## 3.3 含まれないもの

* Multi-user
* RBAC
* 組織Policy強制
* 共有Ledger
* 異種LLM multi-agent
* GUI
* マルチモーダル
* 中央管理

---

## 価格

**$0**

---

# 4. CYRUNE Pro

## 4.1 定義

> 単一LLM内の構造検査層（個人～小規模チーム向け）

Freeの本質を拡張する。

---

## 4.2 追加機能

### A. 推論差分検出（時間軸Diff）

* 前回出力との差分
* 役割間の論理差分
* 引用範囲との差分
* 仮説正当化検知

### B. Multi-Vault

* 複数プロジェクト分離
* 横断検索
* Policy別Vault

### C. Single-LLM Multi-Agent

同一LLM内で役割分離：

* Planner
* Critic
* Verifier
* Synthesizer

役割間検証をLedger保存。

### D. CI Gate（単一モデル）

* PR時Gate実行
* Evidence自動生成
* 推論差分をレビュー出力

---

## 4.3 含まれないもの

* 異種LLM実行
* Cross-model diff
* Consensus engine
* Stability score
* Drift detection
* RBAC
* 共有Ledger

---

## 価格

**$9 / user / month**

---

# 5. CYRUNE Pro+

## 5.1 定義

> 異種LLM合議・推論安定性保証層

Proは単一LLM内で推論を検査する。
Pro+は複数LLM間で推論を比較・合議し、推論安定性を保証する。

---

## 5.2 Pro機能すべて +

### A. 異種LLM multi-agent

* Claude + Codex 等を並列実行
* run_group_idで束ねる

### B. Cross-Model Diff（空間軸Diff）

CM-D1〜CM-D5：

* Citation divergence
* Claim divergence
* Dependency divergence
* Policy divergence
* Working impact divergence

### C. Consensus Engine（合議確定機構）

* 構造的優先ルールに基づく採用
* citation重み比較
* Policy違反側の棄却
* 合議不能時 auto-block

### D. Inference Stability Score

* divergence比率から安定度算出
* 推論の一貫性を数値化

### E. Conflict Escalation Mode

* 重大divergenceで自動Block

### F. Model Drift Detection

* 時系列比較によるモデル変化検出

---

## 5.3 含まれないもの

* RBAC
* 共有Ledger
* SSO
* 組織Policy強制
* 組織Vault

---

## 価格

**$19 / user / month**

---

# 6. CYRUNE Enterprise

## 6.1 定義

> 組織統治強制層

Pro+は推論の正しさを保証する。
Enterpriseは統治の強制を保証する。

---

## 6.2 Pro+機能すべて +

### 組織統治機能

* 共有Ledger
* 共有Working projection
* RBAC
* SSO統合
* 組織Vault
* 組織Policy強制
* LLMパック署名管理
* 組織横断Multi-LLM運用
* 監査レポート出力

---

## 価格

**$99 / user / month**
または
**$30,000+ / year（組織契約）**

---

# 7. CITADEL

## 7.1 定義

> Hardened Defense Distribution

Enterpriseの強化版。

---

## 7.2 追加機能

* WORM Ledger
* 強化MAC
* Air-gapped
* 供給網固定
* ハードウェア鍵管理
* 署名パック固定
* 非自動更新モデル

---

## 価格

**個別見積（防衛・高信頼環境向け）**

---

# 8. ティア差分まとめ

| 機能                          | Free | Pro | Pro+ | Enterprise | CITADEL |
|-------------------------------|------|-----|------|------------|----------|
| 三層メモリ                     | ✔ | ✔ | ✔ | ✔ | ✔ |
| context clear                 | ✔ | ✔ | ✔ | ✔ | ✔ |
| Fail-closed                   | ✔ | ✔ | ✔ | ✔ | ✔ |
| Ledger                        | ✔ | ✔ | ✔ | ✔ | ✔ (WORM) |
| 推論差分（時間軸）              | ✖ | ✔ | ✔ | ✔ | ✔ |
| 異種LLM実行                    | ✖ | ✖ | ✔ | ✔ | ✔ |
| Cross-model diff              | ✖ | ✖ | ✔ | ✔ | ✔ |
| Consensus engine              | ✖ | ✖ | ✔ | ✔ | ✔ |
| Stability score               | ✖ | ✖ | ✔ | ✔ | ✔ |
| Model drift detection         | ✖ | ✖ | ✔ | ✔ | ✔ |
| RBAC                          | ✖ | ✖ | ✖ | ✔ | ✔ |
| 共有Ledger                    | ✖ | ✖ | ✖ | ✔ | ✔ |
| 強化MAC                        | ✖ | ✖ | ✖ | ✖ | ✔ |
| Airgap                        | ✖ | ✖ | ✖ | ✖ | ✔ |

---

# 9. 戦略的原則

1. Freeは削らない
2. Proは検査を売る
3. Pro+は合議と安定性を売る
4. Enterpriseは統治を売る
5. CITADELは強度を売る
6. 本質（制御）は常にFreeに含める
