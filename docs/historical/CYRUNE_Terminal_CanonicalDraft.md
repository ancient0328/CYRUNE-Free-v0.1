# CYRUNE Terminal（WezTermベース再商品化）— Canonical v0 Draft

**日付 (JST)**: 2026-03-01
**状態（当時）**: Draft（正典候補）
**現在の権威状態**: Historical draft / non-authoritative
**取り扱い**: 2026-04-11 JST の D7 closeout 後、この文書は current accepted canonical ではない。初期 canonical 候補と設計背景の参照に限定する。現行 authority は `free/v0.1/dev-docs/04-implementation-notes/TERMINAL_BUNDLE_PRODUCTIZATION_CANONICAL.md`、`free/v0.1/dev-docs/04-implementation-notes/D7_BUNDLE_IDENTITY_AND_REBRAND_CANONICAL.md`、`free/v0.1/dev-docs/90-reports/20260411-terminal-D7-terminal-bundle-productization-proof.md` である。
**対象**: CYRUNE v0.1（22日スコープを含む）

---

## 0. 正典の主語

この文書が固定するもの：

- **CYRUNE Terminal** は WezTerm をベースに「CYRUNEとして再商品化」する（ユーザーに WezTerm を名乗らない）
- WezTermの端末責務を汚さないため、**統制ロジックは端末に埋め込まない**
- **実行の単一入口**は `cyr`（runner/daemon）であり、証跡（ledger）がSSoT
- Terminal統合は「配布統合 + 起動統合 + 入口固定」で達成する

---

## 1. リポジトリ構造（Workspace / crates / third_party）

### 1.1 ルート構成

```

cyrune/
Cargo.toml
Cargo.lock
README.md
LICENSES/
THIRD_PARTY_NOTICES.md
wezterm-MIT.txt
third_party/
wezterm/                   # upstream固定（submodule or subtree）
crates/
cyr/                       # `cyr` CLI（ユーザーが触る入口）
cyrune-daemon/             # 実行制御・policy・sandbox・ledger確定（SSoT側）
cyrune-ledger/             # ledger/workingの型・atomic IO・hash chain（純ライブラリ）
cyrune-policy/             # policy pack 読み込み・評価（純ライブラリ）
cyrune-sandbox/            # OS別sandbox適用（純ライブラリ、v0.1は最小）
cyrune-view/               # viewer（evidence/working/policy explain）TUI/CLI
cyrune-pack/               # パック管理（policy pack, templates, wezterm config生成）
apps/
terminal/                  # CYRUNE Terminal（WezTermビルド/配布定義）
xtask/                       # ビルド/署名/配布/SBOM生成（再現ビルドの司令塔）
tools/
sbom/                      # SBOM関連テンプレ、スクリプト
docs/
adr/
20260301-cyrune-terminal-wezterm-integration.md
canonical/
cyrune-terminal-canonical.md   # ← 本書を置く想定

```

### 1.2 WezTermの取り込み方式（固定）

- `third_party/wezterm/` は **submodule推奨**（commit hashで供給鎖固定）
- CYRUNE側の変更は「配布に必要な最小差分」のみ
  - branding（アプリ名/アイコン/バンドルID）
  - config探索パスの寄せ（`~/.cyrune/terminal/`）
  - About/Licenses導線（Third-party licenses）
- **統制ロジックを入れるPRは禁止**（policy/sandbox/ledger/working等）

> upstream追従は「Evidenceベースで採否」。
> 定期追従はしない。追従作業は `docs/evidence/wezterm-sync/`（後述）に証跡を残す。

---

## 2. データ/設定のCanonicalパス（SSoT）

### 2.1 CYRUNEホーム

- `CYRUNE_HOME = ~/.cyrune`
- Windows等は別途OS標準に寄せるが、正典では **論理名**としてCYRUNE_HOMEを使う

