# dustrun Documentation

This directory contains complete Markdown documentation for `dustrun`.

## Documentation Index

- `getting_started.md`: build, run, and basic commands.
- `architecture.md`: workspace and crate boundaries.
- `cli_reference.md`: `dustrun` command-line contract.
- `dvm_core_api.md`: core types, APIs, and execution flow in `dust-dvm`.
- `dir_model.md`: DIR JSON model expected by the DVM.
- `expression_semantics.md`: expression grammar and evaluator behavior.
- `regime_semantics.md`: K, Q, and Phi regime behavior.
- `traces_and_outputs.md`: human output, JSON trace output, and trace types.
- `conformance_harness.md`: fixture schema and golden trace validation.
- `fixtures_and_goldens.md`: fixture directories and update workflow.
- `error_and_exit_codes.md`: DVM errors and CLI process exit behavior.
- `runtime_support_module.md`: status and API of `crates/dvm/src/runtime.rs`.
- `development_and_ci.md`: tests, CI pipeline, and local validation.
- `trace-schema.md`: existing normative conformance trace schema.

## Scope

`dustrun` is the reference executor for Dust DIR artifacts. It loads DIR JSON, runs one entrypoint using DVM semantics, and emits deterministic outcomes.

It is not a compiler and does not parse Dust source files.
