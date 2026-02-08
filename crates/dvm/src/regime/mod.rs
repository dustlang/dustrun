# File: crates/dvm/src/regime/mod.rs
#
# Regime module root for the Dust Virtual Machine (DVM).
#
# This module groups regime-specific semantic enforcement and host-mode execution
# components. Regime semantics are enforced by the DVM regardless of whether
# execution is native, VM, or delegated to specialized backends.

pub mod q;

pub use q::{
    QBinding, QResId, QResMeta, QResState, QSnapshot, QState,
};