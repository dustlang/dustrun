# Fixtures and Golden Files

## Directories

- `tests/fixtures/*.json`: conformance fixture descriptors.
- `tests/fixtures/dir/*.dir.json`: DIR program inputs.
- `tests/golden/*.trace.json`: expected traces.

## Fixture Descriptor Example

```json
{
  "name": "hello_k",
  "dir": "dir/hello_k.dir.json",
  "entry": "main",
  "effects": "simulate",
  "trace": false,
  "expect_trace": "../golden/hello_k.trace.json"
}
```

Error-case fixture uses `expect_error` instead.

## Updating Goldens

1. Make intentional semantic changes.
2. Run: `DUST_BLESS=1 cargo test -p dustrun-conformance`.
3. Review changed `tests/golden/*.trace.json`.
4. Commit fixtures and code together.

## Determinism Expectation

Given unchanged code and fixtures, conformance traces must remain byte-stable modulo pretty JSON formatting consistency.
