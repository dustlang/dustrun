# Runtime Support Module (`runtime.rs`)

Source: `crates/dvm/src/runtime.rs`

## Status

`runtime.rs` exists in the repository but is not currently wired by `lib.rs` (`mod runtime` is absent).

This means its exported `extern "C"` symbols are not part of the active `dust-dvm` public API unless integration is added.

## Contents

The file defines C-ABI style helpers for:

- heap allocation and reallocation,
- string operations (`DustString`),
- panic/assert helpers,
- primitive conversions,
- array allocation and element access,
- placeholder `dust_main` entry.

## Intended Role

The module appears designed as a runtime support layer for compiled Dust programs, separate from DVM DIR execution semantics.

## Integration Caution

Before treating this module as production API, verify:

- it is included in crate module graph,
- ABI surface is versioned/documented,
- behavior is covered by tests and conformance criteria.
