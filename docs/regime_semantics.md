# Regime Semantics

Sources:

- `crates/dvm/src/lib.rs` (engine)
- `crates/dvm/src/regime/q.rs`
- `crates/dvm/src/regime/phi.rs`
- `crates/dvm/src/regime/phi_witness.rs`

## K Regime

- Executes statements sequentially.
- `Let` evaluates expression and binds value.
- `Constrain` checks admissibility predicate.
- `Prove` checks predicate and binds unit value.
- `Effect` logs rendered payload.
- `Return` ends execution with value.
- tick increments after non-return statements.

## Q Regime

Uses `QState` linear resource model.

Supported intrinsic-like forms in `Let.expr`:

- `q_alloc(Ty)`
- `q_move(name)`
- `q_use(name)`
- `q_consume(name)`

`QState` enforces:

- no duplicate alloc names,
- no use after move,
- no use after consume,
- deterministic resource ids (`qres:<hint>:<counter>`).

## Phi Regime (v0.1 Host Mode)

Flow:

1. Validate local constraints with `phi_validate_proc`.
2. Execute host-mode loop for witness-related operations.
3. Deterministically refuse full Phi execution with `UnsupportedRegime` error.

Supported host intrinsic in `Let.expr`:

- `phi_witness(<expr>)` where evaluated argument must be `String` digest.

Witnesses are produced by `PhiWitnessBuilder` and injected as `Value::Struct` (`ty = "PhiWitness"`).

`Prove` in Phi path creates admissible witness from digest `pred:<predicate>`.
