# USER_GUIDE

この文書は、GitHub 公開リポジトリ root に含まれる公開パッケージの使い方を、一般ユーザー向けに説明する日本語 companion 文書です。
英語 primary は `USER_GUIDE.md` です。
この文書は root `README.md`、`GETTING_STARTED.md`、`CYRUNE_Free_Public_Index.md` を置き換えません。
実行順序の canonical source は `GETTING_STARTED.md`、first-success の期待結果は `FIRST_SUCCESS_EXPECTED.md`、失敗時の remediation の canonical source は `TROUBLESHOOTING.md` です。

## 1. このパッケージでできること

CYRUNE Free v0.1 は、単一ユーザー向けの public alpha 公開パッケージです。
このパッケージでは、手元の host 上に public-run 用 state を作り、`cyr` を使って no-LLM mode の first success まで到達できます。

この public alpha は、native distributable、OS-level sandbox enforcement、enforcement-complete classification / MAC、Pro / Enterprise / CITADEL scope を主張しません。

## 2. パッケージに含まれるもの

パッケージの top-level には次が含まれます。

- `README.md`
- `docs/`
- `scripts/`
- `free/`

通常の利用で主に触るのは次です。

- `docs/GETTING_STARTED.md`
- `docs/FIRST_SUCCESS_EXPECTED.md`
- `docs/TROUBLESHOOTING.md`
- `scripts/prepare-public-run.sh`
- `scripts/doctor.sh`
- `scripts/first-success.sh`

## 3. 開始前に確認すること

このパッケージは、展開済み package root を local terminal で操作する前提です。
少なくとも次を満たしてください。

- `bash` が使えること
- `curl` が使えること
- `python3` が使えること
- `tar` が使えること
- `cargo` が使えること
- `install` command が使えること
- configured release carrier URL に到達できること
- executable permission を保持できる local filesystem であること

この前提が満たされないと、`prepare-public-run.sh` は成功しません。
また、開始前には package root に `README.md`、`docs/`、`scripts/`、`free/` が見えていることを確認してください。

## 4. 使い始める順序

実行順序そのものは `docs/GETTING_STARTED.md` が正です。
利用の流れは次の 3 段だけです。

1. `prepare-public-run.sh` で public-run 用 state を作る
2. `doctor.sh` で診断する
3. `first-success.sh` で no-LLM mode の first success を確認する

順序を変えたり、途中を飛ばしたりしないでください。

この文書だけで開始したい場合は、package root で次をそのまま実行してください。

```bash
./scripts/prepare-public-run.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

## 5. 各 step の意味

### 5.1 prepare-public-run

この step では、release carrier を download / filename check / size check / SHA256 check / tar member safety check で検証し、home template を public-run state に展開した後、公開パッケージ内の Free source tree から必要な binary を build します。
成功すると、`free/v0.1/0/target/public-run/bin/cyr`、`free/v0.1/0/target/public-run/bin/cyrune-daemon`、`free/v0.1/0/target/public-run/home/` が揃います。

### 5.2 doctor

この step では、作成された public-run state を対象に `cyr doctor` を実行し、診断が通るかを確認します。
成功すると、raw JSON object が表示され、その中の `"status"` が `"healthy"` になります。

### 5.3 first-success

この step では、同じ public-run state を使って `cyr run --no-llm` を実行し、公開パッケージの first success まで進めるかを確認します。
成功すると、raw JSON object が表示され、`correlation_id`、`run_id`、`evidence_id`、`policy_pack_id` が入ります。`policy_pack_id` は `cyrune-free-default` である必要があります。

## 6. 成功の見方

この文書は accepted predicate 自体を定義しません。
成功判定の canonical source は公開 script contract を固定している current canonical です。
利用者向けの目安としては、次の状態を成功として読んでください。

1. `prepare-public-run.sh` が prompt に戻り、`free/v0.1/0/target/public-run/bin/` と `free/v0.1/0/target/public-run/home/` が揃っている
2. `doctor.sh` が raw JSON object を返し、`"status":"healthy"` が見える
3. `first-success.sh` が raw JSON object を返し、`correlation_id`、`run_id`、`evidence_id`、`policy_pack_id` が見える

この 3 つが揃わない場合は、成功とみなさず `TROUBLESHOOTING.md` を参照してください。

first-success の出力と生成物の読み方は `docs/FIRST_SUCCESS_EXPECTED.md` を参照してください。

## 7. 失敗したとき

失敗時の canonical remediation は `docs/TROUBLESHOOTING.md` が正です。
まず次を守ってください。

1. `doctor.sh` が失敗したら、先に `prepare-public-run.sh` をやり直す
2. `first-success.sh` が失敗したら、`prepare-public-run.sh` と `doctor.sh` を通した後に再実行する
3. host build prerequisites に不足がある場合は、それを解消してから再実行する

失敗時の読み方は次です。

1. command が途中で止まる、または non-zero exit code で終わる場合は fail です
2. `doctor.sh` の JSON に `"status":"healthy"` が出ない場合は fail です
3. `first-success.sh` の JSON に `correlation_id`、`run_id`、`evidence_id`、`policy_pack_id` が揃わない場合は fail です

この文書は `TROUBLESHOOTING.md` の代わりにはなりません。困った時は必ずそちらを正として見てください。

## 8. このパッケージに含まれないもの

次はこの公開パッケージに含まれません。

- native distributable release
- OS-level sandbox enforcement
- enforcement-complete classification / MAC lattice
- concrete signing / notarization values
- Pro / Pro+ / Enterprise / CITADEL feature surface
- private development / internal operational corpus
- organization-specific operational workflow

## 9. 次に読む文書

利用前後に読む順序としては、次を推奨します。

1. `CYRUNE_Free_Public_Index.md`
2. `GETTING_STARTED.md`
3. `FIRST_SUCCESS_EXPECTED.md`
4. `TROUBLESHOOTING.md`
5. `ENGINEERING_SPEC.md`

この文書は、利用者向けの使用説明書として参照してください。
