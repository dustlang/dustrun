dustrun/
├── README.md
├── LICENSE
├── Cargo.toml                 # workspace root
├── rust-toolchain.toml        # pin toolchain for deterministic builds
├── .gitignore
├── .github/
│   └── workflows/
│       ├── ci.yml             # build + test + fmt + clippy
│       └── conformance.yml    # optional: spec-driven test matrix
├── crates/
│   ├── dvm/                   # DVM core library (reference semantics live here)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine/        # execution driver (run loop, stepping, replay)
│   │       │   ├── mod.rs
│   │       │   ├── execute.rs
│   │       │   ├── step.rs
│   │       │   └── trace.rs
│   │       ├── dir/           # DIR input handling (parse/load/validate)
│   │       │   ├── mod.rs
│   │       │   ├── format.rs  # format detection + schema versioning
│   │       │   ├── load.rs
│   │       │   └── validate.rs
│   │       ├── regime/        # regime semantics boundaries (K/Q/Φ)
│   │       │   ├── mod.rs
│   │       │   ├── k.rs
│   │       │   ├── q.rs
│   │       │   └── phi.rs
│   │       ├── admissibility/ # admissibility checks + witnesses + explanations
│   │       │   ├── mod.rs
│   │       │   ├── check.rs
│   │       │   ├── witness.rs
│   │       │   └── explain.rs
│   │       ├── effects/       # effect model + simulate/realize boundary
│   │       │   ├── mod.rs
│   │       │   ├── model.rs
│   │       │   ├── simulate.rs
│   │       │   └── realize.rs
│   │       ├── time/          # time domains + deterministic scheduling
│   │       │   ├── mod.rs
│   │       │   ├── clock.rs
│   │       │   └── schedule.rs
│   │       ├── io/            # inputs/outputs, trace sinks, artifact bundles
│   │       │   ├── mod.rs
│   │       │   ├── stdout.rs
│   │       │   └── files.rs
│   │       └── error.rs       # unified error taxonomy
│   ├── dustrun/               # CLI binary crate (produces the `dustrun` executable)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── args.rs
│   └── conformance/           # spec-aligned test harness + fixtures
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── runner.rs
│       │   └── corpus.rs
│       └── tests/
│           └── conformance.rs
├── tests/
│   ├── fixtures/              # small DIR artifacts + expected traces
│   └── golden/                # golden trace outputs for deterministic replay
├── docs/
│   ├── architecture.md        # non-normative: how dustrun maps to DVM
│   └── conformance.md         # non-normative: how tests relate to spec
└── scripts/
    ├── gen-fixtures.sh
    └── run-local.sh