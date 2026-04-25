# ADR: CYRUNE Terminal を WezTerm ベースで統合（再商品化）する

**日付 (JST)**: 2026-03-01
**状態（当時）**: 提案（Proposed）
**現在の権威状態**: Historical / non-authoritative
**取り扱い**: 2026-04-11 JST の D7 closeout 後、この文書は current accepted source ではない。初期 ADR と判断背景の参照にのみ使う。現行 authority は `free/v0.1/dev-docs/04-implementation-notes/TERMINAL_BUNDLE_PRODUCTIZATION_CANONICAL.md`、`free/v0.1/dev-docs/01-roadmap/20260411-d7-terminal-bundle-productization-executable-roadmap.md`、`free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md` である。
**決定ID**: ADR-CTERM-0001

---

## 決定（Decision）

CYRUNE は、端末エミュレータを自作せず **WezTerm をベースに「CYRUNE Terminal」として再商品化（rebrand + repackage）**する。

ただし WezTerm の「端末として完成された機能・責務」を汚さないため、WezTerm 側に CYRUNE の統制ロジック（policy / sandbox / ledger / working）を埋め込まない。
CYRUNE 固有機能は **別プロセス（cyrune-runner / cyrune-daemon）** に集約し、CYRUNE Terminal は

- CYRUNE としての製品外観（名前/署名/設定/配布）を提供し
- 起動統合（レイアウト/既定コマンド/ショートカット）で「一体体験」を成立させる

に責務を限定する。

---

## 背景（Context）

- CYRUNE は GUI/IDE ではなく Terminal-centric な制御OS（Control Plane）である。
- 安定性を最大化するため、PTY/描画/IME/OS差分などの最も故障密度が高い領域を自前実装しない。
- 一方で「WezTermを推奨」ではなく、Cursor≠VSCode のように **製品としては “CYRUNE” を名乗る**必要がある。
- 将来 CITADEL では端末統合（権限/サンドボックス/証跡の強制、UIとledger/workingの完全統合）を前提で固定している。
  - その前段として、CYRUNE でも端末配布・供給鎖・更新責務を CYRUNE 側に移しておく方が二重化を避けられる。

---

## 目標（Goals）

1. **CYRUNEとして配布**：アプリ名・バイナリ名・署名・設定ディレクトリ等が CYRUNE になる（WezTermを名乗らない）。
2. **WezTermの純度維持**：WezTermの責務（terminal + mux）は汚さず、CYRUNE固有の統制ロジックは混ぜない。
3. **強制の単一入口**：実行・証跡・Working再構築が必ず CYRUNE runner を通る（bypassを設計で潰す）。
4. **安定運用**：UIが落ちても ledger/working が壊れない。証跡は atomic に確定する。
5. **将来拡張**：CITADEL向け端末統合へ発展可能（ただし現段階では最小侵食）。

---

## 非目標（Non-Goals）

- WezTerm内部に CYRUNEの policy/sandbox/ledger/working エンジンを実装しない。
- 端末エミュレータ核（PTY/入力/レンダリング）へ機能追加しない（原則）。
- WezTerm upstream へ頻繁追従する仕組み（定期追従）は設計しない（Evidenceベースで判断）。

---

## アーキテクチャ概要（High-level）

### 構成要素
- **CYRUNE Terminal**（WezTermベース再商品化）
  - 端末・mux・表示・キーバインド・起動レイアウト
  - CYRUNEの “入口（launcher）” と “統合体験” を担う
- **cyrune-daemon / runner**
  - 実行の単一入口（codex / claude code 等の起動）
  - policy適用、sandbox適用、ledger確定、working候補抽出
- **ledger / working**
  - SSoT（真実）は常にこちら
  - UIは投影（viewer）

### 統合の方式
- **配布統合**：CYRUNE Terminalが同梱としてCYRUNE製品に含まれる
- **起動統合**：起動時レイアウトで “Run/Evidence/Working” が常に見える
- **実行統合**：ユーザーが実行するコマンドは必ず `cyr run ...` を通す

---

## 詳細設計（Design Details）

### 1. 製品としての “CYRUNE” 化（Rebrand / Repackage）
- アプリ名：CYRUNE
- バイナリ名：`cyrune`（例：macOS app bundle も CYRUNE.app）
- 設定/データ：
  - `~/.cyrune/` を canonical とし、CYRUNE Terminal の config もここへ誘導する
  - WezTerm既定パスは採用しない（移行・混在を避ける）
- About / Licenses：
  - CYRUNE表記を主とする
  - Third-party licenses に WezTerm のライセンス（MIT）と著作権表示を確実に同梱・表示する（法的要件）

> 注：WezTermのコード改変は配布に必要な最小限（名称/アイコン/設定パス/表示）に限定する。

