//! ALGOL 60, from the *Revised Report on the Algorithmic Language ALGOL 60*
//! (Naur, ed., 1962; published January 1963).

pub mod ast;
pub mod builtins;
pub mod compile;
pub mod interp;
pub mod lexer;
pub mod parser;
pub mod value;

use std::io::{BufRead, Write};

pub use compile::Dialect;

/// How a program is to be run.
#[derive(Clone, Copy, Debug, Default)]
pub struct Options {
    /// Stop with an error after this many steps.
    pub max_steps: Option<u64>,
    pub dialect: Dialect,
}

/// The stack the interpreter runs on. Procedures nest deeply in programs
/// like Knuth's man or boy test, and each level is several Rust frames.
const STACK: usize = 4 << 30;

/// Reports what is wrong with a program, without running it, with any
/// warnings.
pub fn check(src: &str, dialect: Dialect) -> Result<Vec<String>, String> {
    compile::compile(src, dialect).map(|p| p.warnings)
}

/// Compiles and runs a program. Warnings are written to `warn`.
pub fn run(
    src: &str,
    input: &mut (dyn BufRead + Send),
    out: &mut (dyn Write + Send),
    warn: &mut (dyn Write + Send),
    options: Options,
) -> Result<(), String> {
    let Options { max_steps, dialect } = options;
    let mut go = move || {
        let prog = compile::compile(src, dialect)?;
        for w in &prog.warnings {
            let _ = writeln!(warn, "warning: {}", w);
        }
        interp::Interp::new(&prog, out, input, max_steps).run()
    };
    // WASI has no threads: the program runs on the main stack, which
    // .cargo/config.toml makes 64 MB.
    if cfg!(target_family = "wasm") {
        return go();
    }
    std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(s, go)
            .expect("cannot start the interpreter thread")
            .join()
            .unwrap_or_else(|_| Err("the interpreter failed".into()))
    })
}

/// Runs a program on the given input, returning its output and how it
/// ended. Warnings are discarded.
pub fn run_source(src: &str, input: &str, max_steps: Option<u64>) -> (String, Result<(), String>) {
    run_source_with(src, input, Options { max_steps, dialect: Dialect::Report })
}

/// As `run_source`, in either dialect.
pub fn run_source_with(src: &str, input: &str, options: Options) -> (String, Result<(), String>) {
    let mut out = Vec::new();
    let mut inp = input.as_bytes();
    let mut warn = std::io::sink();
    let r = run(src, &mut inp, &mut out, &mut warn, options);
    (String::from_utf8_lossy(&out).into_owned(), r)
}
