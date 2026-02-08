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

use dust_dvm::{Dvm, DvmConfig, DvmError, DvmTrace, EffectMode};
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

    #[error("golden mismatch: {0}")]
    GoldenMismatch(String),

    #[error("fixture invalid: {0}")]
    FixtureInvalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub name: String,
    pub dir: String,

    #[serde(default = "default_entry")]
    pub entry: String,

    #[serde(default = "default_effects")]
    pub effects: String,

    #[serde(default)]
    pub trace: bool,

    #[serde(default)]
    pub expect_trace: Option<String>,

    #[serde(default)]
    pub expect_error: Option<String>,
}

fn default_entry() -> String {
    "main".into()
}
fn default_effects() -> String {
    "simulate".into()
}

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

    pub fn expect_trace_path(&self, fixture_file: &Path) -> Result<PathBuf, ConformanceError> {
        let rel = self
            .expect_trace
            .as_ref()
            .ok_or_else(|| ConformanceError::FixtureInvalid("missing expect_trace".into()))?;
        Ok(self.base_dir(fixture_file).join(rel))
    }

    pub fn expect_error_path(&self, fixture_file: &Path) -> Result<PathBuf, ConformanceError> {
        let rel = self
            .expect_error
            .as_ref()
            .ok_or_else(|| ConformanceError::FixtureInvalid("missing expect_error".into()))?;
        Ok(self.base_dir(fixture_file).join(rel))
    }

    pub fn effect_mode(&self) -> Result<EffectMode, ConformanceError> {
        match self.effects.as_str() {
            "simulate" => Ok(EffectMode::Simulate),
            "realize" => Ok(EffectMode::Realize),
            other => Err(ConformanceError::FixtureInvalid(format!(
                "fixture '{}' has unknown effects mode '{}'",
                self.name, other
            ))),
        }
    }

    pub fn validate(&self) -> Result<(), ConformanceError> {
        let has_trace = self.expect_trace.is_some();
        let has_error = self.expect_error.is_some();

        match (has_trace, has_error) {
            (true, false) => Ok(()),
            (false, true) => Ok(()),
            (false, false) => Err(ConformanceError::FixtureInvalid(format!(
                "fixture '{}' must specify exactly one of expect_trace or expect_error",
                self.name
            ))),
            (true, true) => Err(ConformanceError::FixtureInvalid(format!(
                "fixture '{}' must not specify both expect_trace and expect_error",
                self.name
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedError {
    pub error: ExpectedErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExpectedErrorBody {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Produced {
    Success(DvmTrace),
    Failure(ExpectedError),
}

impl Runner {
    pub fn new(cfg: RunnerConfig) -> Self {
        Self { cfg }
    }

    pub fn load_fixture(path: impl AsRef<Path>) -> Result<Fixture, ConformanceError> {
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn run_fixture(
        &self,
        fixture_file: impl AsRef<Path>,
        fixture: &Fixture,
    ) -> Result<Produced, ConformanceError> {
        let fixture_file = fixture_file.as_ref();

        let dir_path = fixture.dir_path(fixture_file);
        let dir_bytes = fs::read(&dir_path)?;

        let dvm = Dvm::new(DvmConfig {
            effect_mode: fixture.effect_mode()?,
            trace: fixture.trace,
        });

        let program = match dvm.load_dir_json(&dir_bytes) {
            Ok(p) => p,
            Err(e) => return Ok(Produced::Failure(map_dvm_error(e))),
        };

        match dvm.run_entrypoint(&program, &fixture.entry) {
            Ok(outcome) => Ok(Produced::Success(outcome.into())),
            Err(e) => Ok(Produced::Failure(map_dvm_error(e))),
        }
    }

    pub fn assert_matches(
        &self,
        fixture_file: impl AsRef<Path>,
        fixture: &Fixture,
        produced: &Produced,
    ) -> Result<(), ConformanceError> {
        let fixture_file = fixture_file.as_ref();

        if self.cfg.bless {
            if fixture.expect_trace.is_some() {
                let p = fixture.expect_trace_path(fixture_file)?;
                fs::create_dir_all(p.parent().unwrap_or_else(|| Path::new(".")))?;
                let s = match produced {
                    Produced::Success(t) => serde_json::to_string_pretty(t)?,
                    Produced::Failure(e) => serde_json::to_string_pretty(e)?,
                };
                fs::write(p, s.as_bytes())?;
                return Ok(());
            }

            if fixture.expect_error.is_some() {
                let p = fixture.expect_error_path(fixture_file)?;
                fs::create_dir_all(p.parent().unwrap_or_else(|| Path::new(".")))?;
                let s = match produced {
                    Produced::Success(t) => serde_json::to_string_pretty(t)?,
                    Produced::Failure(e) => serde_json::to_string_pretty(e)?,
                };
                fs::write(p, s.as_bytes())?;
                return Ok(());
            }
        }

        if fixture.expect_trace.is_some() {
            let golden_path = fixture.expect_trace_path(fixture_file)?;
            let golden_bytes = fs::read(&golden_path)?;
            let golden: DvmTrace = serde_json::from_slice(&golden_bytes)?;

            match produced {
                Produced::Success(t) => {
                    if &golden != t {
                        return Err(ConformanceError::GoldenMismatch(format!(
                            "fixture '{}' produced trace does not match golden.\nfixture_file: {}\ndir: {}\nexpected: {}\n",
                            fixture.name,
                            fixture_file.display(),
                            fixture.dir_path(fixture_file).display(),
                            golden_path.display(),
                        )));
                    }
                    Ok(())
                }
                Produced::Failure(e) => Err(ConformanceError::GoldenMismatch(format!(
                    "fixture '{}' expected SUCCESS but got ERROR.\nfixture_file: {}\ndir: {}\nexpected: {}\nproduced_error_kind: {}\nproduced_error_message: {}\n",
                    fixture.name,
                    fixture_file.display(),
                    fixture.dir_path(fixture_file).display(),
                    golden_path.display(),
                    e.error.kind,
                    e.error.message
                ))),
            }
        } else {
            let golden_path = fixture.expect_error_path(fixture_file)?;
            let golden_bytes = fs::read(&golden_path)?;
            let golden: ExpectedError = serde_json::from_slice(&golden_bytes)?;

            match produced {
                Produced::Failure(e) => {
                    if &golden != e {
                        return Err(ConformanceError::GoldenMismatch(format!(
                            "fixture '{}' produced error does not match golden.\nfixture_file: {}\ndir: {}\nexpected: {}\n",
                            fixture.name,
                            fixture_file.display(),
                            fixture.dir_path(fixture_file).display(),
                            golden_path.display(),
                        )));
                    }
                    Ok(())
                }
                Produced::Success(t) => Err(ConformanceError::GoldenMismatch(format!(
                    "fixture '{}' expected ERROR but got SUCCESS.\nfixture_file: {}\ndir: {}\nexpected: {}\nproduced_returned: {:?}\n",
                    fixture.name,
                    fixture_file.display(),
                    fixture.dir_path(fixture_file).display(),
                    golden_path.display(),
                    t.returned
                ))),
            }
        }
    }

    pub fn run_and_check(&self, fixture_file: impl AsRef<Path>) -> Result<(), ConformanceError> {
        let fixture_file = fixture_file.as_ref();
        let fixture = Self::load_fixture(fixture_file)?;
        fixture.validate()?;
        let produced = self.run_fixture(fixture_file, &fixture)?;
        self.assert_matches(fixture_file, &fixture, &produced)
    }
}

fn map_dvm_error(e: DvmError) -> ExpectedError {
    // IMPORTANT: Use inner messages for stability.
    // We do NOT use `e.to_string()` because it includes outer formatting such as:
    // "inadmissible program: <msg>"
    //
    // The conformance surface wants the canonical inner message for deterministic matching.
    let (kind, message) = match e {
        DvmError::DirLoad(s) => ("DirLoad", s),
        DvmError::DirValidate(s) => ("DirValidate", s),
        DvmError::EntrypointNotFound(s) => ("EntrypointNotFound", s),
        DvmError::UnsupportedRegime(s) => ("UnsupportedRegime", s),
        DvmError::Inadmissible(s) => ("Inadmissible", s),
        DvmError::ConstraintFailure(s) => ("ConstraintFailure", s),
        DvmError::EffectViolation(s) => ("EffectViolation", s),
        DvmError::TimeViolation(s) => ("TimeViolation", s),
        DvmError::Runtime(s) => ("Runtime", s),
    };

    ExpectedError {
        error: ExpectedErrorBody {
            kind: kind.to_string(),
            message,
        },
    }
}}