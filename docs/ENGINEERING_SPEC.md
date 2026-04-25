# ENGINEERING_SPEC

この文書は、`Distro/CYRUNE/public/free-v0.1/` に含まれる公開パッケージの構成と実行面を、エンジニア向けに詳細説明する文書です。
この文書は authority surface や canonical source を置き換えません。公開入口は root `README.md` と `GETTING_STARTED.md`、current accepted public truth の reference は `CYRUNE_Free_Public_Index.md` を起点に確認してください。

## 1. Scope

この文書が扱うのは、公開パッケージに含まれる次の要素と、その上で成立する package-level implementation contract です。

- `README.md`
- `docs/`
- `scripts/`
- `free/v0.1/0/`

この文書を読んだエンジニアは、少なくとも次を誤解なく把握できる状態を目標にします。

1. 公開パッケージの physical surface
2. 3 script の exact 実行順と観測点
3. `target/public-run/` に生成される state の形
4. どこを変えると public-run behavior が崩れるか
5. 保守・検証時に最低限確認すべき predicate
6. public alpha が主張しない runtime / product scope

この文書は次を再定義しません。

- current accepted public truth
- task-level roadmap
- current-state claim
- native distributable
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface
- signing / notarization / concrete release owner values

## 2. Package Surfaces

公開パッケージには、役割の異なる 4 つの surface があります。

1. discovery / authority surface
   `README.md`、`docs/GETTING_STARTED.md`、`docs/FIRST_SUCCESS_EXPECTED.md`、`docs/TROUBLESHOOTING.md`、`docs/CYRUNE_Free_Public_Index.md`
2. current public docs
   `docs/current/` 配下の Free v0.1 current public truth と `docs/` 直下の guide / engineering files
3. separated reference shelves
   `docs/historical/` と `docs/deferred/`
4. runnable source tree
   `free/v0.1/0/`

この文書は 2、3、4 を説明します。
1 の authority surface 自体を変更するものではありません。

## 2.1 Repository Publication Model

GitHub publication uses `main` as the latest public repository surface and SemVer tags as immutable snapshots.

For CYRUNE Free v0.1 public alpha:

- `main` points to the latest public surface.
- `v0.1.0` is the immutable snapshot tag for the Free v0.1 public alpha publication.
- Existing `v0.1` is treated as a version marker / compatibility tag.
- A `v0.1` branch is not used, because it collides semantically and operationally with the existing `v0.1` tag name.
- If maintenance branching becomes necessary later, the branch name must avoid tag-name collision, for example `release/v0.1`.

## 3. Engineer-Facing Reading Contract

この文書は、`GETTING_STARTED.md` と `TROUBLESHOOTING.md` の代替ではありません。
ただし、それら 2 文書だけでは不足する次の engineering question には、この文書で答える必要があります。

1. 3 script はどの root を基準に動くか
2. 何が生成されれば prepare が通ったと判断できるか
3. `doctor` と `first-success` は何を stdout に返すか
4. どの key / path / mode が public-run behavior の invariant か
5. 変更時にどこを観測すれば regression を判断できるか

## 4. Package Topology

公開パッケージの top-level layout は次です。

- `README.md`
- `docs/`
- `scripts/`
- `free/`

`docs/` には、public index、補助文書、first-success expected result、current truth shelf、separated reference shelves が含まれます。
`scripts/` には、公開ユーザー向けの 3 本の shell script が含まれます。
`free/v0.1/0/` には runnable source tree が含まれます。

`docs/current/` は current Free v0.1 public truth shelf です。
`docs/historical/` は historical / non-authoritative corpus です。
`docs/deferred/` は current Free v0.1 public truth に自動採用しない high-tier / deferred-publication corpus です。

`free/v0.1/0/` のうち、public-run behavior を理解するうえで最低限見るべき実装 family は次です。

1. `crates/cyrune-runtime-cli/`
   `cyr` command family、`doctor`、`run --no-llm` を持つ user-facing runtime surface
2. `crates/cyrune-daemon/`
   `cyrune-daemon` binary を持つ host / daemon surface
3. `crates/cyrune-control-plane/`
   request validation、Working rebuild、Gate、Citation、Ledger を持つ product-value core
4. `crates/cyrune-core-contract/`
   request / result / denial / ID family を持つ closed-set contract surface
