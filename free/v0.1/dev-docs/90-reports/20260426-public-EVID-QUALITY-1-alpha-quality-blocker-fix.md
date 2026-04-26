# 20260426 Public EVID-QUALITY-1 Alpha Quality Blocker Fix

## 1. 判定メタ情報

- 判定日時: 2026-04-26T10:39:27+0900 JST
- Correlation ID: CYRUNE-PUBLIC-QUALITY-20260426-103027-JST
- 対象: `Distro/CYRUNE/public/free-v0.1/` / GitHub `ancient0328/CYRUNE` `main` public surface
- 対象フェーズ / タスク: Public Free v0.1 alpha quality blocker fix
- 時間相固定: current public repository surface を、公開 alpha としての現在到達品質へ補正する段階
- 判定スコープ: public repository envelope, public CI, Rust workspace validation, first-success path, license / third-party notice boundary
- build / test / runtime verification: 実施済み

## 2. 今回の目的

今回成立させる対象は、`public/free-v0.1/` が GitHub public repository surface として、public alpha claim に対して最低限必要な CI / test / first-success / notice / license boundary を持つことです。

今回成立させない対象は、native installer、signed desktop distribution、OS-level sandbox enforcement、enforcement-complete classification / MAC、Pro / Pro+ / Enterprise / CITADEL product surface、`v0.1.0` tag の再作成または移動です。

## 3. Initial Findings

1. Public CI に `cargo test --workspace --all-targets` が入っていなかった。
2. Shipping memory / retrieval / daemon tests が、public source tree では carrier-only である `model.onnx` を直接 source resource として期待し得る構造だった。
3. `adapter-resolver` crate に `license` metadata が欠落していた。
4. Root-level third-party notice と embedding artifact adjacent notice がなく、`intfloat/multilingual-e5-small` の source / carrier 境界が public repo 上で閉じていなかった。
5. Public Index に Free repository license grant の authority / non-authority boundary がなかった。

## 4. 補正

1. Public CI に public-run preparation、doctor / first-success、Rust test execution を追加した。
2. Shipping tests は `CYRUNE_TEST_SHIPPING_HOME_ROOT`、prepared `target/public-run/home`、complete `resources/bundle-root` の順に complete embedding artifact set を解決するようにした。Runtime exact-pin validation は弱めていない。
3. `adapter-resolver` に `MIT OR Apache-2.0` license metadata を追加した。
4. Root `THIRD-PARTY-NOTICES.md` と `multilingual-e5-small/NOTICE.md` を追加した。
5. `stage_shipping_readiness.py` の packaged notice bundle に embedding artifact boundary を追加した。
6. Public Index / Japanese companion Public Index に first-party license authority と Pro / Pro+ / Enterprise / CITADEL non-authority license boundary を追加した。
7. Third-party notice は upstream model metadata 由来であることを明示し、別個の upstream `LICENSE` file を vendored notice として推定しない境界に補正した。

補正後、Initial Findings 1-5 は本 report の scope 内では残存しません。

## 5. Verification Executed

All commands were executed from `/tmp/cyrune-publish-9ViAx3I6/CYRUNE`.

| Command | Result |
| --- | --- |
| `cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check` | Pass |
| `bash -n scripts/prepare-public-run.sh` | Pass |
| `bash -n scripts/doctor.sh` | Pass |
| `bash -n scripts/first-success.sh` | Pass |
| `cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets` | Pass |
| `python3 -m py_compile free/v0.1/0/scripts/export_public_corpus.py free/v0.1/0/scripts/publish_release_package_to_github.py free/v0.1/0/scripts/stage_shipping_readiness.py` | Pass |
| `cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings` | Pass |
| `./scripts/prepare-public-run.sh` | Pass |
| `./scripts/doctor.sh` | Pass, `status=healthy`, `registry_ready=true` |
| `./scripts/first-success.sh` | Pass, accepted `EVID-1`, `policy_pack_id=cyrune-free-default` |
| `cargo test --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets` | Pass |
| `cargo build --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets` | Pass |
| `cargo metadata --manifest-path free/v0.1/0/Cargo.toml --no-deps --format-version 1` license check | Pass, all workspace packages have license metadata |
| Markdown relative-link check | Pass |
| `git diff --check` | Pass |

The first direct `cargo test` attempt was rejected as non-acceptance evidence because the public-run carrier materialization state was incomplete at that instant. The accepted verification is the rerun after `./scripts/prepare-public-run.sh` completed with the full pinned carrier size and materialized `model.onnx` under `target/public-run/home/embedding/`.

## 6. 未完了だが正常なもの

- Native installer / signed desktop distribution
  - 正常理由: README / Public Index の alpha non-claim boundary で明示的に対象外。
  - Owner: future packaging / release-hardening owner.
  - 本文採用: 今回の alpha quality 成立根拠に採用していない。
- OS-level sandbox enforcement
  - 正常理由: README / Public Index は sandbox specification normalization / validation に claim を限定している。
  - Owner: future sandbox enforcement owner.
  - 本文採用: 今回の alpha quality 成立根拠に採用していない。
- Enforcement-complete classification / MAC
  - 正常理由: README / Public Index は product intent / claim boundary として扱い、現 alpha claim へ混入していない。
  - Owner: future classification / MAC owner.
  - 本文採用: 今回の alpha quality 成立根拠に採用していない。
- Pro / Pro+ / Enterprise / CITADEL product surfaces
  - 正常理由: Free repository license grant と public alpha claim の non-authority に明記。
  - Owner: future paid-tier / hardened-tier owners.
  - 本文採用: 今回の alpha quality 成立根拠に採用していない。
