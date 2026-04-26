# CYRUNE

## 0. Public Entry

For the current accepted public corpus of CYRUNE Free v0.1, start with `../CYRUNE_Free_Public_Index.md`.
This page is a product overview of CYRUNE as a whole. It is not a claim that this Free v0.1 public alpha implements every CYRUNE or CITADEL enforcement layer.

## 1. What is CYRUNE?

**CYRUNE** is a domain-agnostic operating system for controlled knowledge environments.

It is a structured control layer built on the CRANE Kernel.

CYRUNE is not a chatbot.
It is not a generative AI wrapper.
It is not a cloud service.

CYRUNE is a **knowledge control operating system** that defines structure, classification governance, citation-bound reasoning, and auditability over intelligent systems.

Where CITADEL represents the hardened defense distribution,
CYRUNE represents the general-purpose control OS for high-trust environments.

---

## 2. Core Philosophy

CYRUNE is designed around five principles:

1. **Controlled memory**
2. **Classification governance**
3. **Citation-bound reasoning**
4. **Fail-closed governance**
5. **Detachable intelligence**

CYRUNE treats AI as a replaceable component.
Control and enforceability are primary; generation is secondary.

---

## 3. Architectural Layers

```
CYRUNE
 ├── Policy Pack (Consumer / Medical / Finance / Custom)
 ├── Adapter Layer (LLM / Embedding / Index / Connector)
 ├── Runtime Layer (CLI canonical, Desktop optional)
 └── CRANE Kernel
       ├── Memory (Working / Processing / Permanent)
       ├── Query Engine
       ├── Gate (Enforcement)
       └── Ledger (Audit Log)
```

CYRUNE is the operational distribution of the CRANE Kernel.

---

## 4. Memory Model

CYRUNE uses the same three-layer model as CRANE:

### Working Memory

* Limited to 10±2 structured state elements
* Represents current operational state
* Classification-aware
* Explicitly updated (semi-automatic)

### Processing Memory

* Holds logs, drafts, retrieved materials
* Designed to carry classification metadata
* May be promoted

### Permanent Memory

* Approved and versioned documents
* Promotion requires authorization
* Immutable once approved

The CYRUNE design uses classification labels:

* PUBLIC
* INTERNAL
* RESTRICTED
* SECRET

In this Free v0.1 public alpha, classification / MAC is a product intent and claim boundary. This public source surface does not claim an enforcement-complete classification lattice or clearance governance.

---

## 5. Classification and Control

CYRUNE's target control model is Mandatory Access Control (MAC):

```
clearance >= classification_label
```

Output classification is determined by:

```
max(input, retrieved, working)
```

The purpose is to prevent classification leakage across domains.

Policy Packs define domain-specific extensions (e.g., PHI in medical, financial retention rules).

---

## 6. Citation-Bound Reasoning

CYRUNE does not trust generative output.

Instead:

* Retrieval produces a citation bundle
* All reasoning must remain within the bundle
* Non-cited claims are rejected

This ensures explainability and auditability.

---

## 7. Detachable Intelligence Layer

CYRUNE supports multiple modes:

### Mode 1: No-LLM

* Retrieval + extractive summarization
* Fully operational without generative models

### Mode 2: Local LLM

* LLM operates strictly on citation bundle
* Output constrained by Gate
* Model version recorded

### Mode 3: Approved Adapter

* LLM as replaceable module
* Policy-defined constraints
* Explicit activation required

CYRUNE does not assume the presence of an LLM.

---

## 8. Offline and Deployment Model

CYRUNE supports:

* On-premise deployment
* Closed VPN environments
* Air-gapped environments (when required)
* Controlled update cycles

Unlike SaaS systems, CYRUNE is designed for environments where external communication is restricted.

---

## 9. Update Model

* Major releases: 18-month cycle
* Patch releases: security fixes only
* No self-update mechanism
* Signed update packages
* Version recorded in Ledger

For this Free v0.1 public alpha, the signed update model is a product-wide design direction, not a shipped updater or signed update channel. This repository does not include signing / notarization workflow, concrete signing values, or signed update package delivery.

---

## 10. CLI as Canonical Interface

CLI is the primary interface.

All functionality must be accessible via CLI.

GUI (Desktop Native) is optional and provides:

* Visualization of citation bundles
* Working state inspection
* Ledger timeline view
* Policy visibility

GUI does not introduce additional authority beyond CLI.

---

## 11. Intended Domains

CYRUNE is designed for:

* Medical institutions
* Financial institutions
* Regulated enterprises
* Research facilities
* Government agencies
* Organizations requiring knowledge control

CITADEL is the hardened defense distribution of CYRUNE.

---

## 12. Relationship to CRANE and CITADEL

* **CRANE**: OSS Kernel (control contracts)
* **CYRUNE**: Commercial control operating system
* **CITADEL**: Hardened defense distribution of CYRUNE

CYRUNE generalizes control.
CITADEL maximizes enforcement.

---

## 13. Product Identity

CYRUNE is defined by:

* Structured control
* Classification integrity
* Citation enforcement
* Auditability
* Replaceable intelligence

It is not an AI assistant.
It is an operating system for controlled knowledge.