5. `resources/bundle-root/embedding/`
   shipping exact pin manifest と embedding artifact family を持つ immutable static payload

## 5. Runtime Roots

公開スクリプトは caller の cwd を authority にせず、自身の配置から root を解決します。
各 script で使われる root chain は次です。

- `SCRIPT_DIR`
- `PUBLIC_ROOT`
- `FREE_ROOT`
- `STATE_ROOT`
- `CYRUNE_HOME`

実行結果として、`free/v0.1/0/target/public-run/` 配下に public-run state が構成されます。
この state root には、`bin/` と `home/` が生成されます。

root chain の意味は次の通りです。

1. `SCRIPT_DIR`
   実行された script 自身の配置
2. `PUBLIC_ROOT`
   展開済み package root
3. `FREE_ROOT`
   runnable source tree
4. `STATE_ROOT`
   prepare によって再構成される public-run state root
5. `CYRUNE_HOME`
   `doctor` / `first-success` が固定して使う runtime home

## 6. Generated State Model

`prepare-public-run.sh` の成功後、少なくとも次の path family が存在していなければなりません。

```text
free/v0.1/0/target/public-run/
├── bin/
│   ├── cyr
│   └── cyrune-daemon
└── home/
```

この state model で重要なのは次です。

1. `target/public-run/` は rerun ごとに再初期化される
2. binary source of truth は `free/v0.1/0/target/release/` 側である
3. runnable state source of truth は `target/public-run/bin/` と `target/public-run/home/` である
4. `doctor` と `first-success` は caller cwd ではなく、この state model に依存して動く

## 7. Script Responsibilities And Exact Observables

### 7.1 `scripts/prepare-public-run.sh`

この script は、公開パッケージ内の Free source tree を使って public-run state を再構成します。
実際に行う処理は次です。

- `free/v0.1/0/` 配下へ移動する
- `target/public-run/` を再初期化する
- configured release carrier を download する
- carrier filename / size / SHA256 を検証する
- tar member safety を検証する
- carrier home template を `target/public-run/home/` に展開する
- `cargo build --quiet --release` で `cyr` と `cyrune-daemon` を build する
- `target/public-run/bin/` に 2 つの binary を install する

エンジニアが観測すべき predicate は次です。

1. fixed cwd は `"$FREE_ROOT"` である
2. carrier filename が configured filename と一致する
3. carrier size が configured size と一致する
4. carrier SHA256 が configured SHA256 と一致する
5. archive が absolute path、parent traversal、symlink、hardlink、device file を含まない
6. expected release manifest が carrier 内に存在する
7. exit code は `0` である
8. `"$STATE_ROOT/bin/cyr"` が存在する
9. `"$STATE_ROOT/bin/cyrune-daemon"` が存在する
10. `"$STATE_ROOT/home/"` が存在する

### 7.2 `scripts/doctor.sh`

この script は、`CYRUNE_HOME` を `target/public-run/home` に固定したうえで、`cyr doctor` を実行します。
この step の目的は、public-run state が診断可能な状態にあることを確認することです。

エンジニアが観測すべき predicate は次です。

1. fixed cwd は `"$FREE_ROOT"` である
2. `CYRUNE_HOME` は `"$STATE_ROOT/home"` に固定される
3. exact command は `"$STATE_ROOT/bin/cyr" doctor` である
4. stdout は raw JSON object を pass-through する
5. stdout に wrapper key や banner を追加しない
6. exit code は `0` である
7. stdout JSON object の `"status"` は `"healthy"` である

### 7.3 `scripts/first-success.sh`

この script は、`CYRUNE_HOME` を同じ state root に固定したうえで、`cyr run --no-llm --input "ship-goal public first success"` を実行します。
この step の目的は、公開パッケージが no-LLM mode で first success まで到達できることを確認することです。

エンジニアが観測すべき predicate は次です。

1. fixed cwd は `"$FREE_ROOT"` である
2. `CYRUNE_HOME` は `"$STATE_ROOT/home"` に固定される
3. exact command は `"$STATE_ROOT/bin/cyr" run --no-llm --input "ship-goal public first success"` である
4. stdout は raw JSON object を pass-through する
5. exit code は `0` である
6. stdout JSON object の `correlation_id`、`run_id`、`evidence_id`、`policy_pack_id` は non-empty である
7. `policy_pack_id` は `cyrune-free-default` である

