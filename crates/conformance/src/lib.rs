# File: crates/conformance/src/lib.rs
#
# Conformance harness for DVM semantics.
#
# Purpose:
# - Execute DIR fixtures through dust-dvm deterministically
# - Emit traces
# - Compare traces against golden expectations
#
# This crate is non-normative with respect to language semantics.
# It is normative for conformance enforcement within the dustrun repository.

use dust_dvm::{Dvm, DvmConfig, DvmTrace, EffectMode, DvmError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConformanceError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("dvm error: {0}")]
    Dvm(#[from] DvmError),

    #[error("golden mismatch: {0}")]
    GoldenMismatch(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    /// Human-readable name for the test case.
    pub name: String,

    /// Relative path to DIR JSON file (from fixture file directory).
    pub dir: String,

    /// Entrypoint proc name.
    #[serde(default = "default_entry")]
    pub entry: String,

    /// Effect mode: "simulate" or "realize"
    #[serde(default = "default_effects")]
    pub effects: String,

    /// Whether to enable DVM trace logging (does not affect trace JSON content).
    #[serde(default)]
    pub trace: bool,

    /// Relative path to expected trace JSON file (from fixture file directory).
    pub expect_trace: String,
}

fn default_entry() -> String { "main".into() }
fn default_effects() -> String { "simulate".into() }

impl Fixture {
    pub fn base_dir(&self, fixture_file: &Path) -> PathBuf {
        fixture_file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn dir_path(&self, fixture_file: &Path) -> PathBuf {
        self.base_dir(fixture_file).join(&self.dir)
    }

    pub fn expect_trace_path(&self, fixture_file: &Path) -> PathBuf {
        self.base_dir(fixture_file).join(&self.expect_trace)
    }

    pub fn effect_mode(&self) -> Result<EffectMode, ConformanceError> {
        match self.effects.as_str() {
            "simulate" => Ok(EffectMode::Simulate),
            "realize" => Ok(EffectMode::Realize),
            other => Err(ConformanceError::GoldenMismatch(format!(
                "fixture '{}' has unknown effects mode '{}'",
                self.name, other
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// If true, rewrite golden traces to match current behavior.
    /// This should only be used when intentionally updating semantics
    /// or improving the trace format in a controlled versioned change.
    pub bless: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self { bless: false }
    }
}

pub struct Runner {
    cfg: RunnerConfig,
}

impl Runner {
    pub fn new(cfg: RunnerConfig) -> Self {
        Self { cfg }
    }

    /// Load a fixture from JSON file.
    pub fn load_fixture(path: impl AsRef<Path>) -> Result<Fixture, ConformanceError> {
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Execute a fixture and return the produced trace.
    pub fn run_fixture(&self, fixture_file: impl AsRef<Path>, fixture: &Fixture) -> Result<DvmTrace, ConformanceError> {
        let fixture_file = fixture_file.as_ref();

        let dir_path = fixture.dir_path(fixture_file);
        let dir_bytes = fs::read(&dir_path)?;

        let dvm = Dvm::new(DvmConfig {
            effect_mode: fixture.effect_mode()?,
            trace: fixture.trace,
        });

        let program = dvm.load_dir_json(&dir_bytes)?;
        let outcome = dvm.run_entrypoint(&program, &fixture.entry)?;
        Ok(outcome.into())
    }

    /// Compare a produced trace against a golden trace file.
    /// If `bless` is enabled, rewrite the golden file instead of failing.
    pub fn assert_trace_matches(
        &self,
        fixture_file: impl AsRef<Path>,
        fixture: &Fixture,
        produced: &DvmTrace,
    ) -> Result<(), ConformanceError> {
        let fixture_file = fixture_file.as_ref();
        let golden_path = fixture.expect_trace_path(fixture_file);

        if self.cfg.bless {
            let s = serde_json::to_string_pretty(produced)?;
            fs::create_dir_all(golden_path.parent().unwrap_or_else(|| Path::new(".")))?;
            fs::write(&golden_path, s.as_bytes())?;
            return Ok(());
        }

        let golden_bytes = fs::read(&golden_path)?;
        let golden: DvmTrace = serde_json::from_slice(&golden_bytes)?;

        if &golden != produced {
            let msg = format!(
                "fixture '{}' produced trace does not match golden.\nfixture_file: {}\ndir: {}\nexpected: {}\n",
                fixture.name,
                fixture_file.display(),
                fixture.dir_path(fixture_file).display(),
                golden_path.display(),
            );
            return Err(ConformanceError::GoldenMismatch(msg));
        }

        Ok(())
    }

    /// Convenience: load, run, and compare a fixture file.
    pub fn run_and_check(&self, fixture_file: impl AsRef<Path>) -> Result<(), ConformanceError> {
        let fixture_file = fixture_file.as_ref();
        let fixture = Self::load_fixture(fixture_file)?;
        let produced = self.run_fixture(fixture_file, &fixture)?;
        self.assert_trace_matches(fixture_file, &fixture, &produced)
    }
}