# CITADEL

## 1. What is CITADEL?

**CITADEL** is a hardened, on-premise AI control appliance designed for high-classification and mission-critical environments.

It is not a chatbot.
It is not a cloud AI wrapper.
It is not a generative assistant.

CITADEL is a **controlled knowledge enforcement system**.

Its primary purpose is:

* To enforce strict information classification
* To bind all reasoning to verifiable citations
* To prevent uncontrolled output
* To operate fully offline
* To provide tamper-evident audit trails

CITADEL is the defense-grade distribution of the CYRUNE operating system, built on the CRANE Kernel.

---

## 2. Core Philosophy

CITADEL is built on five non-negotiable principles:

1. **Fail-closed by design**
2. **Mandatory classification**
3. **Citation-bound reasoning**
4. **No external communication**
5. **Immutable audit trail**

CITADEL does not prioritize model intelligence.
It prioritizes control, reproducibility, and enforceability.

---

## 3. Architectural Layers

```
CITADEL
 ├── Policy Pack (Defense Profile)
 ├── Adapter Layer (LLM / Embedding / Index)
 ├── Runtime Layer (CLI / Daemon)
 └── CRANE Kernel
       ├── Memory (Working / Processing / Permanent)
       ├── Query Engine
       ├── Gate (Enforcement)
       └── Ledger (Audit Log)
```

CITADEL is an appliance built on top of CRANE.

---

## 4. Memory Model

CITADEL uses a strict three-layer memory architecture:

### Working Memory

* Limited to 10±2 structured state elements
* Represents current operational context
* Always classification-bound
* Never unbounded conversational history

### Processing Memory

* Holds logs, drafts, intermediate artifacts
* Must be classified before use

### Permanent Memory

* Approved, signed, immutable documents
* Promotion requires explicit authorization

All memory objects require classification labels:

* PUBLIC
* INTERNAL
* RESTRICTED
* SECRET

Unclassified data is rejected at ingestion.

---

## 5. Enforcement (Gate)

CITADEL enforces:

### Access Control

A user may only access data with:

```
clearance >= classification_label
```

### Output Constraint

Every output is bound to a citation bundle.
If a statement cannot be traced to citations, it is rejected.

### Deny-by-default Connectors

No external connectors are enabled unless explicitly allowed in a signed policy pack.

---

## 6. Citation-Bound Reasoning

CITADEL does not trust model output.

Instead:

* Retrieval produces a **citation bundle**
* Any generated output must be derived strictly from this bundle
* Assertions not grounded in citations are rejected

The source of truth is the citation bundle, not the LLM.

---

## 7. Offline Operation

CITADEL:

* Does not perform external network communication
* Does not auto-update
* Does not call cloud APIs
* Does not embed telemetry

It is designed for:

* Air-gapped networks
* Closed VPN environments
* Mission-critical facilities

---

## 8. Update Model

CITADEL follows an ANSYS-style update discipline:

* Major release every 18 months
* No self-update mechanism
* Updates delivered via signed physical media or offline package
* SHA256 verification required
* Update events recorded in Ledger

---

## 9. LLM Layer (Detachable)

CITADEL does not require an LLM.

It supports three modes:

### Mode 1: No-LLM

* Retrieval + citation bundle
* Extractive summaries only
* Fully functional without any generative model

### Mode 2: Local LLM

* LLM operates strictly on citation bundle
* Output constrained by Gate
* Model hash fixed and verified at startup

### Mode 3: Approved Replacement Pack

* LLM distributed as signed detachable module
* Must be verified and approved before activation
* Version and hash recorded in Ledger

Model changes are treated as security events.

---

## 10. Audit (Ledger)

Every operation generates a ledger entry containing:

* Query
* Citation bundle
* Working state hash
* Classification level
* Policy pack ID
* Model ID + model hash
* Timestamp

Ledger is append-only.
Historical outputs are never rewritten.

Re-evaluation creates new entries instead of modifying old ones.

---

## 11. What CITADEL Is Not

CITADEL is not:

* A cloud AI assistant
* A consumer chatbot
* A SaaS product
* A black-box inference engine
* A continuously evolving auto-updating system

---

## 12. Intended Domains

CITADEL is designed for:

* Defense
* Military-adjacent systems
* National infrastructure
* High-security finance
* Classified research environments

It is the hardened distribution of CYRUNE.

---

## 13. Relationship to CYRUNE and CRANE

* **CRANE**: OSS Kernel (control contracts)
* **CYRUNE**: Domain-agnostic control OS
* **CITADEL**: Defense-grade hardened distribution

CITADEL is CYRUNE under maximum enforcement conditions.

---

## 14. Product Identity

CITADEL is not defined by AI capability.

It is defined by:

* Control
* Enforceability
* Classification integrity
* Offline reliability
* Auditability

It is a fortified knowledge control system.