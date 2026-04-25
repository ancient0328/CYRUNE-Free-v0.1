# CYRUNE Pro Canonical

## 1) 推論差分アルゴリズム定義（Inference Diff Detection）

**Status**: Canonical（Pro）
**目的**: 「前回と今回の推論が、どこで・なぜ・どの根拠で変わったか」を**機械的に検出**し、Evidence Ledgerに残す。
**非目的**: 生成文のdiff、言い回しのdiff、UIの差分表示。
**基本方針**: *Diffは“文章”ではなく“推論構造”に対して取る。*

---
> Free は簡易 RR を保存する。
> Pro は RR を入力として推論差分（D1〜D5）を生成する層であり、
> 本章で定義する RR schema は Pro の diff / single-LLM multi-agent に接続する厳密 schema である。
> Free に Diff・multi-agent は要求しない。

## 1.1 入力（Inputs）

各ターン（あるいは各 Run）で、Control Plane が確定する以下を入力とする。

* `EVID_prev`（前回のEvidence）
* `EVID_curr`（今回のEvidence）
* `W_prev` / `W_curr`（Working 10±2 の確定状態）
* `C_prev` / `C_curr`（Citation Bundle：引用束）
* `P_prev` / `P_curr`（Policy Pack ID + ルール版）
* `R_prev` / `R_curr`（Reasoning Record：後述の構造化推論記録）

> Diffの比較単位は、**Evidence単位**（EVID）を基本とする。
> “同一タスク系列”の判定は `task_id`（後述）で閉じる。

---

## 1.2 構造化推論記録（Reasoning Record, RR）

LLMの「内部思考」を要求しない。代わりに、CYRUNEが **出力を構造化して確定**する。

Free では簡易 RR を保存する。
Pro では、本章の RR schema を推論差分と single-LLM multi-agent の基底として使う。

RRは以下の閉集合ブロックから成る：

### Pro RR Schema（v0.1）

* `task_id`: string（人が決める・またはWorkSlotから導出）
* `claims`: Claim[]（主張）
* `assumptions`: Assumption[]（仮定）
* `decisions`: Decision[]（決定）
* `open_questions`: OpenQuestion[]（未解決）
* `actions`: Action[]（次アクション）
* `constraints`: Constraint[]（制約）
* `citations_used`: CitationRef[]（使用した根拠）
* `policy_events`: PolicyEvent[]（deny/allow/require-proof）
* `agent_meta`: { role, run_id, model_id, adapter_id }

### Claim（v0.1）

* `claim_id`: string（決定的ID：hash）
* `kind`: enum { fact, inference, recommendation, plan_step }
* `text`: string（短文）
* `support`: enum { cited, derived, uncited }

  * `uncited` は **Gateで拒否**（Pro/Free共通）
* `citations`: CitationRef[]（必須：support=cited/derived）
* `depends_on`: claim_id[]（任意、推論依存）

> ここでの “derived” は「引用束の複数箇所からの合成（要約/統合）」であり、引用束外の知識混入は禁止。

---

## 1.3 差分の種類（Diff Taxonomy：閉集合）

Diffは以下の **5種類**だけを扱う（閉集合）。

1. **D1: Citation Diff**

   * 引用束（根拠）が変わった
   * 例：Top-kが変化、別文書が採用された、同一文書の別箇所が採用された

2. **D2: Claim Diff**

   * 主張（claim set）が変わった
   * 追加/削除/意味変更（hash変化）

3. **D3: Dependency Diff**

   * 推論依存関係が変わった
   * 例：AがBに依存しなくなった、依存先が変わった

4. **D4: Policy Diff**

   * Policy Pack / ルール評価が変わった
   * 例：以前allowだったがdenyになった、require-proofが増えた

5. **D5: Working State Diff**

   * Working 10±2 の状態（decision/constraint/todo）が変わった
   * 例：前提が更新された、制約が追加された、タスクが完了した

> 文章表現のdiffは一切扱わない。これは設計上の禁止。

---

## 1.4 アルゴリズム（Deterministic Core）

### Step 0: 正規化（Normalization）

