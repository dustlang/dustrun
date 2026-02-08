// Regime module root for the Dust Virtual Machine (DVM).
//
// This module groups regime-specific semantic enforcement and host-mode execution
// components. Regime semantics are enforced by the DVM regardless of whether
// execution is native, VM, or delegated to specialized backends.

pub mod phi;
pub mod phi_witness;
pub mod q;

pub use q::{QBinding, QResId, QResMeta, QResState, QSnapshot, QState};

pub use phi::{refuse_execution as phi_refuse_execution, validate_proc as phi_validate_proc, PhiValidation};

pub use phi_witness::{refuse_global_witness as phi_refuse_global_witness, PhiWitness, PhiWitnessBuilder, PhiWitnessKind};
