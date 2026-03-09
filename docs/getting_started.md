# Getting Started

## Prerequisites

- Rust stable toolchain
- Cargo

## Build

From repository root:

```bash
cargo build --workspace --verbose
```

## Run CLI

```bash
cargo run -p dustrun -- tests/fixtures/dir/hello_k.dir.json
```

Run with explicit entrypoint:

```bash
cargo run -p dustrun -- tests/fixtures/dir/hello_k.dir.json --entry main
```

Emit JSON trace:

```bash
cargo run -p dustrun -- tests/fixtures/dir/hello_k.dir.json --emit-trace
```

Select effect mode:

```bash
cargo run -p dustrun -- tests/fixtures/dir/hello_k.dir.json --effects simulate
cargo run -p dustrun -- tests/fixtures/dir/hello_k.dir.json --effects realize
```

## Run Tests

```bash
cargo test --workspace --verbose
```

Run conformance tests only:

```bash
cargo test -p dustrun-conformance --verbose
```

Bless conformance goldens:

```bash
DUST_BLESS=1 cargo test -p dustrun-conformance --verbose
```
