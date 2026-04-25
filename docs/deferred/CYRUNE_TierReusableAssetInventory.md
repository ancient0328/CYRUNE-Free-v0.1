# CYRUNE Tier Reusable Asset Inventory

**Status**: Reference Inventory
**Date**: 2026-03-27
**Scope**: CYRUNE Free / Pro / Pro+ / Enterprise に対する既存資産の流用・応用可能性の体系整理

---

## 1. この文書の位置づけ

この文書は、CYRUNE の各 tier に対して、既存リポジトリ群にどの程度の再利用可能資産が存在するかを、
**真正情報のみ**で棚卸しした正式 inventory である。

この文書は次のことを行わない。

- 実装充足判定
- 採用済み依存の宣言
- 足不足を主語にした評価
- prototype / sibling 実装を current CYRUNE 実装とみなすこと

本 inventory の主語は一貫して **「どの tier に、どの既存資産が、どの粒度で流用・応用できるか」** である。

---

## 2. 時間相と分類規則

異なる時点の成果物を混同しないため、資産は次の区分で扱う。

| 区分 | 意味 |
|------|------|
| current canonical dependency side | 現行 CYRUNE 正典が依存面として受けている current 側資産 |
| current implementation | 現在の sibling / dependency repo に存在する現行実装 |
| prototype | 過去の試作実装。構想・実験・部分移植の源泉 |
| backup / prototype | 履歴保存された試作群。現在の採用を意味しない |
| sibling implementation | 現在存在する別プロジェクトの現行実装。部分移植や内部設計参考になり得る |

また、再利用の強さは次の 4 段で記述する。

| 再利用区分 | 意味 |
|------------|------|
| drop-in | 現行 CYRUNE の下位依存・契約基盤としてそのまま受けやすい |
| adapt | 意味論差分を保った上で移植・再構成しやすい |
| pattern | 実装パターンや運用構造の参照元として有効 |
| archive-reference | 履歴把握には有用だが、現行計画の直接資産としては扱わない |

---

## 3. tier 前提

### 3.1 Free

Free は最小完成形の Control OS 本体であり、三層メモリ、Working 10±2、context clear per turn、
fail-closed gate、citation-bound reasoning、append-only / atomic evidence ledger、
detachable LLM、No-LLM、CLI canonical を含む。

### 3.2 Pro

Pro は Free を前提に、時間軸 Diff、より厚い RR、single-LLM multi-agent、CI gate、
multi-vault を加える。

### 3.3 Pro+

Pro+ は Pro を前提に、異種 model 並列実行、run_group、cross-model diff、
consensus、stability、drift を加える。

### 3.4 Enterprise

Enterprise は Pro+ を前提に、organization schema、RBAC、shared ledger、SSO、
organization policy enforcement、vault segmentation、governance audit を加える。

---

## 4. source group inventory

## 4.1 CRANE-Kernel v0.1

**分類**: current implementation + current canonical dependency side
**再利用区分**: drop-in

確認できる資産:

- `crane-kernel`
- `crane-store-inmem`
- `crane-embed-null`
- `crane-kernel-ref-inmem`
- `crane-extension-thinking`
- 6 public interfaces
- 3-layer memory contracts
- closed error model
- deterministic in-memory reference implementation
- `dep-gate` / `vocab-gate` / `perf-gate`
- 19 fixed benchmarks

tier への効き方:

- Free: 直接依存の契約核
- Pro: Free 継承の契約核
- Pro+: Free / Pro 継承の契約核
- Enterprise: 全 tier 共通の契約核

## 4.2 Adapter v0.1

**分類**: current implementation + current canonical dependency side
**再利用区分**: drop-in

確認できる資産:

- adapter-resolver
- capability manifest / policy pack / binding schema
- `memory-kv-inmem` catalog
- `cyrune-free-default.v0.1.json`
- `cyrune-free-default.v0.1.binding.json`

確認できる policy 値:

- Working `target_items = 10`
- Working `ttl_ms = 3600000`
- Processing `target_items = 20000`
- Processing `ttl_ms = 3628800000`
- `promotion_threshold = 0.8`
- fail-closed flags

tier への効き方:

- Free: policy / binding の直接基盤
- Pro: lower-layer policy / binding 基盤
- Pro+: lower-layer policy / binding 基盤
- Enterprise: lower-layer policy / binding 基盤

## 4.3 CRANE-Kernel backup/prototype v0

**分類**: prototype
**再利用区分**: adapt

確認できる資産:

- working-memory optimization 実験
- DashMap / SkipList / LRU 比較
- RocksDB persistence 実験
- Qdrant / ONNX 探索

tier への効き方:

- Free: Working 10±2 と persistence 実装の実験根拠として最も近い
- Pro / Pro+ / Enterprise: lower-layer 実験根拠として間接利用

## 4.4 CRANE-Kernel backup/prototype v0.2

**分類**: backup / prototype
**再利用区分**: adapt

確認できる資産群:

- `crane-cli`
- `crane-core`
- `crane-mcp`
- `crane-memory`
- `crane-ml`
- `crane-storage`
- `crane-thinking`

確認できる surface:

- `crane-cli`: daemon start / stop / status / backup の管理面
- `crane-mcp`: server / protocol / transport / memory tool / thinking tool / RAG / vector search
- `crane-memory`: three-tier memory / Miller's Law working memory / transition manager
- `crane-storage`: RocksDB / Qdrant / hybrid search / RAG / vector index
- `crane-thinking`: staged process selection / orchestration substrate
- `crane-ml`: distribution / security / version management / embedding 周辺