```

~/.cyrune/
version.json
terminal/
config/wezterm.lua         # 生成物（canonical）
state/                     # 端末関連の状態（必要なら）
ledger/
manifests/                 # 走査用index（壊れても再構築）
evidence/
EVID-<id>/
manifest.json
policy.json
run.json
stdout.log
stderr.log
artifacts/...
hashes.json
working/
working.json               # Working 10±2（canonical）
candidates.json            # 抽出候補（任意）
packs/
policy/
default/                 # Consumer/Custom等（pack）
medical/
finance/
templates/
cache/
downloads/
build/

````

### 2.2 Evidence ID（決定的識別）

- Evidenceは `EVID-<u64>` または `EVID-<time+rand>` の閉集合形式
- v0.1は衝突しない範囲でよいが、**順序は単調増加**が望ましい（監査上）

---

## 3. 実行モデル：単一入口 `cyr` と daemon/runner

### 3.1 プロセス責務（固定）

- `cyr`（CLI）
  - ユーザー操作の入口
  - daemon起動/接続
  - viewer呼び出し
- `cyrune-daemon`
  - **実行の単一入口**
  - policy評価・sandbox適用
  - stdout/stderr収集、artifact収集
  - ledger atomic確定
  - working候補抽出（v0.1はルールベース）
- `cyrune-view`
  - ledger/working閲覧、policy explain
  - v0.1はTUIでなくてもよい（まずはpager/CLIでも可）

> 端末（CYRUNE Terminal）は、これらを起動・表示するだけ。
> 端末に統制ロジックを持たせない。

### 3.2 IPC契約（閉集合）

daemonは「閉集合コマンド」だけ受け付ける。未知コマンドは禁止（fail-closed）。

**コマンド（例）**
- `Run { adapter, argv, cwd, env_overrides, policy_pack, io_mode }`
- `Cancel { run_id }`
- `Tail { evidence_id, stream, from_offset }`
- `GetEvidence { evidence_id }`
- `ListEvidence { limit, cursor }`
- `GetWorking {}`
- `UpdateWorking { slots, reason }`
- `ExplainPolicy { policy_pack, last_denial_id? }`
- `Health {}`

**レスポンス（例）**
- `RunAccepted { run_id, evidence_id }`
- `RunRejected { denial_id, rule_id, message, remediation }`
- `Evidence { ...manifest... }`
- `Working { slots... }`

> “曖昧な非同期Error”は禁止。相関不能は禁止。
> request_id/response_toを top-level に持つ（arcRTCの相関決定性と同型）。

---

## 4. Ledger（証跡）の正典仕様（atomic / 改竄検知）

### 4.1 Evidenceディレクトリの構造（固定）

`~/.cyrune/ledger/evidence/EVID-<id>/`

必須ファイル：
- `manifest.json`（索引：SSoT）
- `run.json`（実行コマンド・cwd・env・exit）
- `policy.json`（適用policy pack + 判定結果）
- `stdout.log`, `stderr.log`
- `hashes.json`（改竄検知）

任意：
- `artifacts/`（diff、test結果、生成物）
- `working_delta.json`（Working更新の差分）

### 4.2 atomic確定手順（固定）

1) `EVID-<id>.tmp/` を作成
2) すべてのファイルを `.tmp` に書く
3) `fsync`（ディレクトリも含む）
4) `rename(EVID-<id>.tmp -> EVID-<id>)`
5) `manifests/index.jsonl` に append（壊れても再構築可能）

> 「確定できなかったEvidence」は存在してはならない。確定不能はfail-closedでRun自体を失敗させる。

### 4.3 hashes.json（最小改竄検知）

- 各ファイルのSHA256
- 前Evidenceのハッシュ（hash chain）を入れるのが望ましい
  - v0.1では `prev_hash` が無い初回だけ例外

---

## 5. Working（10±2）正典仕様（毎ターン再構築のための核）

### 5.1 形式（固定）

`~/.cyrune/working/working.json`

```json
{
  "version": 1,
  "slots": [
    { "id": 1, "kind": "decision", "text": "...", "source_evidence": "EVID-10", "ts": "..." },
    { "id": 2, "kind": "todo",     "text": "...", "source_evidence": "EVID-11", "ts": "..." }
  ],
  "limit": 12
}
````

