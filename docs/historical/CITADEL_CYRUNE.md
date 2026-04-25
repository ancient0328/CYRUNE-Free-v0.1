# 製品ポジショニング図（CRANE / CYRUNE / CITADEL 関係図）

**状態（当時）**: Historical product positioning and whitepaper sketch
**現在の権威状態**: Historical / non-authoritative
**取り扱い**: 2026-04-12 JST の `PB-C / PBC-I1 authority-state segregation` 後、この文書は current accepted source ではない。CRANE / CYRUNE / CITADEL 関係図と whitepaper 構成案の初期整理として参照に限定する。現行 authority は `docs/deferred/CYRUNE_ProductTierCanonical.md`、`docs/deferred/CITADEL.md`、`docs/deferred/CITADEL_ThreatModel.md`、`free/v0.1/dev-docs/summary/01-SYSTEM_AND_SCOPE.md` である。

```
                        ┌──────────────────────────────┐
                        │            RWA               │
                        │  High-Trust Architecture Co. │
                        └──────────────────────────────┘
                                       │
                                       ▼
                      ┌────────────────────────────────┐
                      │            CRANE               │
                      │  Contract-Reinforced Kernel    │
                      │  (OSS, Apache-2.0)             │
                      └────────────────────────────────┘
                                       │
                                       ▼
                 ┌────────────────────────────────────────────┐
                 │                CYRUNE                      │
                 │   Domain-Agnostic Control Operating System │
                 │   - Policy Packs                           │
                 │   - Adapter Layer                          │
                 │   - CLI Canonical                          │
                 │   - Desktop Optional                       │
                 └────────────────────────────────────────────┘
                                       │
                                       ▼
            ┌──────────────────────────────────────────────────────┐
            │                    CITADEL                          │
            │   Hardened Defense Distribution of CYRUNE           │
            │   - Air-gapped                                      │
            │   - Signed Updates                                  │
            │   - Mandatory Access Control (Strict)               │
            │   - No Self-Update                                  │
            │   - WORM Ledger                                     │
            └──────────────────────────────────────────────────────┘
```

---

### レイヤー関係の意味

| レイヤー    | 役割         | 公開性   |
| ------- | ---------- | ----- |
| CRANE   | 統制契約Kernel | OSS   |
| CYRUNE  | 統制OS（商用）   | RWA製品 |
| CITADEL | 防衛ディストロ    | 特殊契約  |

---

### 市場ポジショニング軸

```
                 ← Control Strength →
  ------------------------------------------------------
  Consumer  |  Medical  |  Finance  |  Defense
             CYRUNE  ──────────────►  CITADEL
```

* 制御強度を上げるほど CITADEL 側へ
* 一般用途は CYRUNE
* Kernelは共通

---

# 技術ホワイトペーパー構成案

CITADEL / CYRUNE 共通の技術文書構成。

---

## 1. Abstract

* Knowledge Control OSの定義
* Problem Statement（生成AIの統制不能問題）

## 2. Motivation

* AIのブラックボックス問題
* Context汚染
* 監査不能性
* 供給網攻撃リスク

## 3. Architecture

* CRANE Kernel
* Memory Model (W/P/P)
* Classification MAC
* Citation-bound reasoning
* Gate
* Ledger

## 4. Enforcement Model

* fail-closed
* deny-by-default
* classification lattice
* citation constraints

## 5. Deployment Modes

* No-LLM
* Local LLM
* Detached LLM Pack
* Airgap Mode (CITADEL)

## 6. Update & Supply Chain Security

* 18-month release
* Signed packages
* Hash verification
* No self-update

## 7. Threat Model

* Model manipulation
* Data exfiltration
* Policy tampering
* Context pollution

## 8. Compliance Alignment

* Medical (PHI)
* Finance (Retention)
* Defense (Airgap & MAC)

## 9. Future Work

* Formal verification of Gate
* Deterministic inference envelope
* Secure enclave execution

---

# エグゼクティブブリーフ（経営層向け1ページ）

---

## CYRUNE / CITADEL

### What Problem We Solve

Modern AI systems lack:

* Classification enforcement
* Citation integrity
* Audit traceability
* Offline reliability
* Deterministic governance

This creates risk in regulated industries.

---

### What We Built

A control-layer operating system for intelligent environments.

* Mandatory classification
* Citation-bound reasoning
* Fail-closed governance
* Offline deployment
* Signed update model
* Replaceable intelligence layer

---

### Market Position

* Medical institutions
* Financial institutions
* Regulated enterprises
* Defense & mission-critical systems

---

### Why It Matters

AI capability is not enough.

Control, auditability, and enforceability define real-world adoption.

CYRUNE enables safe AI deployment.
CITADEL hardens it for defense-grade environments.

---

# 公式サイト向け短縮版（シンプル）

---

## CYRUNE

A domain-agnostic operating system for controlled knowledge environments.

* Enforces classification
* Binds reasoning to verifiable sources
* Operates offline
* Maintains immutable audit trails
* Supports detachable intelligence

---

## CITADEL

The hardened defense distribution of CYRUNE.

* Air-gapped operation
* Signed, non-automatic updates
* Mandatory access control
* Immutable WORM ledger
* No external communication
