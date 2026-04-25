# CITADEL Threat Model v0.1

# 0.Deployment Profiles

CITADEL と CYRUNE は異なる運用前提を持つ。

## CYRUNE

* External connectors may be enabled via Policy Pack.
* Network communication is policy-defined.
* Designed for high-trust but not strictly air-gapped environments.

## CITADEL

* No external communication by architecture.
* Connectors disabled unless explicitly provisioned in hardened profile.
* Designed for air-gapped or strictly controlled networks.

---

# 1. Purpose

This threat model defines:

* What CITADEL protects
* What it assumes
* What it does not attempt to protect
* How threats are structurally mitigated

CITADEL is not designed to be a general AI assistant.
It is designed to operate in high-classification, high-trust environments.

---

# 2. Security Objectives

CITADEL guarantees:

1. **No unauthorized data exfiltration**
2. **No citation-unbound reasoning**
3. **No unclassified data ingestion**
4. **No silent model mutation**
5. **Immutable auditability**
6. **Offline operational integrity**

---

# 3. Assets to Protect

## 3.1 Data Assets

* Classified documents
* Permanent memory records
* Working state context
* Policy Packs
* Model files
* Ledger logs

## 3.2 Control Assets

* Classification labels
* Gate enforcement logic
* Policy signature validation
* Model hash verification

---

# 4. Adversary Categories

## A1. External Network Attacker

Attempts:

* Data exfiltration
* Remote code execution
* Supply chain injection

Mitigation:

* No external communication
* Deny-by-default connectors
* Offline deployment
* Signed update packages

---

## A2. Malicious Insider (Authorized User)

Attempts:

* Access data above clearance
* Force unclassified ingestion
* Override policy
* Modify ledger

Mitigation:

* Mandatory Access Control (MAC)
* Classification enforcement
* Immutable Ledger (append-only)
* Signature validation on policy
* Promotion authorization workflow

---

## A3. Model-Level Attack

Attempts:

* Prompt injection
* Hallucination
* Citation bypass
* Model tampering

Mitigation:

* Citation-bound reasoning
* Gate rejection of non-cited claims
* Model hash verification
* Detached LLM architecture
* No dynamic model replacement

---

## A4. Supply Chain Attack

Attempts:

* Replace model files
* Replace binaries
* Replace policy packs

Mitigation:

* SHA256 verification
* Signed packages
* Hash validation at startup
* Ledger entry on version change
* No auto-update

---

## A5. Configuration Drift

Attempts:

* Silent policy modification
* Classification downgrade
* Connector enablement

Mitigation:

* Signed Policy Pack
* Deny-by-default configuration
* Versioned configuration tracking
* `citadel doctor` integrity check

---

# 5. Explicit Non-Goals

CITADEL does NOT attempt to:

* Protect against full OS compromise
* Prevent physical access attacks
* Replace enterprise identity systems
* Guarantee deterministic LLM output
* Replace human judgment

CITADEL enforces structural boundaries, not cognitive correctness.

---

# 6. Trust Assumptions

CITADEL assumes:

* Host OS integrity is maintained
* Hardware is trusted
* Administrator is authenticated
* Physical security exists

If these assumptions fail, CITADEL cannot guarantee protection.

---

# 7. Enforcement Model Summary

| Threat              | Structural Mitigation     |
| ------------------- | ------------------------- |
| Data exfiltration   | No external communication |
| Citation bypass     | Citation-bound Gate       |
| Policy tampering    | Signed Policy Pack        |
| Model tampering     | Hash + signed LLM Pack    |
| Ledger modification | Append-only log           |
| Classification leak | Mandatory Access Control  |

---

# 8. Defense-Grade Extensions (CITADEL Only)

* Air-gapped deployment
* WORM ledger storage
* Hardware-backed key storage (future)
* Secure enclave support (future)
* Per-installation encrypted LLM pack (optional)

---

# 9. Security Philosophy

CITADEL does not attempt to make AI smarter.

It attempts to make AI controllable.

Security is achieved through:

* Structural constraint
* Mandatory classification
* Fail-closed enforcement
* Immutable auditability
