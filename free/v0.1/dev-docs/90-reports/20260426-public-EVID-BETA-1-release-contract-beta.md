# 20260426-public-EVID-BETA-1-release-contract-beta

**Report type**: Closed Gate Report
**Subject**: CYRUNE Free v0.1 public beta release contract
**Public surface**: `Distro/CYRUNE/public/free-v0.1/` / GitHub `ancient0328/CYRUNE`
**Beta tag**: `v0.1.1-beta.1`
**Beta release commit**: `61eb4c68630600d9b1a7f325fd6d06759ede846c`
**Post-release closeout main commit**: `4c655ea5455c0cb0e76fb611b422292bf54fb3bb`
**Created at**: 2026-04-26T12:19:13+0900 JST
**Correlation ID**: `EVID-BETA-1`

## 1. Scope

This report covers the Release-Contract Beta implementation for the CYRUNE Free v0.1 public repository surface.

The beta contract binds:

- tracked public source,
- beta carrier archive,
- immutable beta tag / GitHub prerelease,
- public CI,
- public docs,
- local first-success runtime evidence,
- this Closed Gate Report.

Out of scope:

- moving or reinterpreting `v0.1.0`,
- native installer or signed desktop distribution,
- OS-level sandbox enforcement,
- enforcement-complete classification / MAC,
- Pro / Pro+ / Enterprise / CITADEL surface,
- production maturity.

## 2. Implementation Summary

- Added `docs/BETA_CRITERIA.md` and `docs/ja/BETA_CRITERIA.md`.
- Updated public entry docs from alpha wording to public beta release-contract wording.
- Kept `v0.1.0` as immutable public alpha snapshot.
- Introduced `v0.1.1-beta.1` as the public beta release-contract line.
- Updated `scripts/prepare-public-run.sh` to pin the beta carrier URL, filename, size, and SHA256.
- Added `scripts/check-beta-release-contract.sh`.
- Updated public CI to run beta release-contract checks before and after public-run preparation.
- Updated carrier/release helper scripts for `v0.1.1-beta.1`.
- Retired the stale alpha-era `export_public_corpus.py` path by making it fail-closed.

## 3. Release Evidence

- GitHub repository: `ancient0328/CYRUNE`
- `main` at beta release publication: `61eb4c68630600d9b1a7f325fd6d06759ede846c`
- `main` after post-release closeout report publication: `4c655ea5455c0cb0e76fb611b422292bf54fb3bb`
- `v0.1.1-beta.1` tag target: `61eb4c68630600d9b1a7f325fd6d06759ede846c`
- `v0.1.0` tag target remains unchanged:
  - tag object: `55d9622d0795a1626be5da28dfbc5c4a6ac47f98`
  - peeled commit: `e39b0326c746e1827c7829c3bf6bb804b0238b2a`
- GitHub release: `https://github.com/ancient0328/CYRUNE/releases/tag/v0.1.1-beta.1`
- Release mode: prerelease
- Release asset: `cyrune-free-v0.1.1-beta.1.tar.gz`
- Asset size: `563982199`
- Asset SHA256: `73654922f0f1c170ce34001d6f1021b72ec9eb8c28aa8a81a3d572ccde00c938`

## 4. Local Validation Evidence

Commands executed from `/tmp/cyrune-publish-9ViAx3I6/CYRUNE`:

```bash
./scripts/check-beta-release-contract.sh
python3 -m py_compile free/v0.1/0/scripts/*.py
git diff --check
for script in scripts/*.sh; do bash -n "$script"; done
cargo fmt --manifest-path free/v0.1/0/Cargo.toml --all -- --check
cargo check --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets
cargo clippy --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path free/v0.1/0/Cargo.toml --workspace --all-targets
./scripts/prepare-public-run.sh
./scripts/check-beta-release-contract.sh
./scripts/doctor.sh
./scripts/first-success.sh
```

Observed local first-success result:

- `correlation_id`: `RUN-20260426-6118`
- `run_id`: `RUN-20260426-6118-R01`
- `evidence_id`: `EVID-1`
- `policy_pack_id`: `cyrune-free-default`
- `citation_bundle_id`: `CB-20260426-6118`
- `working_hash_after`: `sha256:3eec2a461c2351ccf17a855986d45414f30575552bac2afa223b00aa15a4d33e`

Required local evidence files were present under:

```text
free/v0.1/0/target/public-run/home/ledger/evidence/EVID-1/
```

Observed files:

- `manifest.json`
- `run.json`
- `policy.json`
- `citation_bundle.json`
- `rr.json`
- `working_delta.json`
- `stdout.log`
- `stderr.log`
- `hashes.json`

Working projection was present at:

```text
free/v0.1/0/target/public-run/home/working/working.json
```

## 5. Remote CI Evidence

GitHub Actions run:

- workflow: `public-ci`
- run id: `24946963574`
- head SHA: `61eb4c68630600d9b1a7f325fd6d06759ede846c`
- conclusion: `success`
- URL: `https://github.com/ancient0328/CYRUNE/actions/runs/24946963574`

Post-release closeout GitHub Actions run:

