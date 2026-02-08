//! Dust Virtual Machine (DVM) — reference executor for DPL.
//!
//! This crate implements the normative execution semantics for DIR artifacts:
//! - K-regime: deterministic classical execution (reference semantics)
//! - Q-regime: linear resource semantics enforcement (host-mode semantics)
//! - Φ-regime: validation + deterministic refusal (v0.1), with witness stub wiring
//!
//! This crate contains NO compiler logic and NO CLI logic.
//! It consumes DIR and produces execution traces or refusal/failure traces.

// use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error, Clone, PartialEq, Eq)]
    pub enum DvmError {
        #[error("DIR load error: {0}")]
        DirLoad(String),

        #[error("DIR validation error: {0}")]
        DirValidate(String),

        #[error("entrypoint not found: {0}")]
        EntrypointNotFound(String),

        #[error("regime not supported by this execution path: {0}")]
        UnsupportedRegime(String),

        #[error("inadmissible program: {0}")]
        Inadmissible(String),

        #[error("constraint failure: {0}")]
        ConstraintFailure(String),

        #[error("effect violation: {0}")]
        EffectViolation(String),

        #[error("time violation: {0}")]
        TimeViolation(String),

        #[error("runtime error: {0}")]
        Runtime(String),
    }
}

pub use error::DvmError;

