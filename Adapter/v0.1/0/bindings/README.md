# Distro Adapter Bindings v0.1

このディレクトリは Distro Adapter Binding を保存する場所です。

## 想定ファイル名

- `cyrune-free-default.v0.1.json`
- `cyrune-free-shipping.v0.1.json`
- `forge-core-default.v0.1.json`

## 運用ルール

1. `../schemas/distro-adapter-binding.schema.json` に適合させる
2. Adapter ID は Distro 非依存命名を使う
3. 容量/TTL 等の意味論を含めない（Policy 側で定義）