- workflow: `public-ci`
- run id: `24947149680`
- head SHA: `4c655ea5455c0cb0e76fb611b422292bf54fb3bb`
- conclusion: `success`
- URL: `https://github.com/ancient0328/CYRUNE/actions/runs/24947149680`

Remote CI completed these beta gates:

- public shell scripts parse,
- Python helper syntax,
- beta release-contract predicates,
- Rust formatting,
- Rust workspace check,
- Rust lint with warnings denied,
- public first-success state preparation from the beta carrier,
- prepared beta carrier contract check,
- public first-success path,
- Rust tests.

## 6. Closed Gate Report

### Gate 1. 個別事案固定性

**判定**: Strong Yes
**理由**: 対象は `Distro/CYRUNE/public/free-v0.1/` / GitHub `ancient0328/CYRUNE` の CYRUNE Free v0.1 public beta release contract に固定されている。`v0.1.0` alpha snapshot、native distribution、OS sandbox、complete MAC、Pro / Enterprise / CITADEL は対象外として明示され、beta criteria と public docs に反映された。
**直接根拠**: `docs/BETA_CRITERIA.md`, `README.md`, `docs/CYRUNE_Free_Public_Index.md`, GitHub release `v0.1.1-beta.1`
**崩れる条件**: `v0.1.0` を beta と再解釈する、または native / Enterprise / CITADEL scope を beta 成立条件へ混入した場合。

### Gate 2. fail-closed

**判定**: Strong Yes
**理由**: `scripts/check-beta-release-contract.sh` は beta tag / asset / docs / helper predicates を fail-closed に検査し、`prepare-public-run.sh` は beta carrier の filename / size / SHA256 / tar safety / manifest を検証する。CI はその検査を public-run preparation の前後で実行する。
**直接根拠**: `scripts/check-beta-release-contract.sh`, `scripts/prepare-public-run.sh`, `.github/workflows/public-ci.yml`, CI run `24946963574`
**崩れる条件**: asset hash mismatch、missing release manifest、docs claim overreach、または CI gate skip を成功扱いした場合。

### Gate 3. 根拠の接続と範囲

**判定**: Strong Yes
**理由**: docs claim は beta criteria、runtime evidence は local first-success、release identity は GitHub prerelease/tag/asset、quality gate は local command と remote CI に分離され、first-success 単独を production maturity や native distribution の根拠にしていない。
**直接根拠**: `docs/BETA_CRITERIA.md`, `docs/FIRST_SUCCESS_EXPECTED.md`, release asset SHA256, local `EVID-1`, CI run `24946963574`
**崩れる条件**: first-success success を production maturity、native installer、OS-level sandbox、complete MAC の根拠として扱った場合。

### Gate 4. 構造・責務・意味論整合

**判定**: Strong Yes
**理由**: source、carrier、release asset、CI、docs、evidence report の責務が分離され、`main` は latest public surface と post-release closeout surface、`v0.1.0` は alpha snapshot、`v0.1.1-beta.1` は immutable beta release-contract tag として時間相と責務が分かれている。stale alpha exporter は fail-closed 化された。
**直接根拠**: `README.md`, `docs/CYRUNE_Free_Public_Index.md`, `free/v0.1/0/scripts/export_public_corpus.py`, `free/v0.1/0/scripts/publish_release_package_to_github.py`
**崩れる条件**: stale alpha generator が public beta surface を上書きできる状態に戻る、または beta carrier pin が alpha carrierへ戻る場合。

### Gate 5. 時間軸整合

**判定**: Strong Yes
**理由**: `v0.1.0` は旧 alpha snapshot のまま保持され、`v0.1.1-beta.1` を新規 beta line として作成した。過去の alpha evidence は beta evidence に流用せず、新しい asset、CI、first-success evidence を作成した。
**直接根拠**: `git ls-remote` result for `v0.1.0` and `v0.1.1-beta.1`, GitHub release `v0.1.1-beta.1`, local `EVID-1`, CI run `24946963574`
**崩れる条件**: alpha tag を移動する、alpha release asset を beta asset として再利用する、または alpha closeout report を beta closeout として扱う場合。

### Gate 6. 未証明採用の不在

**判定**: Strong Yes
**理由**: beta claim は remote prerelease、asset hash、public CI、local first-success、docs consistency、Closed Gate Report に限定されている。production maturity、native installer、OS-level sandbox enforcement、enforcement-complete classification / MAC、Pro / Enterprise / CITADEL は non-claim のまま残され、成立根拠に採用していない。
**直接根拠**: `README.md`, `docs/BETA_CRITERIA.md`, `docs/GETTING_STARTED.md`, `docs/USER_GUIDE.md`, local and remote validation listed above
**崩れる条件**: 未実装または未検証の product maturity / native distribution / OS sandbox / complete MAC / upper-tier feature を beta 成立根拠へ追加した場合。

## 7. Result

The `v0.1.1-beta.1` release-contract evidence satisfies the public beta criteria for the CYRUNE Free v0.1 public repository surface.

This result does not claim production maturity, native distribution, OS-level sandbox enforcement, enforcement-complete classification / MAC, Pro / Enterprise / CITADEL surface, or private/internal corpus publication.
