# Adapter Catalog v0.1

このディレクトリは Adapter Capability Manifest の配置場所です。

## 目的

- Distro から参照される Adapter 能力宣言を単一の場所に集約する
- CI で schema 検証できる形で管理する

## 想定ファイル名

- `memory-kv-inmem.v0.1.json`
- `memory-redb-processing.v0.1.json`
- `memory-stoolap-permanent.v0.1.json`
- `memory-kv-rocksdb.v0.1.json`
- `vector-index-basic.v0.1.json`

命名ルール:

- Adapter ID は Distro 非依存（`cyrune-*` / `forge-*` を禁止）
- 1つの manifest 内で `working/processing/permanent` を明示する

## 運用ルール

1. 追加時は `../schemas/adapter-capability.schema.json` に適合させる
2. 破壊的変更は `dev-docs/02-decisions/` に ADR を追加する
3. Distro 固有意味論は manifest に含めない
4. Distro と Adapter の紐付けは Binding で管理する
