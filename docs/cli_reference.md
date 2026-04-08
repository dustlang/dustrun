# CLI Reference

Sources:

- `crates/dustrun/src/args.rs`
- `crates/dustrun/src/main.rs`

## Positional Argument

- `DIR_FILE`: path to DIR JSON artifact.

## Flags

- `--entry <name>`
  - Entrypoint proc name.
  - Default: `main`.

- `--effects <simulate|realize>`
  - Maps to DVM effect mode.
  - Default: `simulate`.

- `--trace`
  - Enables runtime statement/tick logging via `log::info!`.

- `--emit-trace`
  - Emits structured JSON `DvmTrace` to stdout.

- `--quiet`
  - Suppresses human-readable summary output.

## Runtime Behavior

- Reads DIR file with `std::fs::read`.
- On `--emit-trace`, serializes `DvmTrace::Success` from `run_entrypoint` outcome.
- Without `--emit-trace`, prints:
  - return value,
  - effects list,
  - logical tick,
  - selected effect mode,
  - entry name.

## Logging

`init_logging()` configures `env_logger` with default filter `warn` and no timestamps.

## Version Note

`args.rs` clap version metadata is `0.1.0` while crate version is `0.2.0`.
