# CYRUNE Free v0.1 Structure Pack

**状態（当時）**: Historical explanatory structure pack
**現在の権威状態**: Historical / non-authoritative
**取り扱い**: 2026-04-12 JST の `PB-C / PBC-I1 authority-state segregation` 後、この文書は current accepted source ではない。Free v0.1 の初期構造整理として参照に限定する。現行 authority は `docs/current/CYRUNE-Free_Canonical.md`、`free/v0.1/dev-docs/03-architecture/ARCHITECTURE_OVERVIEW.md`、`free/v0.1/dev-docs/summary/02-ARCHITECTURE_AND_RUNTIME_LINES.md` である。

## 1. Logical Architecture

```mermaid
flowchart TD
  U["User"] --> T["CYRUNE Terminal / CLI"]
  T --> C["cyr (single entry command)"]
  C --> D["cyrune-daemon"]

  D --> CP["Control Plane"]
  CP --> W["Working rebuild (10±2)"]
  CP --> P["Policy + Capability check (deny-by-default)"]
  CP --> X["LLM/No-LLM execution via adapter"]
  CP --> G["Citation validation + Fail-closed Gate"]
  CP --> L["Evidence Ledger atomic commit"]

  CP --> A["Adapter Layer"]
  A --> A1["LLM Adapter (detachable)"]
  A --> A2["Memory Adapter (W/P/P contract)"]
  A --> A3["Connector Adapter"]
```

## 2. Process Boundary

```mermaid
flowchart LR
  subgraph UI["UI Process (Unprivileged)"]
    T["CYRUNE Terminal"]
    C["cyr CLI"]
  end

  subgraph CORE["Control Process (Gated)"]
    D["cyrune-daemon"]
    CP["Control Plane"]
    GW["Gate / Policy / Capability"]
    LG["Ledger Writer (atomic)"]
  end

  subgraph EXT["External Resources"]
    FS["Filesystem"]
    NET["Network"]
    ST["Store / Index / Vault"]
  end

  T --> C
  C --> D
  D --> CP
  CP --> GW
  CP --> LG
  CP --> FS
  CP --> NET
  CP --> ST
```

## 3. 1-Turn Execution Flow

```mermaid
sequenceDiagram
  participant User
  participant Terminal as CYRUNE Terminal
  participant CLI as cyr
  participant Daemon as cyrune-daemon
  participant CP as Control Plane
  participant Adapter as LLM/No-LLM Adapter
  participant Ledger as Evidence Ledger

  User->>Terminal: Input
  Terminal->>CLI: Dispatch
  CLI->>Daemon: Run request
  Daemon->>CP: Start turn
  CP->>CP: Context clear
  CP->>CP: Working rebuild (max 12 slots)
  CP->>CP: Classification + policy pre-check
  CP->>Adapter: Execute
  Adapter-->>CP: Result + citations
  CP->>CP: Citation validate + fail-closed
  CP->>Ledger: Atomic commit (tmp -> fsync -> rename)
  Ledger-->>CP: Evidence ID
  CP-->>Daemon: Accepted output
  Daemon-->>CLI: Output + evidence ref
  CLI-->>Terminal: Render
  Terminal-->>User: Response
```

## 4. Runtime Data Layout (`~/.cyrune`)

```text
~/.cyrune/
├─ terminal/
│  └─ config/
│     └─ wezterm.lua
├─ ledger/
│  ├─ manifests/
│  │  └─ index.jsonl
│  └─ evidence/
│     └─ EVID-<id>/
│        ├─ manifest.json
│        ├─ run.json
│        ├─ policy.json
│        ├─ stdout.log
│        ├─ stderr.log
│        └─ hashes.json
├─ working/
│  └─ working.json
├─ packs/
│  └─ policy/
│     └─ default/
└─ cache/
```

## 5. Repository Layout (v0.1 Minimal)

```text
cyrune/
├─ crates/
│  ├─ cyr/
│  ├─ cyrune-daemon/
│  ├─ cyrune-core/
│  ├─ cyrune-control-plane/
│  ├─ cyrune-policy/
│  ├─ cyrune-ledger/
│  ├─ cyrune-adapter/
│  ├─ cyrune-view/
│  └─ cyrune-pack/
├─ apps/
│  └─ terminal/
├─ packs/
│  └─ policy/
│     └─ default/
├─ docs/
│  ├─ canonical/
│  └─ adr/
└─ xtask/
```
