# ENGINEERING_SPEC

この文書は、GitHub 公開リポジトリとしての CYRUNE Free v0.1 public beta の engineering-facing structure と execution contract を説明する日本語 companion です。

公開 authority surface は、root の `README.md`、`docs/GETTING_STARTED.md`、`docs/CYRUNE_Free_Public_Index.md` を優先します。この日本語 companion は、それらの claim boundary を上書きしません。

## 1. 対象範囲

この文書が扱う公開 surface は次です。

- `README.md`
- `README.ja.md`
- `docs/`
- `scripts/`
- repository-root source tree

この文書が説明するのは、公開される physical surface、public script の root 解決、`target/public-run/` に生成される state、public-run behavior を壊し得る変更点、maintenance 時の確認 predicate、そして public beta が主張しない runtime / product scope です。

この文書は、current public truth、task roadmap、native distribution、OS-level sandbox enforcement、completed classification / MAC、upper-tier feature、signing、notarization、installer distribution を再定義しません。

## 2. Public Surfaces

公開リポジトリには、次の surface があります。

1. discovery / authority surface:
   `README.md`、`docs/GETTING_STARTED.md`、`docs/FIRST_SUCCESS_EXPECTED.md`、`docs/TROUBLESHOOTING.md`、`docs/CYRUNE_Free_Public_Index.md`
2. current public docs:
   `docs/current/` と `docs/` 直下の guide / engineering docs
3. separated reference shelves:
   `docs/historical/`、`docs/deferred/`、`docs/ja/`
4. runnable source tree:
   repository root

## 3. Repository Publication Model

GitHub publication は、`main` を latest public repository surface、SemVer tag を immutable snapshot として扱います。

CYRUNE Free v0.1 public beta では、次を採用します。

- `main` は latest public surface を指します。
- `v0.1.0` は Free v0.1 public alpha の published immutable snapshot tag です。
- `v0.1.1-beta.1` は最初の public beta release-contract tag です。
- 既存の `v0.1` は version marker / compatibility tag です。
- `v0.1` branch は使用しません。
- 将来 v0.1 maintenance が必要な場合は、`release/v0.1` のような衝突しない branch 名を使用します。

## 4. Topology

Top-level layout は次です。

- `README.md`
- `README.ja.md`
- `Adapter/`
- `CRANE-Kernel/`
- `Cargo.toml`
- `Cargo.lock`
- `crates/`
- `docs/`
- `resources/`
- `scripts/`
- `tests/`

`docs/current/` は current public truth references を含みます。
`docs/deferred/` は Free v0.1 beta claim に自動採用されない future-publication / upper-tier material を含みます。
`docs/historical/` は non-authoritative historical material を含みます。
`docs/ja/` は Japanese companion documents を含みます。

repository root は runnable source tree です。

主な implementation family は次です。

1. `crates/cyrune-runtime-cli/`: `cyr` command family と user-facing runtime surface
2. `crates/cyrune-daemon/`: daemon / host execution surface
3. `crates/cyrune-control-plane/`: request validation、Working rebuild、policy gate、citation validation、ledger commit
4. `crates/cyrune-core-contract/`: request / result / denial / ID contract types
5. `resources/bundle-root/embedding/`: shipping embedding pin と static payload references

## 5. Script Root Chain

public scripts は、すべて repository root から呼び出します。

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

script は次を導出します。

1. `SCRIPT_DIR`: `scripts/`
2. `PUBLIC_ROOT`: repository root
3. `FREE_ROOT`: repository root
4. `STATE_ROOT`: `target/public-run`
5. `CYRUNE_HOME`: `target/public-run/home`

## 6. prepare-public-run Contract

`scripts/prepare-public-run.sh` は、次を満たす必要があります。

1. `target/public-run/` を再作成する
2. configured release carrier を download する
3. filename、size、SHA256 を検証する
4. unsafe tar members を拒否する
5. expected carrier manifest を要求する
6. home template を展開する
7. `cyrune-runtime-cli` と `cyrune-daemon` を build する
8. `cyr` と `cyrune-daemon` を `target/public-run/bin/` に配置する

concrete carrier URL / filename / size / SHA256 は beta release-contract pins であり、product identity authority ではありません。

## 7. doctor Contract

`scripts/doctor.sh` は、prepared public-run state に対してのみ実行します。

期待される成功条件は次です。

- exit code `0`
- JSON output
- `"status": "healthy"`

public-run state が欠落または invalid な場合、hidden fallback state を作らず fail する必要があります。

## 8. first-success Contract

`scripts/first-success.sh` は、prepared `cyr` binary を通して `cyr verify first-success` を実行します。

期待される成功条件は次です。

- exit code `0`
- verifier report JSON output
- `verified` が `true`
- `outcome` が `accepted`
- `policy_pack_id` が `cyrune-free-default`
- `evidence_id` が返る
- evidence files が `CYRUNE_HOME/ledger/evidence/<evidence_id>/` に存在する
- `CYRUNE_HOME/working/working.json` が存在する
- `CYRUNE_HOME/ledger/terminal-bindings/<evidence_id>.json` が存在し、response、evidence hashes、visible working hash を束縛する

## 9. Change Impact Map

次の変更は public-run behavior に直接影響します。

- public scripts の root resolution
- carrier URL / size / SHA256 pins
- tar member safety validation
- binary names and installation paths
- `CYRUNE_HOME` layout
- `cyr verify first-success` report contract
- evidence ledger paths
- terminal binding marker paths
- `working/working.json`

次の変更は public-reader interpretation に影響します。

- root README claim boundary
- public index reading order
- current / deferred / historical shelf placement
- Japanese companion routing
- release/tag wording

## 10. 非主張範囲

この public beta は、次を主張しません。

- production maturity
- native distributable release
- installer packaging
- concrete signing / notarization values
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface

## 11. Validation

public CI は次を確認します。

- public shell scripts parse
- beta release-contract static predicates
- Rust formatting
- Rust workspace check
- Rust lint with warnings denied

runtime first-success validation は `docs/FIRST_SUCCESS_EXPECTED.md` に記載され、local evidence は `target/public-run/home/` に生成されます。

beta release-contract criteria は `docs/BETA_CRITERIA.md` に定義されています。
