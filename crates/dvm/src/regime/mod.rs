// File: mod.rs - This file is part of the DPL Toolchain
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
//   Regime module root for DVM (Dust Virtual Machine).
//   Groups regime-specific semantic enforcement and host-mode execution.
//   Regimes: K (deterministic), Q (linear resources), Φ (validation)
//   Exports: q, phi, phi_witness modules and types

pub mod phi;
pub mod phi_witness;
pub mod q;

pub use q::{QBinding, QResId, QResMeta, QResState, QSnapshot, QState};

pub use phi::{
    refuse_execution as phi_refuse_execution, validate_proc as phi_validate_proc, PhiValidation,
};

pub use phi_witness::{
    refuse_global_witness as phi_refuse_global_witness, PhiWitness, PhiWitnessBuilder,
    PhiWitnessKind,
};