* 文字列はtrim、連続空白正規化
* Claimは `claim_id = SHA256(kind + text + sorted(citations) + sorted(depends_on))` で決定
* CitationRefは `(doc_id, chunk_id, span)` の決定形式に統一
* RR全体は `rr_hash` を算出しLedgerに保存

### Step 1: Citation Diff（D1）

* `C_prev` と `C_curr` を集合比較

  * `added_citations = C_curr - C_prev`
  * `removed_citations = C_prev - C_curr`
* 変更量が閾値を超える場合、以降のDiffの信頼度に注記（“根拠が大きく変化した”）

### Step 2: Claim Diff（D2）

* `claims_prev` と `claims_curr` を `claim_id` で集合比較

  * added / removed
* “意味変更”は、同一kindで `text_similar` かつ citationが変わるケースを `mutated_claim` として扱う
  （実装は v0.1では保守的でよい：**基本はhash差＝別claim**）

### Step 3: Dependency Diff（D3）

* `depends_on` のエッジ集合（claim_id -> claim_id）を比較
* 重要度は「どのDecision/Recommendationに到達する経路が変わったか」で評価

### Step 4: Policy Diff（D4）

* `policy_events` を比較
* deny/require-proofが増えた場合は **“安全側”の変化**として表示する（が、評価はしない）

### Step 5: Working Diff（D5）

* Working slotsを `slot_id` 単位で比較
* `decision`/`constraint` の変更を最重要扱い
* 変更は必ず `source_evidence` を持つ（持てない変更は fail-closed）

### Step 6: Diff Summary生成（構造出力）

* 生成物は文章ではなく、**Diff Record**（JSON/TOML）として確定しLedgerへ保存
* Viewer（CLI）はそのDiff Recordを表示するだけ

---

## 1.5 Diff Record（Ledgerへ保存する確定形式）

```json
{
  "version": 1,
  "task_id": "T-20260301-001",
  "prev_evidence": "EVID-120",
  "curr_evidence": "EVID-121",
  "rr_prev_hash": "...",
  "rr_curr_hash": "...",
  "diff": {
    "D1_citation": { "added": [...], "removed": [...] },
    "D2_claim": { "added": ["c1"], "removed": ["c9"], "mutated": [] },
    "D3_dependency": { "added_edges": [...], "removed_edges": [...] },
    "D4_policy": { "changes": [...] },
    "D5_working": { "changed_slots": [...] }
  },
  "severity": "info|warn|block",
  "gate_notes": []
}
```

### severity（閉集合）

* `info`: 通常の差分
* `warn`: 根拠が大きく変化、またはWorkingの重要スロットが変化
* `block`: “差分が説明不能” または “出典なき変更” が混入（fail-closed）

---

## 1.6 Fail-Closed条件（推論差分における拒否）

以下は **Diff生成自体を失敗**させる（＝Pro機能として成立していない）。

* RRが構造化できない（欠落・不正形式）
* claim/support が `uncited` を含む
* Working更新に `source_evidence` が無い
* 前回/今回の `task_id` が不整合で、かつ解決不能（曖昧な紐付け禁止）
* Diff Record の atomic 確定に失敗

---

## 1.7 “推論差分検出”のPro価値（定義）

Proにおける推論差分は、次を可能にする：

* AIが “前と言ってたこと” を変えた瞬間を捕捉
* 変化理由を「根拠束」「Policy」「Working更新」に還元
* PR/CIで「根拠が変わった変更」を止める入口になる

---

## 2) Single-LLM Multi-Agent プロトコル仕様

**Status**: Canonical（Pro）
**目的**: 単一LLMを、**役割分離した複数agent**として運用し、推論差分検出とGate/ Ledgerと結合する。
**非目的**: 異種LLM間の直接会話、モデルアンサンブル（Enterpriseに送る）。

---

## 2.1 Agent Roles（閉集合）

Proで許可するroleは以下のみ：

1. `Planner`（計画/案出し）
2. `Critic`（反証/弱点検出）
3. `Verifier`（引用・policy・制約の整合チェックの前段）
4. `Synthesizer`（統合/最終化）

