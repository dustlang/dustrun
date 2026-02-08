//! Φ-regime witness stubs (v0.1).
//!
//! In the Φ regime, execution is governed by admissibility and global consistency.
//! A **witness** is the program-visible artifact that attests:
//! - a constraint set is admissible under the regime rules, or
//! - a constraint set is non-existent (inadmissible) under the regime rules.
//!
//! In v0.1, we do not build global witnesses yet.
//! We provide deterministic **stub witness objects** so developers can:
//! - write programs that request witnesses,
//! - test compilation + runtime wiring,
//! - build tooling around witness transport,

//! - This module provides witness stubs
//!   without Φ execution being implemented.

use crate::DvmError;
use serde::{Deserialize, Serialize};

/// Witness kind for Φ-regime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhiWitnessKind {
    /// A witness that a constraint set is admissible.
    Admissible,

    /// A witness that a constraint set is non-existent (inadmissible).
    NonExistent,
}

/// A deterministic witness envelope.
///
/// v0.1 guarantees:
/// - fully deterministic fields
/// - stable JSON serialization
/// - no host-specific data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhiWitness {
    /// Witness kind.
    pub kind: PhiWitnessKind,

    /// Deterministic identifier for this witness within a run.
    pub id: String,

    /// Canonical digest of the constraint set (v0.1: string digest, not cryptographic).
    ///
    /// Future versions can switch to a cryptographic digest with versioning.
    pub constraint_digest: String,

    /// Human-readable explanation string (stable, not verbose).
    pub note: String,
}

/// v0.1 witness builder:
/// - Accepts a constraint digest string (caller-provided, deterministic).
/// - Returns a deterministic witness id.
/// - Does not perform global proof construction.
#[derive(Debug, Default)]
pub struct PhiWitnessBuilder {
    counter: u64,
}

impl PhiWitnessBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    fn next_id(&mut self) -> String {
        self.counter = self.counter.saturating_add(1);
        format!("Φwitness:{}", self.counter)
    }

    /// Construct a stub admissible witness.
    pub fn admissible(&mut self, constraint_digest: &str) -> PhiWitness {
        PhiWitness {
            kind: PhiWitnessKind::Admissible,
            id: self.next_id(),
            constraint_digest: constraint_digest.to_string(),
            note: "Φ witness stub (v0.1): admissible".into(),
        }
    }

    /// Construct a stub non-existence witness.
    pub fn non_existent(&mut self, constraint_digest: &str, reason: &str) -> PhiWitness {
        PhiWitness {
            kind: PhiWitnessKind::NonExistent,
            id: self.next_id(),
            constraint_digest: constraint_digest.to_string(),
            note: format!("Φ witness stub (v0.1): non-existent: {reason}"),
        }
    }
}

/// Canonical refusal for full witness construction in v0.1.
///
/// This is separate from execution refusal: witness *stubs* exist, but global proof does not.
pub fn refuse_global_witness() -> DvmError {
    DvmError::UnsupportedRegime("Φ global witness construction is not implemented in v0.1".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_ids_are_deterministic() {
        let mut b = PhiWitnessBuilder::new();
        let w1 = b.admissible("c0");
        let w2 = b.non_existent("c1", "failed");
        assert_eq!(w1.id, "Φwitness:1");
        assert_eq!(w2.id, "Φwitness:2");
        assert_eq!(w1.constraint_digest, "c0");
        assert_eq!(w2.constraint_digest, "c1");
        assert_eq!(w1.kind, PhiWitnessKind::Admissible);
        assert_eq!(w2.kind, PhiWitnessKind::NonExistent);
    }

    #[test]
    fn witness_serialization_is_stable() {
        let mut b = PhiWitnessBuilder::new();
        let w = b.admissible("digest:example");
        let s = serde_json::to_string(&w).unwrap();
        // Sanity: key fields exist.
        assert!(s.contains("\"kind\""));
        assert!(s.contains("\"id\""));
        assert!(s.contains("\"constraint_digest\""));
        assert!(s.contains("digest:example"));
    }
}
