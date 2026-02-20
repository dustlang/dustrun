# DVM Core API

Source: `crates/dvm/src/lib.rs`

## Public Error Type

`DvmError` variants:

- `DirLoad`
- `DirValidate`
- `EntrypointNotFound`
- `UnsupportedRegime`
- `Inadmissible`
- `ConstraintFailure`
- `EffectViolation`
- `TimeViolation`
- `Runtime`

## Core Config and Outcome Types

- `DvmConfig { effect_mode, trace }`
- `DvmOutcome { returned, effects, time }`
- `DvmFault { error, effects, time }`

## Main Engine Type

- `Dvm::new(cfg)`
- `Dvm::load_dir_json(bytes)`
- `Dvm::validate_dir(program)`
- `Dvm::run_entrypoint(program, entry)`
- `Dvm::run_entrypoint_with_fault(program, entry)`
- `Dvm::run_entrypoint_trace(program, entry)`

## Validation Rules (`validate_dir`)

- program must contain at least one forge,
- forge name must be non-empty,
- proc name must be non-empty,
- proc regime must be one of `K`, `Q`, or `Phi` (`"?"` literal in source).

## Entrypoint Rules

- entrypoint must exist by proc name match,
- entrypoint params are rejected in v0.1 host-runner path.

## Trace Types

- `TraceError`
- `DvmSuccessTrace`
- `DvmFailureTrace`
- `DvmTrace` (untagged enum: success or failure object)

Failure traces optionally include partial `effects` and `time` context when non-empty / non-zero.
