# CYRUNE Enterprise Canonical

**Status**: Canonical（Enterprise）
**Scope**: 組織統治 / 共有Ledger / RBAC / SSO / 組織Policy強制

---

# 1) 定義

> Enterprise は、組織単位で推論・実行・証跡を強制統治する層である。

Pro+ は推論の正しさを保証する。
Enterprise は統治の強制を保証する。

---

# 2) Pro+との差異（決定的定義）

| 軸 | Pro+ | Enterprise |
|----|------|------------|
| 対象 | 個人 / 小規模 | 組織全体 |
| 合議 | ✔ | ✔ |
| Stability/Drift | ✔ | ✔ |
| RBAC | ✖ | ✔ |
| 共有Ledger | ✖ | ✔ |
| 組織Policy強制 | ✖ | ✔ |
| SSO | ✖ | ✔ |
| 組織Vault | ✖ | ✔ |
| 監査レポート | ✖ | ✔ |
| LLMパック署名管理 | ✖ | ✔ |

---

# 3) 組織統治モデル

## 3.1 Organization Schema

```json
{
  "org_id": "ORG-001",
  "policy_pack_id": "enterprise-default",
  "ledger_mode": "shared",
  "rbac_enabled": true,
  "sso_provider": "SAML|OIDC",
  "vaults": ["V-ARCH", "V-SEC", "V-FIN"]
}
````

---

# 4) RBAC（Mandatory Access Governance）

## 4.1 Role Model（閉集合）

* Admin
* Policy Admin
* Auditor
* Developer
* Reviewer
* Observer

## 4.2 権限軸

* Ledger閲覧
* Policy変更
* Vaultアクセス
* LLMパック変更
* Run承認
* Drift承認

Fail-openは禁止。

---

# 5) 共有Ledger（Organization Ledger）

EnterpriseではLedgerは個人単位ではなく、組織単位で共有される。

## 5.1 特性

* append-only
* atomic確定
* run_group単位で保存
* 改竄検知（hash chain）

## 5.2 監査可能項目

* 誰が実行したか
* どのPolicyで実行したか
* どのモデルを使用したか
* どのVaultを参照したか
* Stability score
* Divergence severity

---

# 6) 組織Policy強制

Pro+ではローカルPolicy。
Enterpriseでは組織Policyを強制する。

## 6.1 強制事項

* 禁止Capabilityのoverride不可
* 未承認LLMパック使用不可
* Vault間参照禁止（Cross-domain block）
* 外部Connector制限

---

# 7) LLMパック署名管理

## 7.1 Pack Model

```json
{
  "model_id": "claude-code",
  "hash": "sha256:...",
  "approved_by": "Policy Admin",
  "approval_timestamp": "..."
}
```

未承認モデルは実行不可。

---

# 8) 組織Vault

## 8.1 Vault分離

* Project Vault
* Department Vault
* Confidential Vault

Vault間参照はPolicyで明示許可。

---

# 9) 監査レポート生成

Enterpriseは監査出力機能を持つ。

例：

* 月次実行統計
* Divergence発生回数
* Drift検出回数
* Policy違反Attempt
* Stability平均値

出力形式：

* JSON
* PDF
* CSV

---

# 10) Governance Fail-Closed

以下は組織レベルBlock：

* RBAC違反
* Policy未承認変更
* 未署名LLMパック使用
* Vault越境参照
* Ledger不整合

---

# 11) Deployment Modes

* On-Prem
* Private Cloud
* VPN限定
* SSO連携必須モード

---

# 12) 価格

**$99 / user / month**
または
**$30,000+ / year（組織契約）**

価格は責任に比例する。

---

# 13) Enterpriseの本質

Enterpriseは「便利さ」を売らない。

売るのは：

* 組織統治
* 強制力
* 監査可能性
* 供給鎖固定
* 実行責任の明確化

---

# 14) 最終定義

Pro+ が保証するのは「推論の安定性」。

Enterprise が保証するのは「組織統治の強制」。

これらは階層的だが、性質は異なる。

Enterpriseは推論品質を超えた、
**統治インフラ層である。**