### 2. “WezTermを汚さない” 境界
- WezTerm側に追加してよい差分：
  - ブランド・バンドルID・アイコン・署名
  - config探索パス（CYRUNE canonical への寄せ）
  - 既定レイアウト/既定キーバインド（起動統合のため）
- WezTerm側に追加してはいけない差分：
  - policy判定、sandbox、ledger処理、working抽出などの統制ロジック
  - “CYRUNEのドメイン知識” を端末内に埋め込む行為

### 3. 起動統合（デフォルトレイアウト）
CYRUNE Terminal起動時、既定で以下を立ち上げる（最小案）。

- Pane A: `cyr shell`（ユーザーの作業シェル）
- Pane B: `cyr view evidence`（ledger viewer / 実行履歴）
- Pane C: `cyr view working`（Working 10±2 viewer）

> UI統合は “同一アプリ内で複数pane” によって達成する。
> WezTerm自体に新UIを埋め込まず、CYRUNE側のviewerを起動するだけで成立させる。

### 4. 実行の単一入口（bypass潰し）
- CYRUNEは `cyr run` を唯一の実行入口として設計し、runnerが
  - policy適用
  - sandbox適用
  - ledger atomic 確定
  - working再構築（候補抽出 or 手動採用）
  を行う。
- 端末側は “実行操作” を `cyr run` に寄せる（既定のショートカット、コマンドパレット等）

**bypass対策（最低限）**
- CYRUNE Terminalの既定シェルは `cyr shell` を経由し、環境をCYRUNEの実行モデルに固定する
- “実行＝cyrune runner 経由” を体験として自然にする（ユーザーが別経路を探さなくて済む）

> CITADELではOS/配布レベルでの強制（他端末/他経路の禁止）を別ADRで扱う。

### 5. 安定性（障害モード設計）
- UIクラッシュ（端末）≠ 証跡消失：ledgerは runner が atomic に確定し、端末は閲覧のみ
- runnerクラッシュ：端末は落ちず、実行のみ fail-closed（実行不可を明示）
- ledger書き込みは必ず atomic（tmp→fsync→rename）
- 追従判断：WezTerm upstream 追従は Evidence（脆弱性/重大バグ/必要機能）に基づき実施

---

## 代替案（Alternatives Considered）

1. **WezTermを“推奨”に留め、CYRUNEは端末非同梱**
   - Pros: 保守が軽い
   - Cons: 製品の主語が揺れ、UX統合が弱い。CITADEL統合時に二重実装が発生しやすい。

2. **WezTermにCYRUNE Modeを埋め込み、端末内部に統制ロジックも実装**
   - Pros: 表面的に“完全統合”が早い
   - Cons: WezTermの責務を汚し、最も壊れやすい領域へドメインロジックを混ぜる。安定性が落ち、境界が崩れる。

3. **Alacritty + Zellij を採用し、CYRUNEは別TUIで提供**
   - Pros: 分離が強い
   - Cons: 構成要素が増え、統合点が増える。供給鎖・配布責務が分散し、製品主語が弱い。

---

## 影響（Consequences）

### 良い影響
- CYRUNEが “端末製品” として成立（名前・配布・署名・更新責務がCYRUNEへ）
- WezTermの完成された責務を維持できる（汚さない）
- Control Plane（runner/ledger/working）が端末から独立し、安定性が高い
- CITADELへの発展で二重UX・二重配布が減る

### 悪い影響 / コスト
- CYRUNEが端末配布の責務を持つ（ビルド/署名/配布/脆弱性追従の判断）
- upstream追従ポリシーの運用が必要（Evidenceベースで採否を決める）

---

## 実装計画（Phased Plan）

### Phase 0（22日スコープの最小）
- CYRUNE Terminalの再商品化（名前/アイコン/署名/設定パス）
- 起動統合（paneで `cyr view evidence` / `cyr view working` を起動）
- runner/ledgerの atomic 確定と viewer 実装（最低限）

### Phase 1（安定運用）
- SBOM/署名/配布パイプライン整備
- WezTerm追従の判断基準（Evidenceテンプレ）を正典化
- bypassの追加対策（CITADELに寄せずCYRUNE範囲で可能な強制）

### Phase 2（CITADELへの橋）
- OS/配布レベル統制（管理端末前提）のADR分離
- 端末内部に踏み込む必要が出た場合の “侵食範囲” の別ADR（慎重に）

---

## オープン事項（Open Questions）
- 初期ターゲットOS（macOSのみ先行か、Linux/Windowsも同時か）
- 署名・配布方式（自己更新無しの固定配布で開始するか）
- 設定互換：WezTermの既存ユーザー設定との共存を許すか（原則：許さずCYRUNE canonical へ統一）

---

## 付記（Notes）
- 本ADRは「WezTermを汚さず取り込む」ため、端末内へのドメインロジック侵入を禁止する。
- “完全統合” は UI内製ではなく、配布統合＋起動統合＋実行入口固定で達成する。
