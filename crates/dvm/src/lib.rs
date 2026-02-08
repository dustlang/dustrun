//! Dust Virtual Machine (DVM) — reference executor for DPL.
//!
//! This crate implements the normative execution semantics for DIR artifacts:
//! - K-regime: deterministic classical execution (reference semantics)
//! - Q-regime: linear resource semantics enforcement (host-mode semantics)
//! - Φ-regime: admissibility resolution + witness handling (host-mode semantics)
//!
//! This crate contains NO compiler logic and NO CLI logic.
//! It consumes DIR and produces execution / non-existence outcomes.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod error {
    use thiserror::Error;

    #[derive(Debug, Error)]
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

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EffectEvent {
        pub kind: String,    // "observe" | "emit" | "seal" (v0.1)
        pub payload: String, // rendered payload expression
    }

    #[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct LogicalTick(pub u64);

    #[derive(Debug, Clone, Serialize, Deserialize)]
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
        Struct { ty: String, fields: IndexMap<String, Value> },
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
                                let esc = chars
                                    .next()
                                    .ok_or_else(|| DvmError::Runtime("unterminated string escape".into()))?;
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
    //! - Constrain predicates must evaluate to true in K/Q evaluation context over classical env.
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
        admissibility, dir::DirStmt, effects::EffectMode, effects::EffectLog, expr,
        regime::QState, time::TimeState, DirProgram, DirProc, DvmError, Value,
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

    #[derive(Debug, Clone)]
    pub struct DvmOutcome {
        pub returned: Option<Value>,
        pub effects: EffectLog,
        pub time: TimeState,
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
            serde_json::from_slice::<DirProgram>(bytes).map_err(|e| DvmError::DirLoad(format!("{e}")))
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
                        return Err(DvmError::DirValidate(format!("unknown regime: {}", proc_.regime)));
                    }
                }
            }
            Ok(())
        }

        /// Execute an entrypoint proc by name.
        pub fn run_entrypoint(&self, program: &DirProgram, entry: &str) -> Result<DvmOutcome, DvmError> {
            self.validate_dir(program)?;

            let proc_ = find_proc(program, entry).ok_or_else(|| DvmError::EntrypointNotFound(entry.to_string()))?;

            let mut env = IndexMap::<String, Value>::new();
            for p in &proc_.params {
                return Err(DvmError::Runtime(format!(
                    "entrypoint has params in v0.1 host-runner: {}:{}",
                    p.name, p.ty
                )));
            }

            match proc_.regime.as_str() {
                "K" => self.exec_k(proc_, &mut env),
                "Q" => self.exec_q(proc_, &mut env),
                "Φ" => Err(DvmError::UnsupportedRegime(
                    "Φ-regime execution wiring into engine is a later step".into(),
                )),
                other => Err(DvmError::UnsupportedRegime(format!("unknown regime: {other}"))),
            }
        }

        fn exec_k(&self, proc_: &DirProc, env: &mut IndexMap<String, Value>) -> Result<DvmOutcome, DvmError> {
            let mut effects = EffectLog::default();
            let mut time = TimeState::default();

            for stmt in &proc_.body {
                if self.cfg.trace {
                    log::info!("tick={} stmt={:?}", time.tick.0, stmt);
                }

                match stmt {
                    DirStmt::Let { name, expr: e } => {
                        let v = expr::eval(e, env)?;
                        env.insert(name.clone(), v);
                    }
                    DirStmt::Constrain { predicate } => {
                        admissibility::check_predicate(predicate, env)?;
                    }
                    DirStmt::Prove { name, from } => {
                        admissibility::check_predicate(from, env)?;
                        env.insert(name.clone(), Value::Unit);
                    }
                    DirStmt::Effect { kind, payload } => {
                        let rendered = render_payload(payload, env)?;
                        effects.push(kind.clone(), rendered);
                        match self.cfg.effect_mode {
                            EffectMode::Simulate => {}
                            EffectMode::Realize => {
                                // v0.1: realization is logging-only unless a realizer is configured (future step).
                            }
                        }
                    }
                    DirStmt::Return { expr: e } => {
                        let v = expr::eval(e, env)?;
                        return Ok(DvmOutcome {
                            returned: Some(v),
                            effects,
                            time,
                        });
                    }
                }

                time.step();
            }

            Ok(DvmOutcome {
                returned: None,
                effects,
                time,
            })
        }

        fn exec_q(&self, proc_: &DirProc, env: &mut IndexMap<String, Value>) -> Result<DvmOutcome, DvmError> {
            let mut effects = EffectLog::default();
            let mut time = TimeState::default();

            // Q-regime host state enforcing linearity.
            let mut q = QState::new();

            for stmt in &proc_.body {
                if self.cfg.trace {
                    log::info!("tick={} stmt={:?}", time.tick.0, stmt);
                }

                match stmt {
                    DirStmt::Let { name, expr: e } => {
                        // Host-mode Q intrinsics are expressed as calls in DIR strings.
                        // This is a deterministic bridge that will be replaced with explicit DIR ops
                        // when the compiler emits typed Q instructions.
                        if let Some(ty) = parse_q_alloc(e) {
                            q.alloc(name, &ty)?;
                            env.insert(name.clone(), Value::Unit);
                        } else if let Some(src) = parse_q_move(e) {
                            q.mov(&src, name)?;
                            env.insert(name.clone(), Value::Unit);
                        } else if let Some(src) = parse_q_use(e) {
                            // Require usable enforces "not moved" + "resource live"
                            let _ = q.require_usable(&src, "q_use")?;
                            env.insert(name.clone(), Value::Unit);
                        } else if let Some(src) = parse_q_consume(e) {
                            q.consume(&src, "q_consume")?;
                            env.insert(name.clone(), Value::Unit);
                        } else {
                            // Allow classical computation inside Q-regime as a constrained subset.
                            // This is needed for indices, sizes, flags, and deterministic orchestration.
                            let v = expr::eval(e, env)?;
                            env.insert(name.clone(), v);
                        }
                    }
                    DirStmt::Constrain { predicate } => {
                        // Constraints are evaluated over the classical env in host-mode.
                        admissibility::check_predicate(predicate, env)?;
                    }
                    DirStmt::Prove { name, from } => {
                        admissibility::check_predicate(from, env)?;
                        env.insert(name.clone(), Value::Unit);
                    }
                    DirStmt::Effect { kind, payload } => {
                        let rendered = render_payload(payload, env)?;
                        effects.push(kind.clone(), rendered);
                        match self.cfg.effect_mode {
                            EffectMode::Simulate => {}
                            EffectMode::Realize => {
                                // v0.1: realization is logging-only unless a realizer is configured (future step).
                            }
                        }
                    }
                    DirStmt::Return { expr: e } => {
                        let v = expr::eval(e, env)?;
                        return Ok(DvmOutcome {
                            returned: Some(v),
                            effects,
                            time,
                        });
                    }
                }

                time.step();
            }

            Ok(DvmOutcome {
                returned: None,
                effects,
                time,
            })
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

    fn render_payload(payload_expr: &str, env: &IndexMap<String, Value>) -> Result<String, DvmError> {
        let v = expr::eval(payload_expr, env)?;
        Ok(match v {
            Value::String(s) => s,
            Value::Int(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Struct { ty, fields } => {
                let mut parts = Vec::new();
                for (k, vv) in fields.iter() {
                    parts.push(format!("{k}:{}", value_to_string(vv)));
                }
                format!("{ty}{{{}}}", parts.join(","))
            }
            Value::Unit => "unit".into(),
        })
    }

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
        // q_alloc(QBit)
        parse_call_1(expr, "q_alloc").filter(|s| !s.is_empty())
    }

    fn parse_q_move(expr: &str) -> Option<String> {
        // q_move(a)
        parse_call_1(expr, "q_move").filter(|s| !s.is_empty())
    }

    fn parse_q_use(expr: &str) -> Option<String> {
        // q_use(a)
        parse_call_1(expr, "q_use").filter(|s| !s.is_empty())
    }

    fn parse_q_consume(expr: &str) -> Option<String> {
        // q_consume(a)
        parse_call_1(expr, "q_consume").filter(|s| !s.is_empty())
    }
}

pub use engine::{Dvm, DvmConfig, DvmOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DvmTrace {
    pub returned: Option<Value>,
    pub effects: EffectLog,
    pub time: TimeState,
}

impl From<DvmOutcome> for DvmTrace {
    fn from(o: DvmOutcome) -> Self {
        Self {
            returned: o.returned,
            effects: o.effects,
            time: o.time,
        }
    }
}