- Full legal audit of third-party model/tokenizer redistribution
  - 正常理由: 今回は public alpha repository notice boundary の補正であり、legal opinion / full redistribution audit を成立根拠にしていない。
  - Owner: future legal / release-hardening owner.
  - 本文採用: 今回の alpha quality 成立根拠に採用していない。

## 7. Closed Gate Report

### Gate 1: 個別事案固定性

- 判定: Strong Yes
- 判定理由: 対象を `public/free-v0.1/` public repository surface の alpha quality blocker fix に固定し、CI、test、first-success、license metadata、third-party notice、Public Index boundary の確認済み差分だけで判断している。
- 直接根拠: `.github/workflows/public-ci.yml`, `README.md`, `README.ja.md`, `docs/CYRUNE_Free_Public_Index.md`, `docs/ja/CYRUNE_Free_Public_Index.md`, `free/v0.1/0/Adapter/v0.1/0/Cargo.toml`, `free/v0.1/0/crates/cyrune-control-plane/src/memory.rs`, `free/v0.1/0/crates/cyrune-control-plane/src/retrieval.rs`, `free/v0.1/0/crates/cyrune-daemon/src/command.rs`, `THIRD-PARTY-NOTICES.md`, `free/v0.1/0/resources/bundle-root/embedding/artifacts/multilingual-e5-small/NOTICE.md`
- この判定が崩れる条件: 判断対象に `Distro/CYRUNE/free/v0.1/0` private canonical scope、Pro / Enterprise / CITADEL maturity、native packaging completion を混入した場合。

### Gate 2: fail-closed

- 判定: Strong Yes
- 判定理由: Public CI は shell parse、format、check、clippy、public-run preparation、doctor、first-success、cargo test を実行する。Shipping tests は complete artifact set が見つからない場合に panic で失敗し、silent fallback しない。
- 直接根拠: `.github/workflows/public-ci.yml`, `free/v0.1/0/crates/cyrune-control-plane/src/memory.rs`, `free/v0.1/0/crates/cyrune-control-plane/src/retrieval.rs`, `free/v0.1/0/crates/cyrune-daemon/src/command.rs`, `./scripts/prepare-public-run.sh`, `./scripts/doctor.sh`, `./scripts/first-success.sh`, `cargo test --workspace --all-targets`
- この判定が崩れる条件: CI から cargo test / first-success が外れる、artifact absence が skip / placeholder success へ変わる、または `prepare-public-run.sh` が carrier verification を省略する場合。

### Gate 3: 根拠の接続と範囲

- 判定: Strong Yes
- 判定理由: License claim は first-party `MIT OR Apache-2.0` と third-party notice boundary に限定し、embedding upstream metadata は notice と exact-pin manifest の範囲に限定している。Runtime behavior claim は executed commands の結果だけに限定している。
- 直接根拠: `LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD-PARTY-NOTICES.md`, `free/v0.1/0/resources/bundle-root/embedding/exact-pins/cyrune-free-shipping.v0.1.json`, `free/v0.1/0/resources/bundle-root/embedding/artifacts/multilingual-e5-small/NOTICE.md`, Hugging Face `intfloat/multilingual-e5-small` model metadata, `cargo metadata` license check, verification table
- この判定が崩れる条件: Third-party model/tokenizer が CYRUNE first-party license に含まれるように読める文言を追加する、upstream notice absence を CYRUNE が推定補完する、または unverified Pro / Enterprise / CITADEL rights を license grant に含める場合。

### Gate 4: 構造・責務・意味論整合

- 判定: Strong Yes
- 判定理由: Runtime exact-pin validation は維持し、tests は complete artifact source を明示探索するだけにしたため、source tree が carrier-only `model.onnx` を tracked public source として要求する不整合を解消している。Public docs は alpha claim と non-claim boundary を同じ主語へ揃えている。
- 直接根拠: `free/v0.1/0/crates/cyrune-control-plane/src/memory.rs`, `free/v0.1/0/crates/cyrune-control-plane/src/retrieval.rs`, `free/v0.1/0/crates/cyrune-daemon/src/command.rs`, `README.md`, `docs/CYRUNE_Free_Public_Index.md`
- この判定が崩れる条件: Production exact-pin verification を test convenience のために緩和する、または docs が native installer / enforcement-complete MAC / OS sandbox を current alpha claim として扱う場合。

### Gate 5: 時間軸整合

- 判定: Strong Yes
- 判定理由: `main` は latest public surface、`v0.1.0` は immutable snapshot として維持し、今回の変更は `main` の alpha quality correction に限定している。Past tag を移動せず、future tier / packaging scope は non-claim として分離している。
- 直接根拠: `README.md`, `docs/CYRUNE_Free_Public_Index.md`, Git policy in current task scope, verification commands executed on current `main` worktree
- この判定が崩れる条件: `v0.1.0` tag をこの修正に合わせて移動する、または future release-hardening items を current alpha completion の根拠に採用する場合。

### Gate 6: 未証明採用の不在

- 判定: Strong Yes
- 判定理由: 実装成立は local command results によって確認し、未実装領域は non-claim / 未完了だが正常として明示した。`cargo test` は初回不完全 state を acceptance evidence とせず、carrier materialization 完了後の再実行結果だけを採用した。
- 直接根拠: Verification table, Initial Findings and correction list, README alpha claim boundary, Public Index non-authority list
- この判定が崩れる条件: 未実行の CI success、GitHub remote state、native distribution readiness、OS sandbox enforcement、classification / MAC enforcement を今回の成立根拠として採用する場合。

## 8. 総括

- `No`: なし
- `Provisional Yes`: なし
- 最終結論: この修正差分は、`public/free-v0.1/` の public alpha quality blocker fix として成立する。
- 次 task: source mirror 同期、commit、push、GitHub Actions `public-ci` 成功確認、`v0.1.0` tag 不変確認。
