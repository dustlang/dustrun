//! Q-regime host semantics (v0.1):
//! - Enforces linear (non-clonable) resource discipline deterministically.
//! - Does NOT simulate quantum physics amplitudes.
//! - Provides the semantic guardrails needed to develop and test Q-regime programs
//!   without quantum hardware.
//!
//! This module is intentionally backend-agnostic: it can later delegate to
//! quantum hardware backends while preserving DPL semantics.

use crate::DvmError;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A stable identifier for a linear quantum resource within a DVM execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QResId(pub String);

/// The lifecycle state of a linear quantum resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QResState {
    /// Allocated and currently usable.
    Live,

    /// Consumed by an irreversible operation (e.g., measurement) or explicit consume.
    Consumed,

    /// Invalidated due to an error (kept for diagnostics / determinism).
    Invalid,
}

/// Metadata for a quantum resource.
///
/// NOTE: Kept minimal in v0.1. Future revisions can add:
/// - declared shape/register width
/// - backend handle
/// - provenance (which proc allocated it)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QResMeta {
    pub ty: String,
    pub state: QResState,
}

/// A linear binding that refers to a resource.
///
/// In a Q-regime program, user-facing names map to these bindings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QBinding {
    pub res: QResId,
    pub moved: bool, // if true, binding can no longer be used
}

/// Q-regime state container enforcing linearity.
///
/// This is the semantic core: all Q-regime operations must go through this API.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QState {
    /// Resource table: resource id -> metadata
    resources: IndexMap<QResId, QResMeta>,

    /// Name environment for Q bindings: name -> binding
    env: IndexMap<String, QBinding>,

    /// Deterministic allocation counter
    alloc_counter: u64,
}

impl QState {
    /// Create a new QState.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate a new linear quantum resource and bind it to a name.
    pub fn alloc(&mut self, name: &str, ty: &str) -> Result<(), DvmError> {
        if self.env.contains_key(name) {
            return Err(DvmError::Inadmissible(format!(
                "Q alloc failed: name already bound: {name}"
            )));
        }

        let id = self.fresh_id(name);
        self.resources.insert(
            id.clone(),
            QResMeta {
                ty: ty.to_string(),
                state: QResState::Live,
            },
        );

        self.env.insert(
            name.to_string(),
            QBinding {
                res: id,
                moved: false,
            },
        );

        Ok(())
    }

    /// Move ownership of a linear binding: `src` -> `dst`.
    ///
    /// After move:
    /// - `dst` refers to the same resource
    /// - `src` is marked moved and cannot be used again
    pub fn mov(&mut self, src: &str, dst: &str) -> Result<(), DvmError> {
        if self.env.contains_key(dst) {
            return Err(DvmError::Inadmissible(format!(
                "Q move failed: destination already bound: {dst}"
            )));
        }

        let src_binding = self.env.get(src).cloned().ok_or_else(|| {
            DvmError::Inadmissible(format!("Q move failed: unknown binding: {src}"))
        })?;

        if src_binding.moved {
            return Err(DvmError::Inadmissible(format!(
                "Q move failed: binding already moved: {src}"
            )));
        }

        // Ensure resource is live
        self.ensure_live(&src_binding.res, "q_move", src)?;

        // Mark src as moved
        if let Some(b) = self.env.get_mut(src) {
            b.moved = true;
        }

        // Create dst binding
        self.env.insert(
            dst.to_string(),
            QBinding {
                res: src_binding.res,
                moved: false,
            },
        );

        Ok(())
    }

    /// Consume a binding via an irreversible operation (e.g., measurement).
    ///
    /// After consume:
    /// - the binding becomes moved (cannot be reused)
    /// - the resource becomes Consumed (cannot be used by any other alias)
    pub fn consume(&mut self, name: &str, reason: &str) -> Result<(), DvmError> {
        let binding = self.env.get(name).cloned().ok_or_else(|| {
            DvmError::Inadmissible(format!("Q consume failed: unknown binding: {name}"))
        })?;

        if binding.moved {
            return Err(DvmError::Inadmissible(format!(
                "Q consume failed: binding already moved: {name}"
            )));
        }

        self.ensure_live(&binding.res, "q_consume", name)?;

        // Mark resource consumed
        if let Some(meta) = self.resources.get_mut(&binding.res) {
            meta.state = QResState::Consumed;
        }

        // Mark binding moved
        if let Some(b) = self.env.get_mut(name) {
            b.moved = true;
        }

        // Deterministic diagnostic hook (future trace integration).
        let _ = reason;

        Ok(())
    }

