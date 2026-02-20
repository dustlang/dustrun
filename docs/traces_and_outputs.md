# Traces and Outputs

Sources:

- `crates/dustrun/src/main.rs`
- `crates/dvm/src/lib.rs`
- `docs/trace-schema.md`

## Human Output (default)

When not quiet and not emit-trace:

- `return: ...`
- `effects: ...`
- `time.ticks: ...`
- `effect_mode: ...`
- `entry: ...`

## JSON Trace Output (`--emit-trace`)

CLI currently wraps success outcomes as:

- `DvmTrace::Success` with fields `returned`, `effects`, `time`.

In conformance harness and DVM API, traces can be:

- success trace,
- failure trace with `error`, optional `effects`, optional `time`.

## Effect Log

`EffectLog.events` is ordered and deterministic.
Each event has:

- `kind`
- `payload`

For struct payload values, payload is rendered as JSON string.

## Time Model

`TimeState.tick` is logical deterministic time.

- increments once per processed statement in K/Q non-return steps,
- increments once per processed statement in Phi host loop,
- omitted from failure trace when tick is zero.

## Normative Schema

`trace-schema.md` is the repository's normative conformance trace shape for test artifacts.
