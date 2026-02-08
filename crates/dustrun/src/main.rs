// File: crates/dustrun/src/main.rs

mod args;

use args::{Args, EffectModeArg};
use clap::Parser;
use dust_dvm::{Dvm, DvmConfig, DvmTrace, EffectMode};
use std::fs;

fn main() {
    // Deterministic logging initialization:
    // - respects RUST_LOG if set
    // - otherwise defaults to info when --trace is enabled, warn otherwise
    init_logging();

    let args = Args::parse();

    let bytes = match fs::read(&args.dir_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("dustrun: failed to read DIR file '{}': {e}", args.dir_path);
            std::process::exit(2);
        }
    };

    let effect_mode = match args.effects {
        EffectModeArg::Simulate => EffectMode::Simulate,
        EffectModeArg::Realize => EffectMode::Realize,
    };

    let cfg = DvmConfig {
        effect_mode,
        trace: args.trace,
    };

    let dvm = Dvm::new(cfg);

    let program = match dvm.load_dir_json(&bytes) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dustrun: DIR load error: {e}");
            std::process::exit(3);
        }
    };

    let outcome = match dvm.run_entrypoint(&program, &args.entry) {
        Ok(o) => o,
        Err(e) => {
            // Inadmissibility is a first-class outcome, but it is still a failure to execute.
            // Exit code reflects semantic failure vs IO failure.
            if !args.quiet {
                eprintln!("dustrun: {e}");
            }
            // 10-series codes are semantic failures (inadmissible / time / effect / runtime)
            std::process::exit(10);
        }
    };

    if args.emit_trace {
        let trace: DvmTrace = DvmTrace::Success(outcome.into());
        match serde_json::to_string_pretty(&trace) {
            Ok(s) => {
                println!("{s}");
            }
            Err(e) => {
                eprintln!("dustrun: failed to serialize trace: {e}");
                std::process::exit(4);
            }
        }
        return;
    }

    // Human-readable deterministic output.
    if !args.quiet {
        if let Some(ret) = outcome.returned {
            println!("return: {}", format_value(&ret));
        } else {
            println!("return: <none>");
        }

        if outcome.effects.events.is_empty() {
            println!("effects: <none>");
        } else {
            println!("effects:");
            for (i, ev) in outcome.effects.events.iter().enumerate() {
                println!("  {}. {} {}", i + 1, ev.kind, ev.payload);
            }
        }

        println!("time.ticks: {}", outcome.time.tick.0);
        println!("effect_mode: {}", args.effects.as_str());
        println!("entry: {}", args.entry);
    }
}

fn init_logging() {
    // env_logger is deterministic given fixed inputs; we avoid timestamps by default.
    // Users can still opt-in via RUST_LOG and env_logger formatting, but default is stable.
    let mut builder = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"));

    // Remove timestamps for deterministic output
    builder.format(|buf, record| {
        use std::io::Write;
        writeln!(buf, "[{}] {}", record.level(), record.args())
    });

    let _ = builder.try_init();
}

fn format_value(v: &dust_dvm::Value) -> String {
    match v {
        dust_dvm::Value::Int(n) => n.to_string(),
        dust_dvm::Value::Bool(b) => b.to_string(),
        dust_dvm::Value::String(s) => format!("{:?}", s),
        dust_dvm::Value::Struct { ty, fields } => {
            let mut parts = Vec::new();
            for (k, vv) in fields.iter() {
                parts.push(format!("{k}:{}", format_value(vv)));
            }
            format!("{ty}{{{}}}", parts.join(","))
        }
        dust_dvm::Value::Unit => "unit".into(),
    }
}