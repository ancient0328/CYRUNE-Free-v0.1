# CYRUNE Free v0.1: Contracts And Data Models

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. この巻の目的

この巻は、CYRUNE Free v0.1 を構成する contract と data model を、実装に依存しすぎずに本文で固定する。
ここを読めば、何が必須で、何が禁止で、何が fail-closed 条件かを把握できる。

## 2. identity family

### 2.1 request_id

Runtime が受け取った 1 回の要求を識別する。

### 2.2 correlation_id

1 turn を end-to-end で追跡するための相関 ID である。
request、run、ledger、metrics、working update を同じ turn に結びつける。

### 2.3 run_id

実行単位の ID である。
Free v0.1 は `single_run` topology を採るため、1 correlation_id に対して `<correlation_id>-R01` だけを許す。

### 2.4 evidence_id

ledger 上の証跡単位を識別する。
`EVID-<u64>` 形式を採る。

### 2.5 denial_id

rejected run に付く deny event の識別子である。
`DENY-<u64>` 形式を許す。

## 3. memory semantics

### 3.1 Working

Working は、その turn に実際に使ってよい判断材料だけを保持する小さな集合である。

必須条件:

1. hard limit は 12
2. 運用目標は 10±2
3. 毎ターン再構築する
4. 前ターンの生文脈は引き継がない
5. slot は evidence に紐づかなければならない

### 3.2 Processing

Processing は中期保持層である。

必須条件:

1. 直近の run 結果、citation candidate、中間要約、Working 候補を置く
2. 保持方針は 42 日
3. 検索対象に含めてよい
4. そのまま Working に流し込んではならない
5. Permanent への自動昇格は禁止

### 3.3 Permanent

Permanent は手動昇格の長期保持層である。

必須条件:

1. non-expiring を要求する
2. 明示的昇格が必要
3. 静かな上書きを許さない
4. 長期に残す定義や決定を置く

## 4. Working projection model

`working.json` は次の性質を持つ。

1. schema version は 1
2. `generated_at` を持つ
3. `correlation_id` を持つ
4. `limit` は 12 固定
5. `slots` は 12 件以下

各 slot は少なくとも次を持つ。

1. `slot_id`
2. `kind`
3. `text`
4. `source_evidence_id`
5. `source_layer`
6. `priority`
7. `updated_at`

`kind` は閉集合である。

1. `decision`
2. `constraint`
3. `assumption`
4. `todo`
5. `definition`
6. `context`
7. `command`

## 5. Policy Gate model

Policy Gate は思想ではなく強制分岐である。

### 5.1 基本原則

1. 既定は deny
2. allow は明示されたものだけ
3. pre-check と post-check の両方を行う
4. 未定義 capability を通さない

### 5.2 capability の閉集合

1. `exec`
2. `fs_read`
3. `fs_write`
4. `git_write`
5. `net`
6. `browser`
7. `ci`

### 5.3 pre-check で止めるもの

1. policy pack 不在
2. capability 閉集合違反
3. capability 未許可
4. binding 未解決
5. Working 不正
6. 必須分類情報欠落

### 5.4 post-check で止めるもの

1. Citation Bundle 不在
2. uncited claim
3. citation scope を超える derived claim

### 5.5 Policy Gate 起因の reason_kind

1. `policy_denied`
2. `binding_unresolved`
3. `working_invalid`
4. `citation_denied`

## 6. Citation Bundle model

accepted output には Citation Bundle が必須である。

bundle は少なくとも次を持つ。

1. `bundle_id`
2. `correlation_id`
3. `claims`

claim entry は少なくとも次を持つ。

1. `claim_id`
2. `text`
3. `claim_kind`
4. `evidence_refs`

`claim_kind` は閉集合である。

1. `verbatim`
2. `extractive`
3. `derived`

`derived` は citation scope 内でのみ許可される。
bundle 外の claim を accepted output に出してはならない。

## 7. simplified reasoning record

Free v0.1 では reasoning record を削除しない。
ただし Pro 相当の diff / consensus / role graph までは要求しない。
最小 reasoning record は次を含む。

1. `claims`
2. `decisions`
3. `assumptions`
4. `actions`
5. `citations_used`

## 8. Evidence Ledger model

Ledger は通常ログではなく run 単位の監査構造体である。

### 8.1 最上位制約

1. append-only
2. accepted / rejected を問わず証跡化
3. atomic commit
4. commit 不能なら accepted を返さない
5. evidence は 1 run_id と 1 correlation_id に結びつく

### 8.2 論理配置

ledger 配下には少なくとも次を持つ。

1. `manifests/index.jsonl`
2. `evidence/EVID-<id>/manifest.json`
3. `run.json`
4. `policy.json`
5. `hashes.json`
6. accepted の場合は `citation_bundle.json`、`rr.json`、`working_delta.json`、`stdout.log`、`stderr.log`
7. rejected の場合は `denial.json`

### 8.3 manifest の最小責務

1. `evidence_id`
2. `correlation_id`
3. `run_id`
4. `outcome`
5. `created_at`
6. `policy_pack_id`
7. `working_hash_before`
8. `working_hash_after`
9. `citation_bundle_id` または `null`
10. `rr_present`

