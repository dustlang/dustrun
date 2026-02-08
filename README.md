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

## What `dustrun` Does *Not* Do

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

```text
dustrun/
├── crates/
│   ├── dvm/        # DVM core: execution semantics, regimes, admissibility
│   ├── dustrun/    # CLI binary (produces the `dustrun` executable)
│   └── conformance/# Spec-aligned conformance test harness
├── tests/          # Deterministic fixtures and golden traces
├── docs/           # Non-normative architecture and usage notes
└── Cargo.toml      # Workspace root