tier への効き方:

- Free: CLI / daemon / memory / storage の応用源
- Pro: CLI / MCP / thinking / storage の応用源
- Pro+: grouped execution / tool surface / orchestration の応用源
- Enterprise: model distribution / validation / version 管理の応用源

## 4.5 Ferrune

**分類**: sibling implementation
**再利用区分**: adapt

確認できる資産:

- `ReactiveCell` / `DerivedCell` / `EffectCell`
- Tarjan-based cycle detection
- dependency tracking / optimized deps
- opt-in extension isolation
- registry replication / majority consensus 実装

tier への効き方:

- Free: control-plane 内部の状態遷移参考として限定的に効く
- Pro: role / state / effect orchestration に強く効く
- Pro+: parallel orchestration / dependency graph / effect scheduling に強く効く
- Enterprise: registry replication / majority 合意の pattern として一部効く

## 4.6 arcRTC

**分類**: sibling implementation
**再利用区分**: adapt

確認できる資産:

- correlation-rich control-plane contract
- closed reason/detail を持つ request / response 型
- D1 audit insert path carrying `correlation_id`
- hash-chain audit sink
- chain verification
- transactional persistence

tier への効き方:

- Free: evidence / correlation / deterministic contract の応用源
- Pro: role message correlation と RR persistence の応用源
- Pro+: `run_group` と multi-run evidence correlation の応用源
- Enterprise: shared ledger / organizational audit の応用源

## 4.7 HugMeDo backend / quality-platform

**分類**: sibling implementation
**再利用区分**: pattern

確認できる資産:

- Rust multi-service backend topology
- `tracing` / `tracing-subscriber`
- logging redaction helper
- quality-platform の governance / audit / contracts 構造
- OIDC / IAM 分離の infra role separation 文書

tier への効き方:

- Free: tracing / service hygiene の軽量 pattern
- Pro: gate / audit / reporting 運用 pattern
- Pro+: drift / gate / evidence 運用 pattern
- Enterprise: governance / SSO / reporting / evidence operation pattern

---

## 5. tier 別 inventory

## 5.1 Free

### 直接流用の核

- CRANE-Kernel v0.1
- Adapter v0.1

### 応用資産

- prototype v0 working-memory 実験
- prototype v0 persistence 実験
- prototype v0.2 `crane-cli`
- prototype v0.2 `crane-mcp`
- prototype v0.2 `crane-memory`
- prototype v0.2 `crane-storage`
- Ferrune runtime
- arcRTC audit / correlation
- HugMeDo tracing pattern

### 位置づけ

Free は、最も厚い直接流用基盤を持つ tier である。
current canonical dependency side と一致する current 実装基盤が明確であり、その周囲に prototype と sibling の応用源が多層に存在する。

## 5.2 Pro

### 継承する基盤

- Free の全 lower-layer 資産

### 追加で応用しやすい資産

- Ferrune runtime の role / state / effect orchestration
- arcRTC の correlation / audit flow
- prototype v0.2 `crane-cli`
- prototype v0.2 `crane-mcp`
- prototype v0.2 `crane-thinking`
- prototype v0.2 `crane-storage`
- HugMeDo quality-platform

### 位置づけ

Pro は、Free を固定基盤として継承した上で、orchestration、Diff 保存、CI gate、vault 運用に関わる既存資産の応用余地が広い tier である。

## 5.3 Pro+

### 継承する基盤

- Free と Pro の全 lower-layer 資産

### 追加で応用しやすい資産

- arcRTC の correlation / audit flow
- Ferrune の dependency / effect graph primitives
- prototype v0.2 `crane-cli`
- prototype v0.2 `crane-mcp`
- Ferrune registry replication / majority consensus pattern

### 位置づけ

Pro+ は、下位 tier 継承の厚みが大きく、その上に multi-run correlation、parallel orchestration、consensus pattern を載せやすい tier である。

## 5.4 Enterprise

### 継承する基盤

- Free / Pro / Pro+ の全 lower-layer 資産

### 追加で応用しやすい資産

- arcRTC hash-chain / audit persistence
- prototype v0.2 `crane-ml` distribution / security / version management
- HugMeDo OIDC / IAM separation pattern
- HugMeDo quality-platform governance / audit structure
- Ferrune registry replication pattern

### 位置づけ

Enterprise は、下位 tier の制御基盤を前提に、governance、shared ledger、distribution、SSO、運用監査の周辺で既存資産を広く応用できる tier である。

---

## 6. 取り扱い上の注意

- inventory は adoption を意味しない
- prototype / sibling の存在は current CYRUNE 実装化を意味しない
- current canonical dependency side と prototype / sibling source は、同じ完成基準で扱わない
- tier inventory は sufficiency 判定ではなく reuse landscape の把握に使う

---

## 7. 要点

- Free は、直接流用の核と応用候補の両方が最も厚い
- Pro は、Free 継承に加えて orchestration / gate / vault / diff 周辺の応用源が厚い
- Pro+ は、multi-run correlation と consensus pattern を支える資産が下位 tier 継承の上に積みやすい
- Enterprise は、governance / SSO / audit / distribution 周辺の pattern / subsystem 応用が広い
- 全 tier を通して、最下層の不変基盤は CRANE-Kernel v0.1 と Adapter v0.1 である