pub mod dir {
    //! Minimal DIR model as currently produced by the Dust compiler.
    //! This mirrors the `dust_dir` crate structures found in the compiler repository.
    //!
    //! NOTE: In this repository, DIR types are defined locally to keep dustrun standalone.

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirProgram {
        pub forges: Vec<DirForge>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirForge {
        pub name: String,
        pub shapes: Vec<DirShape>,
        pub procs: Vec<DirProc>,
        pub binds: Vec<DirBind>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirShape {
        pub name: String,
        pub fields: Vec<DirField>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirField {
        pub name: String,
        pub ty: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirProc {
        pub regime: String, // "K" | "Q" | "Φ"
        pub name: String,
        pub params: Vec<DirParam>,
        pub uses: Vec<DirUses>,
        pub ret: Option<String>,
        pub qualifiers: Vec<String>,
        pub body: Vec<DirStmt>, // v0.1 lowered statements
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirParam {
        pub name: String,
        pub ty: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirUses {
        pub resource: String,
        pub args: Vec<(String, DirLit)>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DirLit {
        Int(i64),
        Bool(bool),
        String(String),
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DirStmt {
        Let { name: String, expr: String },
        Constrain { predicate: String },
        Prove { name: String, from: String },
        Effect { kind: String, payload: String },
        Return { expr: String },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirBind {
        pub source: String,
        pub target: String,
        pub contract: Vec<DirClause>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DirClause {
        pub key: String,
        pub op: String,
        pub value: String,
    }
}

pub use dir::*;

pub mod effects {
    //! Effect model for DVM execution.
    //!
    //! `simulate`: effects are recorded, not enacted.
    //! `realize`: effects may be enacted via pluggable realizers (not yet in v0.1).

    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EffectMode {
        Simulate,
        Realize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct EffectEvent {
        pub kind: String,    // "observe" | "emit" | "seal" (v0.1)
        pub payload: String, // rendered payload expression
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct EffectLog {
        pub events: Vec<EffectEvent>,
    }

    impl EffectLog {
        pub fn push(&mut self, kind: impl Into<String>, payload: impl Into<String>) {
            self.events.push(EffectEvent {
                kind: kind.into(),
                payload: payload.into(),
            });
        }
    }
}

pub use effects::*;

pub mod time {
    //! Time model (v0.1): deterministic logical time.
    //! This will expand to match the spec's full time domains.

    use serde::{Deserialize, Serialize};

    /// A deterministic logical tick counter.
    ///
    /// Serialized as a plain integer for stable conformance traces.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct LogicalTick(pub u64);

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TimeState {
        pub tick: LogicalTick,
    }

    impl Default for TimeState {
        fn default() -> Self {
            Self {
                tick: LogicalTick(0),
            }
        }
    }

    impl TimeState {
        pub fn step(&mut self) {
            self.tick.0 = self.tick.0.saturating_add(1);
        }
    }
}

pub use time::*;

pub mod value {
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum Value {
        Int(i64),
        Bool(bool),
        String(String),
        Struct {
            ty: String,
            fields: IndexMap<String, Value>,
        },
        Unit,
    }

    impl Value {
        pub fn as_bool(&self) -> Option<bool> {
            match self {
                Value::Bool(b) => Some(*b),
                _ => None,
            }
        }
        pub fn as_int(&self) -> Option<i64> {
            match self {
                Value::Int(n) => Some(*n),
                _ => None,
            }
        }
    }
}

pub use value::Value;

pub mod expr {
    //! Minimal expression parser for v0.1 DIR strings.
    //!
    //! Operators are emitted as identifiers: Add, Sub, Mul, Div, Eq, Lt, Le, Gt, Ge, And, Or

    use super::{DvmError, Value};
    use indexmap::IndexMap;

    #[derive(Debug, Clone, PartialEq)]
    enum Tok {
        Ident(String),
        Int(i64),
        Bool(bool),
        Str(String),
        LParen,
        RParen,
        Comma,
        Dot,
        LBrace,
        RBrace,
        Colon,
        Eof,
    }

    fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || c == 'Φ'
    }

    fn is_ident_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == 'Φ'
    }

    fn lex(input: &str) -> Result<Vec<Tok>, DvmError> {
        let mut out = Vec::new();
        let mut chars = input.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            match c {
                '(' => {
                    chars.next();
                    out.push(Tok::LParen);
                }
                ')' => {
                    chars.next();
                    out.push(Tok::RParen);
                }
                ',' => {
                    chars.next();
                    out.push(Tok::Comma);
                }
                '.' => {
                    chars.next();
                    out.push(Tok::Dot);
                }
                '{' => {
                    chars.next();
                    out.push(Tok::LBrace);
                }
                '}' => {
                    chars.next();
                    out.push(Tok::RBrace);
                }
                ':' => {
                    chars.next();
                    out.push(Tok::Colon);
                }
                '"' => {
                    chars.next(); // consume opening "
                    let mut s = String::new();
                    while let Some(ch) = chars.next() {
                        match ch {
                            '"' => break,
                            '\\' => {
                                let esc = chars.next().ok_or_else(|| {
                                    DvmError::Runtime("unterminated string escape".into())
                                })?;
                                match esc {
                                    '"' => s.push('"'),
                                    '\\' => s.push('\\'),
                                    'n' => s.push('\n'),
                                    't' => s.push('\t'),
                                    'r' => s.push('\r'),
                                    other => {
                                        return Err(DvmError::Runtime(format!(
                                            "unsupported string escape: \\{other}"
                                        )));
                                    }
                                }
                            }
                            other => s.push(other),
                        }
                    }
                    out.push(Tok::Str(s));
                }
                '-' | '0'..='9' => {
                    let mut buf = String::new();
                    if c == '-' {
                        buf.push('-');
                        chars.next();
                    }
                    while let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            buf.push(d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    let n: i64 = buf
                        .parse()
                        .map_err(|_| DvmError::Runtime(format!("invalid int literal: {buf}")))?;
                    out.push(Tok::Int(n));
                }
                _ if is_ident_start(c) => {
                    let mut id = String::new();
                    while let Some(&d) = chars.peek() {
                        if is_ident_char(d) {
                            id.push(d);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    match id.as_str() {
                        "true" => out.push(Tok::Bool(true)),
                        "false" => out.push(Tok::Bool(false)),
                        _ => out.push(Tok::Ident(id)),
                    }
                }
                _ => {
                    return Err(DvmError::Runtime(format!(
                        "unexpected character in expression: {c}"
                    )));
                }
            }
        }

        out.push(Tok::Eof);
        Ok(out)
    }

    #[derive(Debug, Clone)]
    struct Parser {
        toks: Vec<Tok>,
        i: usize,
    }

    impl Parser {
        fn new(toks: Vec<Tok>) -> Self {
            Self { toks, i: 0 }
        }
        fn peek(&self) -> &Tok {
            self.toks.get(self.i).unwrap_or(&Tok::Eof)
        }
        fn next(&mut self) -> Tok {
            let t = self.peek().clone();
            self.i = self.i.saturating_add(1);
            t
        }
        fn eat(&mut self, expected: Tok) -> Result<(), DvmError> {
            let got = self.next();
            if got == expected {
                Ok(())
            } else {
                Err(DvmError::Runtime(format!(
                    "expected {:?}, got {:?}",
                    expected, got
                )))
            }
        }
    }

    // Precedence: Mul/Div > Add/Sub > comparisons > And > Or
    pub fn eval(expr: &str, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let toks = lex(expr)?;
        let mut p = Parser::new(toks);
        let v = parse_or(&mut p, env)?;
        Ok(v)
    }

    fn parse_or(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let mut left = parse_and(p, env)?;
        loop {
            if matches!(p.peek(), Tok::Ident(op) if op == "Or") {
                p.next();
                let right = parse_and(p, env)?;
                let lb = left
                    .as_bool()
                    .ok_or_else(|| DvmError::Runtime("Or requires bool operands".into()))?;
                let rb = right
                    .as_bool()
                    .ok_or_else(|| DvmError::Runtime("Or requires bool operands".into()))?;
                left = Value::Bool(lb || rb);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let mut left = parse_cmp(p, env)?;
        loop {
            if matches!(p.peek(), Tok::Ident(op) if op == "And") {
                p.next();
                let right = parse_cmp(p, env)?;
                let lb = left
                    .as_bool()
                    .ok_or_else(|| DvmError::Runtime("And requires bool operands".into()))?;
                let rb = right
                    .as_bool()
                    .ok_or_else(|| DvmError::Runtime("And requires bool operands".into()))?;
                left = Value::Bool(lb && rb);
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_cmp(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let mut left = parse_add(p, env)?;
        loop {
            let op = match p.peek() {
                Tok::Ident(s) if ["Eq", "Lt", "Le", "Gt", "Ge"].contains(&s.as_str()) => s.clone(),
                _ => break,
            };
            p.next();
            let right = parse_add(p, env)?;
            left = match op.as_str() {
                "Eq" => Value::Bool(left == right),
                "Lt" => Value::Bool(cmp_int(&left, &right, |a, b| a < b)?),
                "Le" => Value::Bool(cmp_int(&left, &right, |a, b| a <= b)?),
                "Gt" => Value::Bool(cmp_int(&left, &right, |a, b| a > b)?),
                "Ge" => Value::Bool(cmp_int(&left, &right, |a, b| a >= b)?),
                _ => return Err(DvmError::Runtime(format!("unknown comparison op: {op}"))),
            };
        }
        Ok(left)
    }

    fn cmp_int<F: FnOnce(i64, i64) -> bool>(l: &Value, r: &Value, f: F) -> Result<bool, DvmError> {
        let a = l
            .as_int()
            .ok_or_else(|| DvmError::Runtime("comparison requires int operands".into()))?;
        let b = r
            .as_int()
            .ok_or_else(|| DvmError::Runtime("comparison requires int operands".into()))?;
        Ok(f(a, b))
    }

    fn parse_add(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let mut left = parse_mul(p, env)?;
        loop {
            let op = match p.peek() {
                Tok::Ident(s) if s == "Add" || s == "Sub" => s.clone(),
                _ => break,
            };
            p.next();
            let right = parse_mul(p, env)?;
            let a = left
                .as_int()
                .ok_or_else(|| DvmError::Runtime("Add/Sub requires int operands".into()))?;
            let b = right
                .as_int()
                .ok_or_else(|| DvmError::Runtime("Add/Sub requires int operands".into()))?;
            left = if op == "Add" {
                Value::Int(a + b)
            } else {
                Value::Int(a - b)
            };
        }
        Ok(left)
    }

    fn parse_mul(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        let mut left = parse_primary(p, env)?;
        loop {
            let op = match p.peek() {
                Tok::Ident(s) if s == "Mul" || s == "Div" => s.clone(),
                _ => break,
            };
            p.next();
            let right = parse_primary(p, env)?;
            let a = left
                .as_int()
                .ok_or_else(|| DvmError::Runtime("Mul/Div requires int operands".into()))?;
            let b = right
                .as_int()
                .ok_or_else(|| DvmError::Runtime("Mul/Div requires int operands".into()))?;
            if op == "Div" && b == 0 {
                return Err(DvmError::Runtime("division by zero".into()));
            }
            left = if op == "Mul" {
                Value::Int(a * b)
            } else {
                Value::Int(a / b)
            };
        }
        Ok(left)
    }

    fn parse_primary(p: &mut Parser, env: &IndexMap<String, Value>) -> Result<Value, DvmError> {
        match p.next() {
            Tok::Int(n) => Ok(Value::Int(n)),
            Tok::Bool(b) => Ok(Value::Bool(b)),
            Tok::Str(s) => Ok(Value::String(s)),
            Tok::Ident(id) => {
                if let Some(v) = env.get(&id) {
                    Ok(v.clone())
                } else {
                    Err(DvmError::Runtime(format!("unknown identifier: {id}")))
                }
            }
            Tok::LParen => {
                let v = parse_or(p, env)?;
                p.eat(Tok::RParen)?;
                Ok(v)
            }
            other => Err(DvmError::Runtime(format!(
                "unexpected token in expression: {:?}",
                other
            ))),
        }
    }
}

pub mod admissibility {
    //! v0.1 admissibility model:
    //! - Constrain predicates must evaluate to true in evaluation context over classical env.
    //! - Φ-regime host-mode semantics will evolve to match the spec.

    use super::{expr, DvmError, Value};
    use indexmap::IndexMap;

    pub fn check_predicate(predicate: &str, env: &IndexMap<String, Value>) -> Result<(), DvmError> {
        let v = expr::eval(predicate, env)?;
        let ok = v.as_bool().ok_or_else(|| {
            DvmError::ConstraintFailure("constraint predicate did not evaluate to bool".into())
        })?;
        if ok {
            Ok(())
        } else {
            Err(DvmError::Inadmissible(format!(
                "constraint failed: {predicate}"
            )))
        }
    }
}

pub mod regime;
pub use regime::*;

pub mod engine {
    use super::{
        admissibility,
        dir::DirStmt,
        effects::EffectLog,
        effects::EffectMode,
        expr,
        regime::{
            phi_refuse_execution, phi_validate_proc, PhiValidation, PhiWitnessBuilder, QState,
        },
        time::TimeState,
        DirProc, DirProgram, DvmError, Value,
    };
    use indexmap::IndexMap;

    #[derive(Debug, Clone)]
    pub struct DvmConfig {
        pub effect_mode: EffectMode,
        pub trace: bool,
    }

    impl Default for DvmConfig {
        fn default() -> Self {
            Self {
                effect_mode: EffectMode::Simulate,
                trace: false,
            }
        }
    }

    /// Successful outcome (no refusal/failure).
    #[derive(Debug, Clone)]
    pub struct DvmOutcome {
        pub returned: Option<Value>,
        pub effects: EffectLog,
        pub time: TimeState,
    }

    /// Fault/refusal with deterministic partial context.
    #[derive(Debug, Clone)]
    pub struct DvmFault {
        pub error: DvmError,
        pub effects: EffectLog,
        pub time: TimeState,
    }

    impl DvmFault {
        pub fn new(error: DvmError, effects: EffectLog, time: TimeState) -> Self {
            Self {
                error,
                effects,
                time,
            }
        }
    }

    impl From<DvmError> for DvmFault {
        fn from(error: DvmError) -> Self {
            DvmFault::new(error, EffectLog::default(), TimeState::default())
        }
    }

    #[derive(Debug)]
    pub struct Dvm {
        cfg: DvmConfig,
    }

    impl Dvm {
        pub fn new(cfg: DvmConfig) -> Self {
            Self { cfg }
        }

        /// Load a DIR program from JSON bytes.
        pub fn load_dir_json(&self, bytes: &[u8]) -> Result<DirProgram, DvmError> {
            serde_json::from_slice::<DirProgram>(bytes)
                .map_err(|e| DvmError::DirLoad(format!("{e}")))
        }

        /// Validate basic DIR structure (v0.1).
        pub fn validate_dir(&self, program: &DirProgram) -> Result<(), DvmError> {
            if program.forges.is_empty() {
                return Err(DvmError::DirValidate("program has no forges".into()));
            }
            for forge in &program.forges {
                if forge.name.trim().is_empty() {
                    return Err(DvmError::DirValidate("forge name is empty".into()));
                }
                for proc_ in &forge.procs {
                    if proc_.name.trim().is_empty() {
                        return Err(DvmError::DirValidate("proc name is empty".into()));
                    }
                    if proc_.regime != "K" && proc_.regime != "Q" && proc_.regime != "Φ" {
                        return Err(DvmError::DirValidate(format!(
                            "unknown regime: {}",
                            proc_.regime
                        )));
                    }
                }
            }
            Ok(())
        }

        /// Compatibility API: prior callers expect `Result<Outcome, DvmError>`.
        ///
        /// This now drops partial context on failure. Prefer `run_entrypoint_trace` in new code.
        pub fn run_entrypoint(
            &self,
            program: &DirProgram,
            entry: &str,
        ) -> Result<DvmOutcome, DvmError> {
            match self.run_entrypoint_with_fault(program, entry) {
                Ok(ok) => Ok(ok),
                Err(fault) => Err(fault.error),
            }
        }

        /// New API: returns a structured fault carrying deterministic partial context.
        pub fn run_entrypoint_with_fault(
            &self,
            program: &DirProgram,
            entry: &str,
        ) -> Result<DvmOutcome, DvmFault> {
            // validation failures have no prior context
            self.validate_dir(program)
                .map_err(|e| DvmFault::new(e, EffectLog::default(), TimeState::default()))?;

            let proc_ = find_proc(program, entry).ok_or_else(|| {
                DvmFault::new(
                    DvmError::EntrypointNotFound(entry.to_string()),
                    EffectLog::default(),
                    TimeState::default(),
                )
            })?;

            let mut env = IndexMap::<String, Value>::new();
            if !proc_.params.is_empty() {
                return Err(DvmFault::new(
                    DvmError::Runtime(format!(
                        "entrypoint has params in v0.1 host-runner: {}:{}",
                        proc_.name, proc_.regime
                    )),
                    EffectLog::default(),
                    TimeState::default(),
                ));
            }

            match proc_.regime.as_str() {
                "K" => self.exec_k(proc_, &mut env),
                "Q" => self.exec_q(proc_, &mut env),
                "Φ" => self.exec_phi(proc_, &mut env),
                other => Err(DvmFault::new(
                    DvmError::UnsupportedRegime(format!("unknown regime: {other}")),
                    EffectLog::default(),
                    TimeState::default(),
                )),
            }
        }

        // Trace API: produce a single trace value for conformance and tooling.
        pub fn run_entrypoint_trace(&self, program: &DirProgram, entry: &str) -> crate::DvmTrace {
            match self.run_entrypoint_with_fault(program, entry) {
                Ok(ok) => crate::DvmTrace::Success(ok.into()),
                Err(fault) => {
                    let effects = if fault.effects.events.is_empty() {
                        None
                    } else {
                        Some(fault.effects)
                    };

                    let time = if fault.time.tick.0 == 0 {
                        None
                    } else {
                        Some(fault.time)
                    };

                    crate::DvmTrace::Failure(crate::DvmFailureTrace {
                        error: crate::TraceError::from(&fault.error),
                        effects,
                        time,
                    })
                }
            }
        }

        fn exec_k(
            &self,
            proc_: &DirProc,
            env: &mut IndexMap<String, Value>,
        ) -> Result<DvmOutcome, DvmFault> {
            let mut effects = EffectLog::default();
            let mut time = TimeState::default();

            for stmt in &proc_.body {
                if self.cfg.trace {
                    log::info!("tick={} stmt={:?}", time.tick.0, stmt);
                }

                let step_res: Result<Option<Value>, DvmError> = match stmt {
                    DirStmt::Let { name, expr: e } => {
                        let v = expr::eval(e, env)?;
                        env.insert(name.clone(), v);
                        Ok(None)
                    }
                    DirStmt::Constrain { predicate } => {
                        admissibility::check_predicate(predicate, env)?;
                        Ok(None)
                    }
                    DirStmt::Prove { name, from } => {
                        admissibility::check_predicate(from, env)?;
                        env.insert(name.clone(), Value::Unit);
                        Ok(None)
                    }
                    DirStmt::Effect { kind, payload } => {
                        let rendered = render_payload(payload, env)?;
                        effects.push(kind.clone(), rendered);
                        match self.cfg.effect_mode {
                            EffectMode::Simulate => {}
                            EffectMode::Realize => {}
                        }
                        Ok(None)
                    }
                    DirStmt::Return { expr: e } => {
                        let v = expr::eval(e, env)?;
                        Ok(Some(v))
                    }
                };

                match step_res {
                    Ok(Some(v)) => {
                        return Ok(DvmOutcome {
                            returned: Some(v),
                            effects,
                            time,
                        });
                    }
                    Ok(None) => {
                        time.step();
                    }
                    Err(e) => {
                        return Err(DvmFault::new(e, effects, time));
                    }
                }
            }

            Ok(DvmOutcome {
                returned: None,
                effects,
                time,
            })
        }

        fn exec_q(
            &self,
            proc_: &DirProc,
            env: &mut IndexMap<String, Value>,
        ) -> Result<DvmOutcome, DvmFault> {
            let mut effects = EffectLog::default();
            let mut time = TimeState::default();
            let mut q = QState::new();

            for stmt in &proc_.body {
                if self.cfg.trace {
                    log::info!("tick={} stmt={:?}", time.tick.0, stmt);
                }

                let step_res: Result<Option<Value>, DvmError> = match stmt {
                    DirStmt::Let { name, expr: e } => {
                        if let Some(ty) = parse_q_alloc(e) {
                            q.alloc(name, &ty)?;
                            env.insert(name.clone(), Value::Unit);
                            Ok(None)
                        } else if let Some(src) = parse_q_move(e) {
                            q.mov(&src, name)?;
                            env.insert(name.clone(), Value::Unit);
                            Ok(None)
                        } else if let Some(src) = parse_q_use(e) {
                            let _ = q.require_usable(&src, "q_use")?;
                            env.insert(name.clone(), Value::Unit);
                            Ok(None)
                        } else if let Some(src) = parse_q_consume(e) {
                            q.consume(&src, "q_consume")?;
                            env.insert(name.clone(), Value::Unit);
                            Ok(None)
                        } else {
                            let v = expr::eval(e, env)?;
                            env.insert(name.clone(), v);
                            Ok(None)
                        }
                    }
                    DirStmt::Constrain { predicate } => {
                        admissibility::check_predicate(predicate, env)?;
                        Ok(None)
                    }
                    DirStmt::Prove { name, from } => {
                        admissibility::check_predicate(from, env)?;
                        env.insert(name.clone(), Value::Unit);
                        Ok(None)
                    }
                    DirStmt::Effect { kind, payload } => {
                        let rendered = render_payload(payload, env)?;
                        effects.push(kind.clone(), rendered);
                        match self.cfg.effect_mode {
                            EffectMode::Simulate => {}
                            EffectMode::Realize => {}
                        }
                        Ok(None)
                    }
                    DirStmt::Return { expr: e } => {
                        let v = expr::eval(e, env)?;
                        Ok(Some(v))
                    }
                };

                match step_res {
                    Ok(Some(v)) => {
                        return Ok(DvmOutcome {
                            returned: Some(v),
                            effects,
                            time,
                        });
                    }
                    Ok(None) => {
                        time.step();
                    }
                    Err(e) => {
                        return Err(DvmFault::new(e, effects, time));
                    }
                }
            }

            Ok(DvmOutcome {
                returned: None,
                effects,
                time,
            })
        }

        fn exec_phi(
            &self,
            proc_: &DirProc,
            env: &mut IndexMap<String, Value>,
        ) -> Result<DvmOutcome, DvmFault> {
            // v0.1: validate constraints (local host-mode) then refuse execution deterministically,
            // but allow construction of Φ witness stubs as a host intrinsic.
            match phi_validate_proc(proc_, env) {
                Ok(PhiValidation::LocallyAdmissible) => {}
                Ok(PhiValidation::LocallyInadmissible { message }) => {
                    return Err(DvmFault::new(
                        DvmError::Inadmissible(message),
                        EffectLog::default(),
                        TimeState::default(),
                    ));
                }
                Err(e) => {
                    return Err(DvmFault::new(e, EffectLog::default(), TimeState::default()));
                }
            }

            let mut effects = EffectLog::default();
            let mut time = TimeState::default();
            let mut builder = PhiWitnessBuilder::new();

            for stmt in &proc_.body {
                if self.cfg.trace {
                    log::info!("tick={} stmt={:?}", time.tick.0, stmt);
                }

                let step_res: Result<(), DvmError> = match stmt {
                    DirStmt::Let { name, expr: e } => {
                        if let Some(arg_expr) = parse_phi_witness(e) {
                            // Evaluate the argument expression and require it to be a String.
                            let v = expr::eval(&arg_expr, env)?;
                            let digest = match v {
                                Value::String(s) => s,
                                other => {
                                    return Err(DvmFault::new(
                                        DvmError::Runtime(format!(
                                            "phi_witness expects a String digest, got {:?}",
                                            other
                                        )),
                                        effects,
                                        time,
                                    ));
                                }
                            };

                            let w = builder.admissible(&digest);

                            // Integrate witness as a first-class Value (struct) rather than a JSON string.
                            env.insert(name.clone(), phi_witness_to_value(&w));
                        } else {
                            // v0.1: allow ordinary Let evaluation in host-mode so Φ intrinsics
                            // can consume previously-bound values (e.g., digest strings).
                            let v = expr::eval(e, env)?;
                            env.insert(name.clone(), v);
                        }
                        Ok(())
                    }

                    // --- re
                    DirStmt::Effect { kind, payload } => {
                        let rendered = render_payload(payload, env)?;
                        effects.push(kind.clone(), rendered);
                        Ok(())
                    }
                    DirStmt::Constrain { .. } => Ok(()), // already validated
                    DirStmt::Prove { name, from } => {
                        // Require predicate to hold in host-mode.
                        admissibility::check_predicate(from, env)?;

                        // Deterministic v0.1 digest of the proved predicate.
                        let digest = format!("pred:{from}");

                        // Produce a witness stub and inject as a first-class Struct Value.
                        let w = builder.admissible(&digest);
                        env.insert(name.clone(), phi_witness_to_value(&w));

                        Ok(())
                    }
                    DirStmt::Return { .. } => Ok(()), // ignored in v0.1
                };

                if let Err(e) = step_res {
                    return Err(DvmFault::new(e, effects, time));
                }

                time.step();
            }

            // Refuse execution but carry partial context.
            Err(DvmFault::new(phi_refuse_execution(), effects, time))
        }
    }

    fn find_proc<'a>(program: &'a DirProgram, name: &str) -> Option<&'a DirProc> {
        for forge in &program.forges {
            for p in &forge.procs {
                if p.name == name {
                    return Some(p);
                }
            }
        }
        None
    }

    fn render_payload(
        payload_expr: &str,
        env: &IndexMap<String, Value>,
    ) -> Result<String, DvmError> {
        let v = expr::eval(payload_expr, env)?;
        Ok(match v {
            Value::String(s) => s,
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Struct { .. } => serde_json::to_string(&v).map_err(|e| {
                DvmError::Runtime(format!("failed to render struct payload as json: {e}"))
            })?,
            Value::Unit => "unit".into(),
        })
    }

    #[allow(dead_code)]
    fn value_to_string(v: &Value) -> String {
        match v {
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => format!("{:?}", s),
            Value::Struct { ty, fields } => {
                let mut parts = Vec::new();
                for (k, vv) in fields.iter() {
                    parts.push(format!("{k}:{}", value_to_string(vv)));
                }
                format!("{ty}{{{}}}", parts.join(","))
            }
            Value::Unit => "unit".into(),
        }
    }

    fn parse_call_1(expr: &str, name: &str) -> Option<String> {
        let s = expr.trim();
        let prefix = format!("{name}(");
        if !s.starts_with(&prefix) || !s.ends_with(')') {
            return None;
        }
        let inner = &s[prefix.len()..s.len() - 1];
        Some(inner.trim().to_string())
    }

    fn parse_q_alloc(expr: &str) -> Option<String> {
        parse_call_1(expr, "q_alloc").filter(|s| !s.is_empty())
    }

    fn parse_q_move(expr: &str) -> Option<String> {
        parse_call_1(expr, "q_move").filter(|s| !s.is_empty())
    }

    fn parse_q_use(expr: &str) -> Option<String> {
        parse_call_1(expr, "q_use").filter(|s| !s.is_empty())
    }

    fn parse_q_consume(expr: &str) -> Option<String> {
        parse_call_1(expr, "q_consume").filter(|s| !s.is_empty())
    }

    fn parse_phi_witness(expr: &str) -> Option<String> {
        // Accept a single-argument call: phi_witness(<arg_expr>)
        // Return the raw argument expression (not evaluated here).
        parse_call_1(expr, "phi_witness").filter(|s| !s.is_empty())
    }

    fn phi_witness_to_value(w: &crate::regime::PhiWitness) -> Value {
        use crate::regime::PhiWitnessKind;

        let mut fields = IndexMap::new();

        let kind_str = match w.kind {
            PhiWitnessKind::Admissible => "Admissible",
            PhiWitnessKind::NonExistent => "NonExistent",
        };

        fields.insert("kind".to_string(), Value::String(kind_str.to_string()));
        fields.insert("id".to_string(), Value::String(w.id.clone()));
        fields.insert(
            "constraint_digest".to_string(),
            Value::String(w.constraint_digest.clone()),
        );
        fields.insert("note".to_string(), Value::String(w.note.clone()));

        Value::Struct {
            ty: "PhiWitness".to_string(),
            fields,
        }
    }

    // fn phi_digest_of_predicate(pred: &str) -> String {
        // v0.1 digest is a stable textual encoding.
        // Future versions can switch to canonical AST hashing with versioning.
        // format!("pred:{pred}")
    // }
}

pub use engine::{Dvm, DvmConfig, DvmFault, DvmOutcome};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceError {
    pub kind: String,
    pub message: String,
}

impl From<&DvmError> for TraceError {
    fn from(e: &DvmError) -> Self {
        let (kind, message) = match e {
            DvmError::DirLoad(s) => ("DirLoad", s.clone()),
            DvmError::DirValidate(s) => ("DirValidate", s.clone()),
            DvmError::EntrypointNotFound(s) => ("EntrypointNotFound", s.clone()),
            DvmError::UnsupportedRegime(s) => ("UnsupportedRegime", s.clone()),
            DvmError::Inadmissible(s) => ("Inadmissible", s.clone()),
            DvmError::ConstraintFailure(s) => ("ConstraintFailure", s.clone()),
            DvmError::EffectViolation(s) => ("EffectViolation", s.clone()),
            DvmError::TimeViolation(s) => ("TimeViolation", s.clone()),
            DvmError::Runtime(s) => ("Runtime", s.clone()),
        };
        Self {
            kind: kind.to_string(),
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DvmSuccessTrace {
    pub returned: Option<Value>,
    pub effects: EffectLog,
    pub time: TimeState,
}

impl From<DvmOutcome> for DvmSuccessTrace {
    fn from(o: DvmOutcome) -> Self {
        Self {
            returned: o.returned,
            effects: o.effects,
            time: o.time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DvmFailureTrace {
    pub error: TraceError,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<EffectLog>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<TimeState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum DvmTrace {
    Failure(DvmFailureTrace),
    Success(DvmSuccessTrace),
}
