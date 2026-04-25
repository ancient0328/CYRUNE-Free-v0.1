# CYRUNE Free v0.1: Implementation And Physical Map

**作成日時 (JST)**: 2026-04-12 10:14:17 JST
**分類**: `現行正典`
**時間相**: `現在との差分を比較する段階`

## 1. この巻の役割

この巻は code walkthrough ではなく、実装の物理配置と責務の対応を説明する。
「どこに何があり、どの層がどの責務を持つか」を理解するための巻である。

## 2. workspace の構成

Free v0.1 の実装 workspace は次の責務群を持つ。

1. contract crate
2. control-plane crate
3. daemon crate
4. runtime-cli crate
5. scripts
6. bundle-root resources
7. proof artifact roots

## 3. contract crate

contract crate は request / result / ID / denial の閉集合 contract を持つ。
ここでは syntax と data shape を固定し、turn の運用意味論は持たない。

主な責務:

1. request / result envelope
2. ID family
3. denial reason family
4. shared data types

## 4. control-plane crate

control-plane crate は製品価値の中心である。
主な責務は次である。

1. request validation
2. resolver integration
3. resolved turn context construction
4. memory façade
5. deterministic Working rebuild
6. retrieval selection
7. policy gate
8. citation validation
9. execution result normalization
10. evidence ledger write
11. final turn outcome decision

## 5. daemon crate

daemon crate は Control Plane の host である。
主な責務は次である。

1. NDJSON IPC
2. command dispatch
3. stdio server
4. read-only view bridge
5. packaged mode doctor / launch support
6. home layout materialization

## 6. runtime-cli crate

runtime-cli crate は user-facing runtime surface を持つ。
主な責務は次である。

1. `cyr` command family
2. view surface
3. doctor surface
4. packaged launch / pack logic
5. D6 proof driver
6. D7 proof driver

## 7. scripts の役割

script family は本番 user path の置換ではない。
主に次の用途で使う。

1. packaged staging
2. proof family 再現
3. smoke 実行
4. validation artifact 採取

current accepted scope では、script の存在自体ではなく、それが生成する accepted / fail-closed / validation artifact family が採用されている。

## 8. bundle-root resources

bundle-root resource family は packaged mode の immutable static payload である。
主な classes は次である。

1. adapter catalog
2. policy packs
3. bindings
4. approved execution adapter registry / profiles
5. terminal template
6. launcher scripts
7. shipping exact pin manifest / artifacts

## 9. CYRUNE_HOME の物理配置

`CYRUNE_HOME` は次の family を持つ。

1. `working/`
2. `ledger/`
3. `memory/processing/`
4. `memory/permanent/`
5. `runtime/`
6. `terminal/config/`
7. `embedding/`
8. `registry/`
9. `packs/`
10. `cache/`
11. `tmp/`

このうち authority として扱ってよいのは `ledger/evidence/*` の証跡本体だけである。
static resource authority は bundle-root に留まる。

## 10. accepted physical artifact roots

current accepted closeout に効く artifact root は大きく 4 系統ある。

1. core / baseline proof root
2. corrective proof root
3. D6 proof root
4. D7 proof root

proof root は「何が観測されたか」を再現するための場所であり、製品 runtime の source of truth ではない。

## 11. runtime surface の物理 family

### 11.1 accepted execution path

accepted execution path は少なくとも次の 2 経路を持つ。

1. No-LLM path
2. approved execution adapter path

### 11.2 view path

read-only view として次を持つ。

1. evidence
2. working
3. policy

### 11.3 packaged family

packaged family では、distribution root、bundle root、home template、release manifest、hash / notice / SBOM family を持つ。

## 12. D6 / D7 実装 family

### 12.1 D6

D6 line の実装 family は次である。

1. launcher-owned surface
2. `launcher -> cyr` handoff
3. accepted artifact family
4. fail-closed artifact family
5. workspace validation family

### 12.2 D7

D7 line の実装 family は次である。

1. productization identity conduit
2. notice / license / SBOM conduit
3. integrity / signature conduit
4. upstream intake judgment conduit
5. productization failure artifact family
6. D7 proof driver family
7. workspace validation family

## 13. どこを変えると何が崩れるか

### 13.1 authority まわり

bundle-root 解決、home layout、packaged launch を崩すと D5 / corrective / D6 / D7 のすべてに影響する。

### 13.2 turn flow まわり

control-plane の request validate、Working rebuild、Gate、Citation、Ledger を崩すと core 成立が崩れる。

### 13.3 launcher / productization まわり

D6 family を崩すと single-entry と split が崩れる。
D7 family を崩すと productization failure separation と packaged release evidence が崩れる。

## 14. code を読む時の優先順

code focus は主目的ではないが、実装を追うなら次の順が理解しやすい。

1. contract family
2. control-plane family
3. daemon family
4. runtime-cli family
5. packaged resource family
6. proof / smoke / validation family

## 15. current accepted physical conclusion

現在の物理配置は、次の条件を満たしている。

1. Control Plane の責務は control-plane crate に集中している
2. Runtime と daemon は projection / hosting に留まる
3. static authority は bundle-root に集中している
4. mutable state は home 配下に集中している
5. proof family は owner-local artifact root に分離されている
6. D6 / D7 の追加 family は core semantics を侵食していない
