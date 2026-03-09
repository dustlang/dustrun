# Errors and Exit Codes

## DVM Error Taxonomy

From `DvmError`:

- `DirLoad`
- `DirValidate`
- `EntrypointNotFound`
- `UnsupportedRegime`
- `Inadmissible`
- `ConstraintFailure`
- `EffectViolation`
- `TimeViolation`
- `Runtime`

These are mapped into `TraceError { kind, message }` for trace outputs.

## CLI Exit Codes (`crates/dustrun/src/main.rs`)

- `2`: failed to read DIR file.
- `3`: DIR load error.
- `10`: semantic/runtime execution failure (`run_entrypoint` error).
- `4`: failed to serialize JSON trace.

Success exits with process code `0`.

## Quiet Mode

With `--quiet`, semantic failure message printing is suppressed, but CLI still exits with code `10`.
