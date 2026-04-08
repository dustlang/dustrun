# DIR Model

Source: `crates/dvm/src/lib.rs` (`dir` module)

## Root

```json
{
  "forges": [ ... ]
}
```

## Major Types

- `DirProgram`
- `DirForge`
- `DirShape`
- `DirField`
- `DirProc`
- `DirParam`
- `DirUses`
- `DirLit`
- `DirStmt`
- `DirBind`
- `DirClause`

## Procedure Regimes

`DirProc.regime` is a string with expected values:

- `K`
- `Q`
- `Phi` represented as unicode `"?"`

## Statement Variants

`DirStmt`:

- `Let { name, expr }`
- `Constrain { predicate }`
- `Prove { name, from }`
- `Effect { kind, payload }`
- `Return { expr }`

## Notes

- DIR is loaded from JSON bytes via serde.
- DVM treats expression/predicate strings as evaluator input, not as source language AST.
- `uses`, `binds`, and shapes are modeled but v0.1 execution logic primarily uses proc body and regime semantics.
