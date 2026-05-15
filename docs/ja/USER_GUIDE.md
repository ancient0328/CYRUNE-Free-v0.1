# USER_GUIDE

この文書は、GitHub 公開リポジトリとしての CYRUNE Free v0.1 public beta の使い方を説明する日本語 companion です。

英語の公開入口は、リポジトリ root の `README.md`、`docs/GETTING_STARTED.md`、`docs/FIRST_SUCCESS_EXPECTED.md`、`docs/TROUBLESHOOTING.md` です。この日本語 companion は、それらの claim boundary を上書きしません。

## 1. このパッケージでできること

CYRUNE Free v0.1 は、single-user 向けの public beta package です。

この公開面では、ローカル host 上で pinned beta carrier から public-run state を準備し、`cyr doctor` を実行し、packaged Free v0.1 Control Plane path を通して no-LLM の first-success semantic verifier を 1 回実行できます。

この public beta は、production maturity、native distribution、OS-level sandbox enforcement、enforcement-complete classification / MAC、より広い製品ラインの scope、signing、notarization、installer distribution を主張しません。

## 2. リポジトリ構成

公開リポジトリ root には、次の surface があります。

- `README.md`
- `README.ja.md`
- `docs/`
- `scripts/`
- `free/`

通常の利用導線は次の文書と script です。

- `docs/GETTING_STARTED.md`
- `docs/FIRST_SUCCESS_EXPECTED.md`
- `docs/BETA_CRITERIA.md`
- `docs/TROUBLESHOOTING.md`
- `scripts/prepare-public-run.sh`
- `scripts/doctor.sh`
- `scripts/first-success.sh`

## 3. 前提

host には次が必要です。

- `bash`
- `curl`
- `python3`
- `tar`
- `cargo`
- `install`
- configured release carrier URL へ到達できる network access
- executable permission を保持できる local filesystem

前提が欠けている場合、`prepare-public-run.sh` は成功を偽装せず fail する必要があります。

## 4. 実行順序

リポジトリ root から、次の順序で実行します。

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

順序の変更や step の省略は、この public first-success path の対象外です。

## 5. 各 step の意味

### 5.1 prepare-public-run

この step は configured release carrier を download し、filename、size、SHA256、tar member safety を検証し、home template を public-run state へ展開し、Free source tree から runtime binaries を build し、`cyr` と `cyrune-daemon` を `target/public-run/bin/` に配置します。

### 5.2 doctor

この step は、準備済み public-run state に対して `cyr doctor` を実行します。

期待結果は、`"status": "healthy"` を含む JSON object です。

### 5.3 first-success

この step は semantic verifier を実行します。

```bash
cyr verify first-success
```

期待結果は、`verified: true`、`outcome: "accepted"`、`correlation_id`、`run_id`、`evidence_id`、`policy_pack_id`、`state_root`、`cyrune_home` を含む JSON report です。

生成される evidence path と output field の詳細は、`docs/FIRST_SUCCESS_EXPECTED.md` を参照します。

## 6. 期待される local state

`prepare-public-run.sh` 成功後、public-run state は次に作成されます。

```text
target/public-run/
```

`first-success.sh` 成功後、主に次を確認します。

- `target/public-run/home/ledger/manifests/index.jsonl`
- `target/public-run/home/ledger/evidence/<evidence_id>/`
- `target/public-run/home/ledger/terminal-bindings/<evidence_id>.json`
- `target/public-run/home/working/working.json`

## 7. 失敗時の扱い

`prepare-public-run.sh` が失敗した場合、`doctor.sh` へ進みません。

`doctor.sh` が失敗した場合、まず `prepare-public-run.sh` を再実行します。

`first-success.sh` が失敗した場合、`prepare-public-run.sh` を再実行し、`doctor.sh` が通ることを確認したうえで、`first-success.sh` を再実行します。

## 8. 非主張範囲

first-success の成功が示すのは、C5 verifier が response、evidence bundle、terminal binding marker、visible working projection を accepted として検証したことです。beta release contract 全体には、`docs/BETA_CRITERIA.md` に定義された source、carrier asset、CI、public docs、Closed Gate evidence も必要です。

それは次を証明しません。

- native distributable release
- installer packaging
- signing / notarization
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- より広い製品ラインの functionality
