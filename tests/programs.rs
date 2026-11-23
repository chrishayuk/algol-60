//! Every program in examples/ is run, with examples/<name>.in as its input
//! if there is one, and its output compared with examples/<name>.out; and
//! run again as the X1 system would have run it, compared with
//! examples/<name>.x1.out. A program that ends in an error has the message
//! as a last line beginning "! ". Run with ALGOL_BLESS=1 to rewrite the
//! expected files.

use std::path::Path;

use algol60::{Dialect, Options};

fn transcript(src: &str, input: &str, dialect: Dialect) -> String {
    let options = Options { max_steps: Some(100_000_000), dialect };
    let (mut out, result) = algol60::run_source_with(src, input, options);
    if let Err(e) = result {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!("! {}\n", e));
    }
    out
}

#[test]
fn example_programs() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let bless = std::env::var("ALGOL_BLESS").is_ok();
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "alg"))
        .collect();
    names.sort();
    assert!(names.len() >= 15, "example programs are missing");
    let mut failed = Vec::new();
    for path in names {
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let src = std::fs::read_to_string(&path).unwrap();
        let input = std::fs::read_to_string(dir.join(format!("{}.in", name))).unwrap_or_default();
        for (dialect, suffix) in [(Dialect::Report, "out"), (Dialect::X1, "x1.out")] {
            let got = transcript(&src, &input, dialect);
            let want_path = dir.join(format!("{}.{}", name, suffix));
            if bless {
                std::fs::write(&want_path, &got).unwrap();
                continue;
            }
            let want = std::fs::read_to_string(&want_path).unwrap_or_default();
            if got != want {
                eprintln!("---- {}.{} ----\nexpected:\n{}\ngot:\n{}", name, suffix, want, got);
                failed.push(format!("{}.{}", name, suffix));
            }
        }
    }
    assert!(failed.is_empty(), "programs with unexpected output: {:?}", failed);
}
