# CYRUNE Free v0.1 Public Beta Criteria

**Status**: current public beta release-contract criteria の日本語 companion
**Subject**: CYRUNE Free v0.1 public repository surface の `main` と `v0.1.1-beta.1`

この文書は `docs/BETA_CRITERIA.md` の日本語 companion です。英語版の claim boundary を上書きしません。

## 1. beta の意味

CYRUNE Free v0.1 public beta は、単なる `alpha` から `beta` への文言変更ではありません。

public beta は、次が同じ候補に対して結合されている状態です。

- tracked public source
- verified carrier archive
- immutable beta tag / release asset
- public CI
- public documentation
- first-success runtime evidence
- Closed Gate Report

## 2. beta release line

- `main` は latest public repository surface です。
- `v0.1.0` は immutable public alpha snapshot のまま保持します。
- `v0.1.1-beta.1` を最初の public beta release-contract line とします。
- beta tag は公開後に移動しません。差し替えが必要な場合は `v0.1.1-beta.2` のような新しい tag を使用します。
- Closed Gate Report は `main` 上の post-release closeout evidence です。release と CI の後に成立する証跡であるため、immutable release tag snapshot 内に含まれる必要はありません。

## 3. 必須証跡

public beta claim には、同一 beta release line に対して次が必要です。

1. beta candidate の source commit SHA
2. その commit を指す `v0.1.1-beta.1` tag
3. beta tag に対応する GitHub release
4. `cyrune-free-v0.1.1-beta.1.tar.gz` release asset
5. asset size と SHA256
6. beta asset と一致する carrier `RELEASE_MANIFEST.json`
7. beta candidate の CI success
8. fresh `prepare-public-run.sh` -> `doctor.sh` -> `first-success.sh`
9. returned `evidence_id` と expected accepted-run evidence files
10. public docs consistency scan
11. license / third-party notice boundary check
12. `free/v0.1/dev-docs/90-reports/` 配下の Closed Gate Report

いずれかが欠ける場合、beta claim は成立しません。

## 4. 非主張範囲

この public beta は、Free v0.1 no-LLM first-success path の repeatable public repository release surface を主張します。

次は主張しません。

- production maturity
- native distributable release
- installer packaging
- signed desktop distribution
- signed update channel
- concrete signing / notarization values
- OS-level sandbox process isolation
- enforcement-complete classification / MAC lattice
- Pro / Pro+ / Enterprise / CITADEL feature surface
- private development / internal operational corpus
