// File: phi.rs - This file is part of the DPL Toolchain
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
//   Φ-regime host-mode semantics (v0.1).
//   Provides validation-only semantics for Φ-regime programs.
//   Responsibilities:
//     - Recognize Φ-regime as distinct
//     - Deterministic refusal for execution
//     - Hooks for admissibility witness generation
//   Does NOT execute Φ-regime computation yet.

// In v0.1, witnesses are stub artifacts with deterministic structure.
// They are produced for `Prove` and `phi_witness(...)` but do not constitute
// global proofs.
use crate::DvmError;
use crate::{DirProc, Value};
use indexmap::IndexMap;

/// Φ-regime validation result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhiValidation {
    /// The procedure body is locally admissible under host-mode checks.
    LocallyAdmissible,

    /// The procedure body is locally inadmissible.
    LocallyInadmissible { message: String },
}

/// Validate a Φ-regime procedure body under host-mode checks.
///
/// Current v0.1 rules:
/// - Evaluate each `Constrain { predicate }` against the provided environment.
///
/// This is intentionally conservative: it only validates that constraints are
/// well-formed and true under the current environment assumptions.
pub fn validate_proc(
    proc_: &DirProc,
    env: &IndexMap<String, Value>,
) -> Result<PhiValidation, DvmError> {
    if proc_.regime != "Φ" {
        return Err(DvmError::Runtime(format!(
            "phi::validate_proc called on non-Φ proc '{}'(regime={})",
            proc_.name, proc_.regime
        )));
    }

    for stmt in &proc_.body {
        if let crate::dir::DirStmt::Constrain { predicate } = stmt {
            match crate::admissibility::check_predicate(predicate, env) {
                Ok(()) => {}
                Err(e) => {
                    let msg = match e {
                        crate::DvmError::Inadmissible(s) => s,
                        crate::DvmError::ConstraintFailure(s) => s,
                        other => other.to_string(),
                    };
                    return Ok(PhiValidation::LocallyInadmissible { message: msg });
                }
            }
        }
    }

    Ok(PhiValidation::LocallyAdmissible)
}

/// Canonical refusal for Φ execution in v0.1.
///
/// This message is part of the conformance surface.
/// Change only with versioned trace updates.
pub fn refuse_execution() -> DvmError {
    DvmError::UnsupportedRegime("Φ-regime execution wiring into engine is a later step".into())
}
