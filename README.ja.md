# CYRUNE Free v0.1

CYRUNE Free v0.1 は、単一ユーザー向け CYRUNE Free runtime の **public beta** リポジトリです。

このリポジトリの一次導線は英語です。この文書は日本語の companion であり、英語版 README、Getting Started、Public Index の claim boundary を上書きしません。

## 公開モデル

- `main` は最新の公開面です。
- `v0.1.0` は CYRUNE Free v0.1 public alpha の immutable snapshot tag です。
- `v0.1.1-beta.1` は CYRUNE Free v0.1 public beta の release-contract tag です。
- `v0.1` は互換 marker / compatibility tag として扱い、branch 名としては使いません。
- `v0.1` branch は作成しません。

## はじめに読むもの

1. [Getting Started](docs/GETTING_STARTED.md)
2. [First Success Expected Result](docs/FIRST_SUCCESS_EXPECTED.md)
3. [Troubleshooting](docs/TROUBLESHOOTING.md)
4. [Public Beta Criteria](docs/BETA_CRITERIA.md)
5. [Public Index](docs/CYRUNE_Free_Public_Index.md)
6. [日本語 companion docs](docs/ja/)

## この beta が主張する範囲

この public beta は、`prepare-public-run.sh`、`doctor.sh`、`first-success.sh` による first-success path を、source、carrier、release asset、CI、docs、runtime evidence、Closed Gate Report を結合した release contract として公開します。`first-success.sh` の成功は raw `cyr run` の終了ではなく、`cyr verify first-success` の `outcome: "accepted"` report と terminal binding marker によって判定します。

この public beta は、native installer、署名済み desktop distribution、OS-level sandbox enforcement、enforcement-complete classification / MAC、より広い製品ラインの機能完成を主張しません。

## License

CYRUNE Free v0.1 の first-party source は MIT または Apache-2.0 のいずれかを選択して利用できます。詳細は [LICENSE](LICENSE)、[LICENSE-MIT](LICENSE-MIT)、[LICENSE-APACHE](LICENSE-APACHE) を参照してください。

再配布される model/tokenizer resource の third-party notice は [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md) にあります。この Free repository の license は、ここに含まれる Free v0.1 first-party source にだけ適用されます。
