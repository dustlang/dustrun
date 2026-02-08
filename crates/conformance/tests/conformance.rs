// crates/conformance/tests/conformance.rs
//
// Conformance test runner.
//
// This test scans a fixture directory for JSON fixture files and checks that
// produced DVM traces match golden traces deterministically.
//
// To bless (rewrite) golden traces:
//   DUST_BLESS=1 cargo test -p dustrun-conformance
//
// Fixtures live in:
//   tests/fixtures/*.json
//
// Each fixture references exactly one golden file via:
//   - `expect_trace` (success trace), or
//   - `expect_error` (failure trace)

use dustrun_conformance::{Runner, RunnerConfig};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture_root() -> PathBuf {
    // Workspace layout: crates/conformance/tests/ -> ../../tests/fixtures (repo root)
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("tests").join("fixtures")
}

fn list_fixture_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(root) {
        for ent in rd.flatten() {
            let p = ent.path();
            if p.is_file() && p.extension().map(|e| e == "json").unwrap_or(false) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn conformance_fixtures_match_golden() {
    let bless = std::env::var("DUST_BLESS").ok().as_deref() == Some("1");

    let runner = Runner::new(RunnerConfig { bless });

    let root = fixture_root();
    let files = list_fixture_files(&root);

    assert!(
        !files.is_empty(),
        "no fixture files found in {}",
        root.display()
    );

    for f in files {
        runner.run_and_check(&f).unwrap_or_else(|e| {
            panic!("fixture failed: {}\nerror: {e}", f.display());
        });
    }
}