### 7.4 Shared Fail-Closed Rule

3 script 共通の engineering rule は次です。

1. non-zero exit code は fail である
2. failure 時に synthetic success JSON を stdout に出してはならない
3. remediation key は `docs/TROUBLESHOOTING.md` の 3 heading に限定される

## 8. Expected Execution Order

公開パッケージの canonical sequence は、`docs/GETTING_STARTED.md` が正です。
順序は固定で、次の 3 段です。

1. `prepare-public-run.sh`
2. `doctor.sh`
3. `first-success.sh`

この文書は、その sequence の意味と内部構成を補助説明するだけで、順序自体を再定義しません。

## 9. Verification Hooks For Engineers

保守・変更後に、最低限次を確認してください。

1. `prepare-public-run.sh` が空 stdout / exit code `0` で終わり、`target/public-run/bin/cyr`、`target/public-run/bin/cyrune-daemon`、`target/public-run/home/` が再生成されること
2. `doctor.sh` が raw JSON object を返し、`"status":"healthy"` を含むこと
3. `first-success.sh` が raw JSON object を返し、`correlation_id`、`run_id`、`evidence_id`、`policy_pack_id` を持つこと
4. `policy_pack_id` が `cyrune-free-default` であること
5. script が caller cwd に依存しないこと
6. script が success 時に wrapper stderr / wrapper stdout を加えないこと

`first-success.sh` の public-facing expected result は `docs/FIRST_SUCCESS_EXPECTED.md` が説明します。

## 10. Operational Boundaries

公開パッケージの engineering surface は、次の前提に依存します。

- shell は `bash`
- build host に `curl`、`python3`、`tar`、`cargo`、`install` が存在する
- configured release carrier URL に到達できる
- local filesystem が regular file copy と executable mode 保存を行える

`scripts/prepare-public-run.sh` の concrete carrier URL / filename / size / SHA256 は operational pin であり、product identity authority ではありません。

次はこのパッケージに含めません。

- internal operational corpus
- private development repository の全量
- native installer / archive variation
- signing / notarization workflow
- release owner 固有の concrete value handling
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface

## 11. Failure Handling Boundary

失敗時の remediation の canonical source は `docs/TROUBLESHOOTING.md` です。
この文書は、failure taxonomy や内部運用フローを増やしません。
公開面での対応は、次に限定されます。

- `prepare-public-run.sh` の再実行
- `doctor.sh` の再実行
- `first-success.sh` の再実行
- host build prerequisites の確認

エンジニアが failure を読む時の最小ルールは次です。

1. `prepare-public-run.sh` failure は、state 再初期化、build host prerequisite、source tree 完全性のどれかを疑う
2. `doctor.sh` failure は、`prepare-public-run.sh` で作られた state の欠損を疑う
3. `first-success.sh` failure は、`doctor.sh` を通った state かどうかを先に確認する

## 12. Change Impact Map

次を変えると public-run behavior に直結します。

1. root resolution
   `SCRIPT_DIR` / `PUBLIC_ROOT` / `FREE_ROOT` / `STATE_ROOT` / `CYRUNE_HOME`
2. generated state model
   `target/public-run/bin/` と `target/public-run/home/`
3. stdout contract
   `doctor.sh` / `first-success.sh` の raw JSON pass-through
4. binary set
   `cyr` と `cyrune-daemon` の 2 binary
5. policy identity
   `policy_pack_id = cyrune-free-default`

## 13. Non-goals

この文書は次を目的としません。

- canonical document の代替
- release package channel や GitHub release metadata の再定義
- test roadmap の代替
- implementation status claim
- ship-goal status claim
- full Control OS maturity claim
- runtime feature roadmap

## 14. Reading Order

エンジニアとして読む順序は次を推奨します。

1. `README.md`
2. `docs/GETTING_STARTED.md`
3. `docs/FIRST_SUCCESS_EXPECTED.md`
4. `docs/CYRUNE_Free_Public_Index.md`
5. `docs/current/CYRUNE-Free_Canonical.md`
6. `docs/current/CYRUNE.md`
7. `docs/TROUBLESHOOTING.md`
8. `free/v0.1/0/`

この文書は、その後に package-level engineering explanation として参照してください。
