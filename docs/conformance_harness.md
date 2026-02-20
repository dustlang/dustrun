# Conformance Harness

Sources:

- `crates/conformance/src/lib.rs`
- `crates/conformance/tests/conformance.rs`

## Purpose

Validate deterministic DVM behavior against fixture-defined goldens.

## Core Types

- `ConformanceError`
- `Fixture`
- `RunnerConfig { bless }`
- `Runner`

## Fixture Fields

- `name`
- `dir`
- `entry` (default `main`)
- `effects` (default `simulate`)
- `trace` (default `false`)
- exactly one of:
  - `expect_trace`
  - `expect_error`

`Fixture::validate()` enforces the one-of rule.

## Runner Flow

1. Load fixture JSON.
2. Resolve DIR path relative to fixture file.
3. Run DVM with fixture config.
4. Produce `DvmTrace`.
5. Compare to golden (`assert_matches`) or rewrite when `bless=true`.

## Bless Mode

- set env `DUST_BLESS=1`
- rerun conformance tests
- produced traces overwrite referenced golden files.