* slotsは最大12（10±2）。超過は拒否（fail-closed）。
* kindは閉集合（例：decision/todo/assumption/constraint/context/definition/command）

### 5.2 更新手順（固定）

* Working更新は `UpdateWorking` のみで行う
* 更新は ledger に必ず紐付く（`source_evidence`必須）
* v0.1は手動採用でよい（candidate抽出は補助）

---

## 6. Policy Pack（最小正典）

### 6.1 位置

`~/.cyrune/packs/policy/<pack>/`

### 6.2 最小構造（例）

```
packs/policy/default/
  pack.toml
  rules/
    10-exec.toml
    20-fs.toml
    30-net.toml
```

`pack.toml`（例）

* pack id / version
* deny-by-defaultの宣言
* 許可するadapter一覧
* ルールの閉集合

ルールは必ず

* `rule_id`（閉集合）
* `predicate`（決定的）
* `denial`（理由＋remediation）
  を持つ。

---

## 7. Sandbox（v0.1は最小、しかし入口は固定）

### 7.1 v0.1の方針

* macOS/Linux：まずは「ファイルアクセス許可範囲」「ネットワーク禁止（任意）」「環境変数固定」から始める
* OS完全隔離（seccomp/bpf/seatbelt等）は v0.2+ に送る
* ただしAPIは固定：`ApplySandbox(spec) -> Result`

> v0.1で無理に完全隔離をやらない。代わりに「単一入口＋証跡＋deny-by-default」で担保する。

---

## 8. CYRUNE Terminal（WezTermベース）— 配布仕様

### 8.1 生成物

* macOS: `CYRUNE.app`（内部に `cyrune` 実体）
* CLI: `cyr`（別途 / 同梱）

### 8.2 バンドル同梱（固定）

CYRUNE Terminal の配布物には以下を同梱できる（v0.1で必須ではないが、方向性として固定）：

* `cyr` CLI
* `cyrune-daemon`
* `cyrune-view`
* default policy pack
* wezterm.lua generator（または生成済みconfig）

### 8.3 Third-party licenses（必須）

* `LICENSES/wezterm-MIT.txt`
* `LICENSES/THIRD_PARTY_NOTICES.md`
* About画面 or メニューから参照可能にする

---

## 9. 起動統合：wezterm.lua 生成仕様（固定）

### 9.1 生成先

* `${CYRUNE_HOME}/terminal/config/wezterm.lua`（canonical）

### 9.2 起動時の統合レイアウト（v0.1最小）

* Tab 1: “Workspace”

  * Pane A: `cyr shell`（ユーザーシェル）
  * Pane B: `cyr view evidence --follow`（実行ログ/証跡）
  * Pane C: `cyr view working --follow`（Working表示）
* Tab 2: “Policy”

  * Pane: `cyr view policy --pack default`

> “完全統合”は WezTerm内部UIの追加ではなく、既存paneにCYRUNE viewerを配置することで達成する。

### 9.3 wezterm.lua（雛形：正典）

（WezTerm自体のAPI差分は将来あり得るが、CYRUNE側は生成器を正典とする）

```lua
local wezterm = require 'wezterm'
local act = wezterm.action

local CYRUNE_HOME = os.getenv("CYRUNE_HOME") or (os.getenv("HOME") .. "/.cyrune")
local CYR = CYRUNE_HOME .. "/bin/cyr"  -- 配布時に同梱する前提（なければPATH fallback）

local function cyr_cmd(args)
  return { CYR, unpack(args) }
end

local config = {}
config.default_prog = cyr_cmd({ "shell" })

-- Keybindings are closed-set: keep minimal & deterministic
config.keys = {
  { key = "E", mods = "CTRL|SHIFT", action = act.SpawnTab { args = cyr_cmd({ "view", "evidence", "--follow" }) } },
  { key = "W", mods = "CTRL|SHIFT", action = act.SpawnTab { args = cyr_cmd({ "view", "working", "--follow" }) } },
  { key = "P", mods = "CTRL|SHIFT", action = act.SpawnTab { args = cyr_cmd({ "view", "policy" }) } },
}

wezterm.on("gui-startup", function(cmd)
  local mux = wezterm.mux
  local tab, pane, window = mux.spawn_window {
    args = cyr_cmd({ "shell" }),
  }

  -- Split right: evidence viewer
  local right = pane:split { direction = "Right", size = 0.42, args = cyr_cmd({ "view", "evidence", "--follow" }) }

  -- Split bottom on right: working viewer
  right:split { direction = "Bottom", size = 0.50, args = cyr_cmd({ "view", "working", "--follow" }) }

  -- Second tab: policy
  local tab2 = window:spawn_tab { args = cyr_cmd({ "view", "policy", "--pack", "default" }) }
  tab2:set_title("Policy")

  tab:set_title("Workspace")
end)

return config
```

