# Architecture

## Workspace Crates

- `crates/dvm` (`dust-dvm`): semantic authority and execution engine.
- `crates/dustrun` (`dustrun`): CLI wrapper over `dust-dvm`.
- `crates/conformance` (`dustrun-conformance`): deterministic fixture runner and golden comparison.

## High-Level Flow

1. CLI parses args (`crates/dustrun/src/args.rs`).
2. CLI reads DIR file bytes and builds `DvmConfig`.
3. DVM loads DIR JSON (`Dvm::load_dir_json`).
4. DVM runs selected entrypoint (`run_entrypoint` or trace API).
5. CLI prints human output or emits `DvmTrace` JSON.

## Separation of Concerns

- CLI crate contains no execution semantics.
- DVM crate contains no argument parsing.
- Conformance crate validates output stability using fixtures and golden traces.
