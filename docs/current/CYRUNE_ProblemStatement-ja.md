# Problem Statement（Technical Edition）

**Public Free v0.1 scope note**: この文書は構造的問題と CYRUNE の target model を説明する。public alpha が enforcement-complete classification / MAC、OS-level sandbox isolation、Pro / Enterprise / CITADEL scope、native distribution を実装済みであるとは主張しない。

**Language note**: この文書は日本語 companion です。public GitHub entry path の primary problem statement は `CYRUNE_ProblemStatement-En.md` です。

## AI 実行前に「強制境界層」が存在しないことが問題である

---

## 1. 問題の本質：LLMは「関数」ではなく「状態機械」である

大規模言語モデルは、理論上は

```
output = f(prompt)
```

の形をとりますが、実運用では

```
output = f(prompt, hidden_context, conversation_history, retrieval_noise, adapter_state)
```

という**暗黙状態依存の準状態機械**です。

この状態は：

* 無制限の会話履歴
* 混在した分類レベル
* 非決定的な検索結果
* 未検証の推論
* 外部API呼び出し副作用

に依存します。

### 結果

* 同一入力でも出力が構造的に再現不能
* 推論の根拠が外部化されていない
* 実行経路が監査できない

これは設計上の欠陥です。

---

## 2. 構造的不在：強制境界層が存在しない

一般的なAIツールの構造：

```
User → LLM → Output
```

あるいは

```
User → Retrieval → LLM → Tool Calls → Filesystem / Network
```

ここには

* Classification boundary governance
* Citation binding
* Capability gating
* Immutable audit logging

が**構造として存在しない**。

すべては“善意の実装”に依存している。

これは高信頼環境では破綻します。

---

## 3. Context 汚染の構造的問題

LLMはトークン長制限があるため、

* 文脈の切り捨て
* 暗黙の前提再構築
* 記憶の混入

が常に発生します。

Working Memory が無制限の場合：

* 過去の誤りが保持される
* 無関係な前提が残留する
* 推論の依存関係が不明瞭になる

これを防ぐには：

> 文脈の容量と寿命を物理的に制限する必要がある。

三層メモリ（Working 10±2 / Processing 42日 / Permanent）は
キャッシュ最適化ではなく、**責務分離による文脈固定**の設計です。

---

## 4. Citation-Free Reasoning は非決定性の源

多くのRAG実装は：

1. Top-k取得
2. LLMにまとめさせる
3. 出力を信頼する

しかし、

* Top-kは非決定的（索引更新・embedding揺らぎ）
* LLMは暗黙知を混ぜる
* “derived” が未定義

結果：

* どこまでが根拠か不明
* 何が引用外推論か不明

**根拠束（Citation Bundle）を第一級オブジェクトに昇格させない限り、説明可能性は成立しない。**

---

## 5. Fail-Closed が存在しない

一般的なAIツールは fail-open です。

* 引用が足りなくても出力する
* 未分類データでも処理する
* policy違反を警告に留める
* 例外はログに残るだけ

しかし高信頼環境では：

```
検証不能 ＝ 実行不能
```

でなければならない。

Fail-closed は思想ではなく、
**制御フロー上の強制分岐**である必要があります。

---

## 6. Ledger が「ログ」では足りない理由

通常のログは：

* 書き換え可能
* 構造が曖昧
* 実行単位が不明瞭
* 証跡と因果関係が弱い

CYRUNEのLedgerは：

* append-only
* atomic確定
* 実行単位（Evidence）ごとに隔離
* ハッシュ連鎖可能
* Policy / Model ID / Working hash を保存

つまり、

> 「出力」ではなく「推論状態のスナップショット」を保存する。

これはログではなく、**監査構造体**です。

---

## 7. Capability Gating がなければOSではない

AIが：

* Gitを書き換える
* シェルを実行する
* 外部APIを叩く

これを単なる“ツール呼び出し”で許可すると、
統治は存在しません。

必要なのは：

```
capability_set = { fs_read, fs_write, net, git_write, ... }
default = deny
policy_pack によって allow 明示
```

これが OS 的責務です。

---

## 8. Multi-Agent は「性能」ではなく「構造的検査」

複数モデル比較はベンチマーク的。

しかし Pro の multi-agent は：

* Planner（仮説生成）
* Critic（反証）
* Verifier（制約検査）
* Synthesizer（統合）

という**役割分離**で推論を分解する。

そして推論差分アルゴリズム（D1〜D5）で構造差分を検出する。

これは「賢さの強化」ではない。

> 推論の自己検査構造の導入

です。

---

## 9. Detachable Intelligence はセキュリティ要件

モデルを交換可能にする理由は：

* モデルは供給鎖リスクを持つ
* ハッシュ固定が必要
* silent mutation を防ぐ必要がある

モデルは権威ではない。

権威は：

* Policy
* Gate
* Ledger
* Classification lattice

である。

---

## 10. 結論

現代AIは能力中心設計です。

しかし高信頼システムは：

* 境界中心設計
* 契約中心設計
* 失敗モード固定
* 証跡第一主義

でなければ成立しません。

CYRUNEは：

```
User
  ↓
Control Plane (mandatory boundary)
  ↓
LLM
```

という構造を強制します。
