# CYRUNE Free v0.1: Architecture And Runtime Lines

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. 全体像

CYRUNE Free v0.1 は、次の層で構成される。

1. User / Terminal surface
2. Runtime projection layer
3. Control Plane layer
4. Kernel adapter layer
5. CRANE-Kernel contract layer

価値の中心は Runtime でも Terminal でもなく、Control Plane にある。

## 2. 各層の責務

### 2.1 User / Terminal surface

ユーザーが触る入口である。
`cyr`、viewer、terminal integration、outer launcher、bundle product surface がここに属する。
この層は表示・起動・導線を担うが、判断意味論は持たない。

### 2.2 Runtime projection layer

`cyr`、daemon、view、pack が属する。
RunRequest を Control Plane へ渡し、Evidence / Working / Policy を投影する。
Working、Policy、Ledger の意味論そのものは持たない。

### 2.3 Control Plane layer

1 ターンの accept / reject を決める本体である。
request 検証、context clear、binding 解決、Working rebuild、Gate、Execution、Citation validate、Ledger commit、Working update をここで行う。

### 2.4 Kernel adapter layer

三層メモリ、検索、埋め込み、ライフサイクルの実体を提供する。
Free 独自の運用意味論は持たない。

### 2.5 CRANE-Kernel contract layer

用途非依存の interface 群を提供する。
MemoryStore、Query、EmbeddingEngine、ForgettingPolicy、LifecycleEngine、MetricsHook を含む。

## 3. 単一入口

実行の単一入口は `cyr` である。
launcher が追加されても、launcher は outer front に留まり、`cyr` を bypass して daemon や Control Plane を直呼びしない。
bundle productization が追加されても、`cyr` 単一入口は変わらない。

## 4. 標準 turn flow

現在成立している turn flow は次の順序に固定される。

1. Runtime が RunRequest を受ける
2. Control Plane が request_id / correlation_id を検証する
3. run_id を割り当てる
4. context clear を行う
5. policy pack と binding を解決する
6. Working を再構築する
7. pre-check を行う
8. 必要に応じて検索する
9. No-LLM か approved execution adapter で実行する
10. Citation Bundle を構築する
11. citation validate と fail-closed 判定を行う
12. Ledger を atomic commit する
13. `working.json` を更新する
14. accepted または rejected の terminal outcome を返す

## 5. authority graph

### 5.1 immutable authority

packaged mode の authoritative static resource root は `BUNDLE_ROOT` のみである。
bundle-root 以外に静的 authority root を増やしてはならない。

### 5.2 mutable state

`CYRUNE_HOME` は mutable state root である。
ledger、working、memory state、generated config、materialized projection を保持してよい。
ただし authority root ではない。

### 5.3 override rule

許可される override は explicit whole-distribution-root override only である。
partial override、per-resource override、workspace fallback、ancestor scan、home-authority retry は禁止される。

## 6. BUNDLE_ROOT と CYRUNE_HOME の関係

### 6.1 BUNDLE_ROOT が持つもの

1. adapter catalog
2. policy packs
3. distro bindings
4. approved execution adapter registry / profiles
5. terminal template
6. launcher script set
7. shipping exact pin manifest / artifact set

### 6.2 CYRUNE_HOME が持つもの

1. `working/working.json`
2. `ledger/`
3. `memory/processing/`
4. `memory/permanent/`
5. `runtime/`
6. `terminal/config/wezterm.lua`
7. `embedding/**` materialized projection
8. `registry/**` materialized copy

### 6.3 重要な原則

CYRUNE_HOME 配下に copy が存在しても、それを authority に昇格させてはならない。
home 側の `embedding/**` と `registry/**` は byte-identical materialized projection としてのみ扱う。

## 7. 三層メモリの配置

### 7.1 Working

Working は小さな判断集合であり、毎ターン再構築される。
物理投影は `working/working.json` だが、source of truth ではない。

### 7.2 Processing

Processing は中期保持層である。
current shipping line では `memory/processing/` が mutable state root である。
既定保持は 42 日である。

### 7.3 Permanent

Permanent は手動昇格の長期保持層である。
current shipping line では `memory/permanent/` が mutable state root である。
non-expiring を要求する。

## 8. Policy、Citation、Ledger の位置づけ

### 8.1 Policy

deny-by-default である。
未定義 capability、未定義 rule、未定義 binding を通さない。

### 8.2 Citation

accepted output は claim 単位で Citation Bundle を持つ。
uncited claim は reject する。

### 8.3 Ledger

accepted / rejected を問わず run 単位で Evidence を残す。
append-only、atomic commit、run_id と correlation_id の一意対応を要求する。

## 9. Runtime family

### 9.1 core runtime

`cyr`、daemon、view、pack を含む。
user-facing command family は最小閉集合で固定されている。

### 9.2 packaged mode

single immutable bundle authority、doctor / launch verification-first、run-path reject と preflight failure の分離を持つ。

### 9.3 D6 native outer launcher

outer front / OS integration / launch orchestration を担う。
`cyr` を置換せず、Control Plane の意味論を持たない。

### 9.4 D7 terminal bundle productization

bundle identity、rebrand、notice、integrity、upstream intake judgment、productization failure family を担う。
runtime behavior や authority graph は変更しない。

## 10. failure surface の分離

current accepted failure surface は少なくとも次に分離される。

1. run-path unresolved / binding failure
2. preflight failure
3. launcher failure
4. productization failure
5. policy denied
6. citation denied
7. ledger commit failed
8. working update failed

これらを 1 つの曖昧なエラーへ丸めてはならない。

## 11. fail-closed 原則

### 11.1 silent success を禁止する

不明・不足・未検証・未解決は success に落としてはならない。

### 11.2 retry graph を増やさない

fallback や implicit retry を増やすと authority graph が崩れる。
そのため、explicit whole-root override 以外の retry は禁止である。

### 11.3 publicization boundary を守る

raw fs error、absolute path、host path、daemon internal error を public failure message に漏らしてはならない。

## 12. D5、D6、D7 の構造的関係

### 12.1 D5

packaged mode baseline を確立した line である。
authority root、bundle/local split、doctor / launch / run-path separation を固定した。

### 12.2 D6

D5 を前提に、native outer launcher を追加した line である。
launcher / preflight / run-path split を壊さずに outer front を足した。

### 12.3 D7

D5 と D6 を前提に、terminal bundle productization を追加した line である。
bundle identity と productization family を足したが、core semantics は変えていない。

## 13. current accepted architecture conclusion

現在の architecture は次の条件を同時に満たしている。

1. Control Plane first
2. `cyr` single-entry
3. `BUNDLE_ROOT` single immutable authority
4. `CYRUNE_HOME` mutable non-authority root
5. Working 10±2
6. citation-bound accepted output
7. append-only atomic ledger
8. fail-closed by default
9. D6 outer launcher without semantic takeover
10. D7 productization without runtime takeover