> 役割追加は禁止（Enterpriseで拡張する場合は別正典）。

---

## 2.2 プロトコル原則

* agent同士が **自由会話**してはいけない
  → すべて **Control Plane を経由**してメッセージングする
* すべてのメッセージは `request_id` と `response_to` により相関決定（相関不能禁止）
* agentの出力は必ず RR（構造化推論記録）として確定する
* agentは **Citation Bundle 外に出てはならない**（Gateで拒否）
* “曖昧な非同期エラー”は禁止（fail-closed）

---

## 2.3 Message Schema（閉集合）

### Request

```json
{
  "version": 1,
  "request_id": "uuid",
  "role": "Planner|Critic|Verifier|Synthesizer",
  "task_id": "T-...",
  "inputs": {
    "working": {...},
    "citation_bundle_id": "CB-...",
    "policy_pack_id": "default",
    "objective": "string",
    "constraints": ["..."]
  },
  "expected_output": "RR|DiffHints|GateReport"
}
```

### Response

```json
{
  "version": 1,
  "response_to": "uuid",
  "role": "...",
  "task_id": "T-...",
  "output": {
    "rr": {...},
    "diff_hints": {...},
    "gate_report": {...}
  }
}
```

---

## 2.4 Execution Order（標準フロー）

Proの標準フロー（閉集合）は次：

1. **Preflight**

   * context clear
   * Working再構成
   * Citation bundle生成（CB）
   * Policy pre-check

2. **Planner**

   * RR生成（案）

3. **Critic**

   * Planner RRに対する反証RR（批判 claim set）

4. **Verifier**

   * Planner/Criticのclaimを引用束・policy・working制約で検査
   * `GateReport` を生成（まだGate本体は動かさない）

5. **Synthesizer**

   * Planner + Critic + Verifier を統合
   * 最終RR生成

6. **Gate（本体）**

   * citation-bound / fail-closed / capability / policy

7. **Ledger**

   * Evidence確定（atomic）
   * RR保存
   * 推論差分（RR_prevとRR_curr）生成・保存

> Verifierは「Gateの代理」ではない。
> Gateがfail-closedの単一権威であり、Verifierは前段検査官。

---

## 2.5 共有コンテキスト（agent間で共有してよいもの）

共有可能なのは **Control Planeが提供する入力のみ**：

* Working 10±2
* Citation Bundle（CB）
* Policy Pack ID
* Task objective / constraints

agentは **互いの自然言語出力**をそのまま食ってはいけない。
必ず **RRとして構造化された出力**を次段が参照する。

---

## 2.6 予算（Budgets）

Proの設計として、以下の予算を導入する（閉集合）。

* `max_rounds = 1`（Planner→Critic→Verifier→Synthesizer を1往復）
* `max_claims_per_agent`（例：20）
* `max_citations_per_claim`（例：5）
* `max_total_tokens`（adapter依存だが上限はControl Planeが保持）

> 予算超過は fail-closed で「要件未達」として扱い、出力を確定しない。

---

## 2.7 Multi-Agent の“直接会話”を禁止する理由

* 会話はノイズを増やす
* 証跡が追えなくなる
* 役割境界が崩れる

代わりに、**RRという構造物**でのみやり取りすることで、

* 差分が取れる
* 監査できる
* Gateで止められる

---

## 2.8 Proでの Multi-Agent の価値定義

Proにおける multi-agent は、

* 「複数モデル比較」ではなく
* 「役割分離による推論の自己検査」

である。

推論差分検出（Inference Diff）と結合して初めて“武器”になる。

---

# 3) 直近の実装優先順位（Pro）

この2つを実装に落とす順序（正しい最短）は：

1. Free の簡易 RR と整合する Pro RR schema（構造化推論記録）を確定する
2. 推論差分（D1〜D5）を、RR + Citation + Working + Policy から決定的に生成
3. Multi-agentプロトコル（Planner→Critic→Verifier→Synthesizer）を導入
4. CI Gate（PRでDiff + Gate + Evidence）へ接続
