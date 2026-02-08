# DVM Trace Schema (Conformance)

This document defines the **stable JSON shape** used by the dustrun conformance harness.
It exists to prevent accidental drift in test artifacts and to ensure deterministic replay.

This schema is **normative for tests** in this repository.
It does not define the full semantic meaning of execution — only the observable trace surface.

───────────────────────────────────────────────────────────────────────────────

1. Trace Envelope

A conformance run produces exactly one of the following outcomes.

1.1 Success Trace

{
  "returned": <value-or-null>,
  "effects": {
    "events": [ <effect-event> ... ]
  },
  "time": {
    "tick": <u64>
  }
}

Rules:
- `returned` may be null if the entrypoint returns no value.
- `effects.events` is an ordered list with deterministic ordering.
- `time.tick` is a deterministic logical tick counter.
- No additional top-level fields are permitted.

───────────────────────────────────────────────────────────────────────────────

1.2 Failure Trace

{
  "error": {
    "kind": "<ErrorKind>",
    "message": "<stable message>"
  }
}

Rules:
- Failure traces represent semantic refusal to execute or deterministic runtime failure.
- Error messages must be stable across executions.
- Error messages must not include incidental formatting, stack traces, or host-specific data.

───────────────────────────────────────────────────────────────────────────────

2. Value Encoding

The DVM uses tagged value encoding to preserve semantic clarity and future extensibility.

2.1 Integer

{ "Int": 42 }

2.2 Boolean

{ "Bool": true }

2.3 String

{ "String": "hello" }

2.4 Unit

"Unit"

2.5 Struct

{
  "Struct": {
    "ty": "MyType",
    "fields": {
      "a": { "Int": 1 },
      "b": { "String": "x" }
    }
  }
}

Rules:
- Field order is deterministic.
- Nested values must follow the same encoding rules recursively.
- No implicit coercions are permitted.

───────────────────────────────────────────────────────────────────────────────

3. Effect Encoding

Effects are recorded even in `simulate` mode.

{
  "kind": "emit",
  "payload": "Hello"
}

Rules:
- `kind` is a lowercase identifier (emit, observe, seal, couple, etc).
- `payload` is a rendered string produced by evaluating the payload expression.
- Effects are recorded in execution order.

───────────────────────────────────────────────────────────────────────────────

4. Time Encoding

Logical time is represented as a monotonic counter.

{
  "time": {
    "tick": 3
  }
}

Rules:
- `tick` increments once per executed statement.
- Tick behavior is deterministic and architecture-independent.
- No wall-clock or real-time data may appear in traces.

───────────────────────────────────────────────────────────────────────────────

5. Stability Rules

The trace schema is a **conformance surface**.

Changes to this schema require:
- explicit versioning,
- regeneration of golden traces,
- and a documented rationale.

Silent or accidental changes are treated as regressions.

───────────────────────────────────────────────────────────────────────────────

6. Non-Goals

This schema does NOT:
- define the language semantics,
- specify compiler behavior,
- encode performance characteristics,
- or model physical execution fidelity.

Its sole purpose is deterministic verification.