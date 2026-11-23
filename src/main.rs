use std::io::{BufReader, BufWriter, Read};
use std::process::ExitCode;

const USAGE: &str = "\
Usage: algol60 [OPTIONS] <FILE>
       algol60 [OPTIONS] -e <PROGRAM>
       algol60 [OPTIONS] -              (the program is read from standard input)
       algol60 --check <FILE>...

Options:
  -e, --eval <PROGRAM>  Run the program text given on the command line
      --x1              Run it as the Dijkstra–Zonneveld system for the
                        Electrologica X1 would have, as far as the documents
                        say (docs/x1.md)
      --max-steps <N>   Stop with an error after N steps
      --check           Report what is wrong with each program without running it
  -h, --help            Show this help
  -V, --version         Show the version";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut max_steps = None;
    let mut check = false;
    let mut dialect = algol60::Dialect::Report;
    let mut eval = None;
    let mut files = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                println!("{}", USAGE);
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("algol60 {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "--check" => check = true,
            "--x1" => dialect = algol60::Dialect::X1,
            "-e" | "--eval" => {
                i += 1;
                eval = args.get(i).cloned();
                if eval.is_none() {
                    eprintln!("algol60: -e needs a program\n\n{}", USAGE);
                    return ExitCode::from(2);
                }
            }
            "--max-steps" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse().ok()) {
                    Some(n) => max_steps = Some(n),
                    None => {
                        eprintln!("algol60: --max-steps needs a number");
                        return ExitCode::from(2);
                    }
                }
            }
            s if s.starts_with('-') && s != "-" => {
                eprintln!("algol60: unknown option {}\n\n{}", s, USAGE);
                return ExitCode::from(2);
            }
            s => files.push(s.to_string()),
        }
        i += 1;
    }

    let read = |name: &str| -> Result<String, String> {
        if name == "-" {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).map_err(|e| format!("-: {}", e))?;
            Ok(s)
        } else {
            std::fs::read_to_string(name).map_err(|e| format!("{}: {}", name, e))
        }
    };

    if check {
        let mut ok = true;
        for f in &files {
            match read(f).and_then(|src| algol60::check(&src, dialect).map_err(|e| format!("{}: {}", f, e))) {
                Ok(warnings) => {
                    for w in warnings {
                        eprintln!("{}: warning: {}", f, w);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    ok = false;
                }
            }
        }
        return if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }

    let (name, src) = match (eval, files.as_slice()) {
        (Some(p), []) => ("-e".to_string(), p),
        (None, [f]) => match read(f) {
            Ok(s) => (f.clone(), s),
            Err(e) => {
                eprintln!("algol60: {}", e);
                return ExitCode::FAILURE;
            }
        },
        _ => {
            eprintln!("{}", USAGE);
            return ExitCode::from(2);
        }
    };

    let mut input = BufReader::new(std::io::stdin());
    let mut out = BufWriter::new(std::io::stdout());
    let mut warn = std::io::stderr();
    let r = algol60::run(&src, &mut input, &mut out, &mut warn, algol60::Options { max_steps, dialect });
    drop(out);
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", name, e);
            ExitCode::FAILURE
        }
    }
}