### 9.4 生成器（cyrune-pack）の責務

* OS差分（パス、shell）を吸収して `wezterm.lua` を生成
* 生成内容は `pack` と `version` に紐付けて ledger に記録可能にする（任意）

---

## 10. `cyr` コマンド体系（最小閉集合）

### 10.1 v0.1で必須

* `cyr shell`
* `cyr run <adapter> -- <cmd...>`
* `cyr view evidence [--follow]`
* `cyr view working [--follow]`
* `cyr view policy [--pack <name>]`
* `cyr doctor`（環境診断：CYRUNE_HOME、daemon、terminal config、packs）

### 10.2 adapter名（閉集合）

* `claude-code`
* `codex`
* `raw`（任意：一般コマンド実行。ただしpolicyで厳格制限）

---

## 11. ビルド＆リリース（22日スコープの固定案）

### 11.1 原則

* 自己更新機能は持たせない（ANSYS型の更新運用）
* 配布物は署名し、SBOMを生成し、Evidenceとして残す

### 11.2 xtask（司令塔）

* `cargo xtask build terminal --release`

  * third_party/wezterm をビルドして CYRUNE Terminal を生成
  * `cyr`, `cyrune-daemon`, `cyrune-view` を同梱（可能なら）
  * `wezterm.lua` を生成して同梱 or 初回起動で生成
* `cargo xtask sbom`

  * SBOMを生成（最小：依存一覧 + commit hash）
* `cargo xtask sign`

  * macOS署名（開発段階では adhoc 可、配布は正式署名）

### 11.3 Evidence（配布証跡）

* `docs/evidence/releases/<version>/`

  * build環境情報（OS、rustc、cargo、commit hash）
  * SBOM
  * 署名情報
  * 配布物のSHA256
  * upstream wezterm commit hash

---

## 12. WezTerm追従（Evidence Gate運用）

### 12.1 追従は定期ではない（固定）

追従条件（いずれか）：

* セキュリティ（CVE/影響大）
* 重大バグ（再現証跡あり）
* CYRUNEで必要な端末機能（Evidenceで正当化）

### 12.2 証跡テンプレ

`docs/evidence/wezterm-sync/<date>-<id>/`

* `motivation.md`（なぜ追従するか）
* `upstream_diff.md`（差分要約）
* `risk_assessment.md`（影響と回避）
* `repro_steps.md`（再現手順）
* `decision.md`（採否）

---

## 13. 22日スコープの “Done” 定義（v0.1最小合格）

最低限これが揃えば「CYRUNE Terminal 統合が成立」とみなす：

1. CYRUNE Terminal（WezTermベース）が **CYRUNEとして起動**する

   * アプリ名/バイナリ名/アイコン/設定パスがCYRUNE
2. 起動時に Workspace レイアウトが出る

   * `cyr shell` が開き、`cyr view evidence` と `cyr view working` が見える
3. `cyr run` が policy判定を通し、ledger を atomic 確定する
4. `cyr view evidence` が確定済みEvidenceを閲覧できる
5. Third-party licenses が同梱され、参照可能である

---

## 14. 禁止事項（fail-closed）

* WezTerm側に統制ロジックを実装するPRは禁止
* ledger確定がatomicでない実装は禁止
* Working上限（10±2）を破る更新は禁止
* IPCの未知コマンド受理は禁止

---
