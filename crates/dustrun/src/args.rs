// File: args.rs - This file is part of the DPL Toolchain
// Copyright (c) 2026 Dust LLC, and Contributors
// Description:
//   Command-line argument definitions for `dustrun` executable.
//   Defines the public CLI contract only (no execution logic).
//   Provides: Args struct, EffectModeArg, Target triple handling.

use clap::{Parser, ValueEnum};

/// Dust Virtual Machine (DVM) reference executor.
///
/// Executes Dust Intermediate Representation (DIR) artifacts according
/// to the DPL specification.
#[derive(Debug, Parser)]
#[command(name = "dustrun")]
#[command(author = "Dust Research Division")]
#[command(version = "0.1.0")]
#[command(about = "Dust Virtual Machine (DVM) reference executor", long_about = None)]
pub struct Args {
    /// Path to the DIR artifact (JSON)
    #[arg(value_name = "DIR_FILE")]
    pub dir_path: String,

    /// Entrypoint procedure name
    ///
    /// If not specified, defaults to `main`.
    #[arg(short, long, default_value = "main")]
    pub entry: String,

    /// Effect handling mode
    ///
    /// - simulate: effects are logged only
    /// - realize: effects may be enacted (when realizers are configured)
    #[arg(long, value_enum, default_value = "simulate")]
    pub effects: EffectModeArg,

    /// Enable execution tracing
    ///
    /// When enabled, each logical tick and executed statement
    /// is logged deterministically.
    #[arg(long)]
    pub trace: bool,

    /// Emit execution trace as JSON to stdout
    ///
    /// Intended for deterministic replay and testing.
    #[arg(long)]
    pub emit_trace: bool,

    /// Suppress non-essential output
    ///
    /// When set, only structured outputs (JSON) are printed.
    #[arg(long)]
    pub quiet: bool,
}

/// CLI-visible effect mode selector.
///
/// This is intentionally a thin wrapper over the DVM's internal EffectMode.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum EffectModeArg {
    Simulate,
    Realize,
}

impl EffectModeArg {
    pub fn as_str(&self) -> &'static str {
        match self {
            EffectModeArg::Simulate => "simulate",
            EffectModeArg::Realize => "realize",
        }
    }
}