    /// Assert that a binding may be used for a reversible unitary-like operation.
    ///
    /// This does not consume the resource, but it must be Live and the binding must not be moved.
    pub fn require_usable(&self, name: &str, op: &str) -> Result<QResId, DvmError> {
        let binding = self.env.get(name).ok_or_else(|| {
            DvmError::Inadmissible(format!("Q use failed: unknown binding: {name} (op={op})"))
        })?;

        if binding.moved {
            return Err(DvmError::Inadmissible(Self::err_use_moved(name, op)));
        }

        self.ensure_live(&binding.res, op, name)?;
        Ok(binding.res.clone())
    }

    /// Get the declared type for a binding's resource (if usable).
    pub fn resource_type(&self, name: &str) -> Result<String, DvmError> {
        let id = self.require_usable(name, "type_query")?;
        let meta = self.resources.get(&id).ok_or_else(|| {
            DvmError::Runtime(format!("Q internal: missing resource meta for {}", id.0))
        })?;
        Ok(meta.ty.clone())
    }

    /// Deterministic snapshot for diagnostics and debugging.
    pub fn snapshot(&self) -> QSnapshot {
        QSnapshot {
            resources: self.resources.clone(),
            env: self.env.clone(),
            alloc_counter: self.alloc_counter,
        }
    }

    // -------------------------
    // internal helpers
    // -------------------------

    fn fresh_id(&mut self, hint: &str) -> QResId {
        self.alloc_counter = self.alloc_counter.saturating_add(1);
        QResId(format!("qres:{}:{}", hint, self.alloc_counter))
    }

    fn err_use_moved(name: &str, op: &str) -> String {
        // CANONICAL ERROR STRING (stable conformance surface)
        // Keep this exact structure unless a versioned trace/error format change is intended.
        format!("Q use failed: binding already moved: {name} (op={op})")
    }

    fn ensure_live(&self, id: &QResId, op: &str, binding_name: &str) -> Result<(), DvmError> {
        let meta = self.resources.get(id).ok_or_else(|| {
            DvmError::Runtime(format!(
                "Q internal: binding '{binding_name}' references missing resource '{}'",
                id.0
            ))
        })?;

        match meta.state {
            QResState::Live => Ok(()),
            QResState::Consumed => Err(DvmError::Inadmissible(format!(
                "Q use failed: resource already consumed: {} (binding={binding_name} op={op})",
                id.0
            ))),
            QResState::Invalid => Err(DvmError::Inadmissible(format!(
                "Q use failed: resource invalid: {} (binding={binding_name} op={op})",
                id.0
            ))),
        }
    }
}

/// A serializable snapshot of Q-regime state (for deterministic replay / debugging).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QSnapshot {
    pub resources: IndexMap<QResId, QResMeta>,
    pub env: IndexMap<String, QBinding>,
    pub alloc_counter: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_and_use_is_ok() {
        let mut q = QState::new();
        q.alloc("a", "QBit").unwrap();
        let id = q.require_usable("a", "H").unwrap();
        assert!(id.0.starts_with("qres:a:"));
        assert_eq!(q.resource_type("a").unwrap(), "QBit");
    }

    #[test]
    fn move_prevents_reuse_of_source() {
        let mut q = QState::new();
        q.alloc("a", "QBit").unwrap();
        q.mov("a", "b").unwrap();

        assert!(q.require_usable("b", "X").is_ok());
        let err = q.require_usable("a", "q_use").unwrap_err();
        match err {
            DvmError::Inadmissible(msg) => {
                assert_eq!(msg, "Q use failed: binding already moved: a (op=q_use)");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn consume_invalidates_all_aliases() {
        let mut q = QState::new();
        q.alloc("a", "QBit").unwrap();
        q.mov("a", "b").unwrap();

        q.consume("b", "measure").unwrap();

        assert!(q.require_usable("b", "H").is_err());
        assert!(q.require_usable("a", "H").is_err());

        let snap = q.snapshot();
        assert_eq!(snap.resources.len(), 1);
        let meta = snap.resources.values().next().unwrap();
        assert_eq!(meta.state, QResState::Consumed);
    }

    #[test]
    fn cannot_move_into_existing_name() {
        let mut q = QState::new();
        q.alloc("a", "QBit").unwrap();
        q.alloc("b", "QBit").unwrap();
        assert!(q.mov("a", "b").is_err());
    }

    #[test]
    fn cannot_alloc_same_name_twice() {
        let mut q = QState::new();
        q.alloc("a", "QBit").unwrap();
        assert!(q.alloc("a", "QBit").is_err());
    }
}
