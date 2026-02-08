# dustrun

**The Dust Virtual Machine (DVM) Reference Executor**

---

## Overview

`dustrun` is the **reference execution environment** for the **Dust Programming Language (DPL)**.  
It implements the **Dust Virtual Machine (DVM)** in Rust and serves as the **normative semantic authority** for executing DPL programs.

While DPL programs are primarily compiled to **native binaries** for production use, `dustrun` is required for:

- executing DPL programs without specialized hardware,
- validating admissibility, effects, and time semantics,
- testing and debugging across classical, quantum, and phase regimes,
- enforcing conformance with the DPL specification,
- providing an executable realization of the language semantics.

`dustrun` is not a compiler.  
It is the **reference executor** against which all other execution targets must be validated.

---

## Role in the DPL Ecosystem

DPL separates **semantic authority** from **performance realization**.

- **Specification (`spec/` in the DPL repo)**  
  Defines the meaning of the language.

- **Compiler (`dustc`)**  
  Produces native binaries or intermediate representations.

- **`dustrun` (this repository)**  
  Implements the Dust Virtual Machine (DVM) and defines *what execution means*.

Native binaries do **not** require `dustrun` at runtime.  
They are correct **only if** they are observationally and admissibly equivalent to execution via `dustrun`.

---

## What `dustrun` Does

`dustrun` executes **Dust Intermediate Representation (DIR)** artifacts according to the DPL specification.

It provides:

- **K-regime execution**  
  Deterministic classical execution with explicit time and effect semantics.

- **Q-regime enforcement**  
  Linear resource tracking, allocation/deallocation, and measurement boundaries, without requiring quantum hardware.

- **Φ-regime resolution**  
  Global admissibility checking, constraint satisfaction, and witness handling.  
  Φ-regime computation resolves *existence*, not step-by-step execution.

- **Simulation vs realization modes**  
  Effects may be logged (simulation) or enacted (realization).

- **Deterministic replay**  
  Identical inputs yield identical outcomes.

If a program is **inadmissible**, `dustrun` does not partially execute it.  
**Non-existence is a valid and enforced outcome.**

---

## What `dustrun` Does Not Do

`dustrun` does **not**:

- parse DPL source code,
- compile or optimize programs,
- perform code generation,
- infer semantics heuristically,
- simulate quantum physics beyond semantic requirements,
- redefine or extend the DPL language.

Those responsibilities belong elsewhere in the DPL toolchain.

---

## Repository Structure

dustrun/
├── crates/
│   ├── dvm/         # DVM core: execution semantics, regimes, admissibility
│   ├── dustrun/     # CLI binary (produces the `dustrun` executable)
│   └── conformance/ # Spec-aligned conformance test harness
├── tests/           # Deterministic fixtures and golden traces
├── docs/            # Non-normative architecture and usage notes
└── Cargo.toml       # Workspace root

- **crates/dvm** is the authoritative implementation of DVM semantics.
- **crates/dustrun** is a thin command-line interface over the DVM.
- **crates/conformance** ensures alignment with the DPL specification.

---

## Typical Usage

Reference execution:

dustrun program.dir

Simulation mode (effects are logged, not realized):

dustrun --simulate program.dir

Deterministic replay:

dustrun --replay trace.json

---

## Conformance Rule

Any execution backend, including native binaries produced by `dustc`, is considered **correct** only if:

- it admits exactly the same executions,
- it rejects exactly the same inadmissible programs,
- it preserves effect ordering and irreversibility,
- it respects time and regime semantics,

as execution via `dustrun`.

If there is disagreement, **dustrun is correct**.

---

## Implementation Language

`dustrun` is implemented in **Rust** to provide:

- deterministic behavior,
- strong correctness guarantees,
- portability,
- integration with existing systems tooling.

A future self-hosting implementation in DPL is possible, but only after full semantic parity is provably achieved.

---

## Status

- DVM semantics: **In active development**
- CLI stability: **Pre-v1**
- Performance: **Not a goal**
- Semantic correctness: **Primary goal**

`dustrun` prioritizes correctness, determinism, and semantic fidelity over speed.

---

## License

`dustrun` is released under the **Dust Open Source License (DOSL)**.

See the LICENSE file for full terms.

---

## Relationship to the DPL Specification

The DPL specification (located in the spec/ directory of the main DPL repository) is **canonical**.

This repository:
- implements the specification,
- does not redefine it,
- and must be updated if the specification changes.

If behavior here conflicts with the spec, **this code is wrong**.

---

## Final Note

Without a Dust Virtual Machine, DPL would be untestable, unverifiable, and unusable for most developers today.

`dustrun` exists to ensure that DPL programs can be written, tested, reasoned about, and trusted—long before specialized hardware becomes commonplace.

---

© 2026 Dust LLC