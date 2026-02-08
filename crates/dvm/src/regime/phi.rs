//! Φ-regime host-mode semantics (v0.1):
//!
//! This module provides **validation-only** semantics for Φ-regime programs.
//! It does NOT execute Φ-regime computation yet.
//!
//! Responsibilities in v0.1:
//! - Recognize Φ-regime as a distinct regime.
//! - Provide deterministic, semantically meaningful refusal for execution.
//! - Provide hooks for future admissibility witness generation.
//!
//! The key idea is that Φ-regime programs must be globally admissible.
//! In v0.1 we can only do local, deterministic checks and then return a
//! stable "not yet executable" outcome.

use crate::DvmError;
use crate::{admissibility, DirProc, Value};
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
/// - `Let`/`Effect`/`Return` are not evaluated here (Φ execution not implemented).
///
/// This is intentionally conservative: it only validates that constraints are
/// well-formed and true under the current environment assumptions.
///
/// Future revisions will:
/// - incorporate global constraint graphs,
/// - incorporate witness construction,
/// - incorporate non-existence proofs.
pub fn validate_proc(proc_: &DirProc, env: &IndexMap<String, Value>) -> Result<PhiValidation, DvmError> {
    if proc_.regime != "Φ" {
        return Err(DvmError::Runtime(format!(
            "phi::validate_proc called on non-Φ proc '{}'(regime={})",
            proc_.name, proc_.regime
        )));
    }

    for stmt in &proc_.body {
        match stmt {
            crate::dir::DirStmt::Constrain { predicate } => {
                match admissibility::check_predicate(predicate, env) {
                    Ok(()) => {}
                    Err(e) => {
                        // Collapse all local failures into a deterministic validation result.
                        let msg = match e {
                            DvmError::Inadmissible(s) => s,
                            DvmError::ConstraintFailure(s) => s,
                            other => other.to_string(),
                        };
                        return Ok(PhiValidation::LocallyInadmissible { message: msg });
                    }
                }
            }
            // In validation-only mode we ignore other statements;
            // they will become meaningful when Φ execution is implemented.
            _ => {}
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