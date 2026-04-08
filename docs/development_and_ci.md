# Development and CI

## Local Commands

Build all crates:

```bash
cargo build --workspace --verbose
```

Run all tests:

```bash
cargo test --workspace --verbose
```

Run only conformance tests:

```bash
cargo test -p dustrun-conformance --verbose
```

## CI Workflow

File: `.github/workflows/ci.yml`

Pipeline runs:

1. checkout,
2. Rust stable install,
3. workspace build,
4. workspace tests.

## Logging Guidance

Runtime trace logs are controlled by `RUST_LOG` and `--trace` usage with deterministic formatter configuration in CLI.

## Recommended Documentation Maintenance

When semantics change:

1. update docs in `dustrun/docs`,
2. update or bless golden traces,
3. run full workspace tests,
4. include rationale in changelog.