### 8.4 policy projection の最小責務

`policy.json` は少なくとも次を追跡可能に残す。

1. requested / resolved policy selection
2. resolved kernel adapters
3. `embedding_exact_pin` または `null`
4. `selected_execution_adapter` または `null`
5. `memory_state_roots` または `null`
6. allowed capabilities
7. rule evaluations
8. final decision

shipping binding で resolved context が確定した run では、`memory_state_roots` に processing / permanent の physical root を残す。

## 9. atomic commit rules

Evidence commit の順序は次である。

1. `.tmp` ディレクトリを作る
2. 必須ファイルをすべて書く
3. `fsync` する
4. `.tmp` から本番名へ rename する
5. `index.jsonl` に append する

index は走査補助であり、本体ではない。
index だけを source of truth としてはならない。

## 10. turn flow rules

current accepted turn flow の必須工程は次である。

1. request validate
2. run_id allocate
3. context clear
4. policy / binding resolve
5. Working rebuild
6. Gate pre-check
7. search and execution
8. Citation Bundle build
9. citation validate
10. ledger commit
11. working update
12. terminal result

ledger commit 前に output を返してはならない。

## 11. runtime surface rules

### 11.1 user-facing command family

最小 user-facing command family は次である。

1. `cyr shell`
2. `cyr run --no-llm --input <text>`
3. `cyr run --adapter <approved-execution-adapter-id> --input <text>`
4. `cyr view evidence`
5. `cyr view working`
6. `cyr view policy`
7. `cyr doctor`

### 11.2 daemon IPC family

最小 IPC family は次である。

1. `Run`
2. `Cancel`
3. `Tail`
4. `GetEvidence`
5. `ListEvidence`
6. `GetWorking`
7. `ExplainPolicy`
8. `Health`

未知コマンドは fail-closed で reject する。

## 12. bundle / home contract

### 12.1 bundle rule

packaged mode の static authority は `BUNDLE_ROOT` のみである。

### 12.2 home rule

`CYRUNE_HOME` は mutable state root である。
generated file や materialized projection を持てるが authority ではない。

### 12.3 exact pin rule

shipping exact pin manifest と artifact set は bundle-root authoritative である。
home 側 `embedding/**` は byte-identical materialized projection にすぎない。

## 13. D6 contract

D6 は native outer launcher line である。
許される主語は outer launch integration のみである。

必須制約:

1. `cyr` を置換しない
2. daemon / Control Plane を bypass しない
3. `BUNDLE_ROOT` authority を変えない
4. `CYRUNE_HOME` を authority 化しない
5. launcher / preflight / run-path split を保つ
6. D7 productization family を持ち込まない

## 14. D7 contract

D7 は terminal bundle productization line である。
許される主語は次に限定される。

1. bundle identity
2. rebrand / product presentation
3. notice / attribution conduit
4. integrity / signature / release preparation
5. upstream intake judgment

禁止事項:

1. `cyr` public surface の再定義
2. launcher family の再定義
3. authority graph の再定義
4. self-update の導入
5. upstream auto-follow の導入
6. unsigned / notice 欠落 package の success 扱い

## 15. fail-closed matrix

### 15.1 run-path

次は run-path unresolved または corresponding reject へ閉じる。

1. binding 未解決
2. packaged resource 未解決
3. exact pin source 不整合
4. authority graph 違反

### 15.2 preflight

次は preflight failure として閉じる。

1. package prerequisite 不足
2. writable check failure
3. generated config postcheck failure

### 15.3 launcher

次は launcher failure として閉じる。

1. outer launcher の起動前提崩壊
2. terminal host が見つからない
3. launcher-owned integration failure

### 15.4 productization

次は productization failure として閉じる。

1. bundle assemble failure
2. branding resource failure
3. notice / SBOM / integrity preparation failure
4. upstream drift / metadata invalid

## 16. 現在の採用範囲

current accepted claim に採用しているのは次である。

1. core semantics
2. corrective line
3. D6 line
4. D7 line

current accepted claim に採用していないのは次である。

1. reverse-DNS bundle identifier
2. signing identity の concrete value
3. notarization provider の concrete value
4. installer / archive file name
5. upstream source pin の concrete revision

これらは未固定だが、current blocker ではない。

ただし post-v0.1 add-on scope `D7-RC1` は既に complete であり、rule-fixed family、organization-owned contract、organization-owned variable family の exact reason / publicization boundary、accepted / fail-closed artifact family、final closeout adoption までは採用済みである。
現時点で fixed 済みなのは concrete value ではなく、organization owner が supply し、top-level `RELEASE_PREPARATION.json.signing_identity` と `RELEASE_PREPARATION.json.notarization_provider` を input location とし、missing / invalid を `signing_identity_invalid` / `notarization_provider_invalid` reason と fixed message に畳み込み、field-level invalidity を `release_preparation_metadata_invalid` に丸めず、`release_preparation_failure` family に閉じるという contract と、その accepted / fail-closed artifact family、および final closeout adoption family である。
