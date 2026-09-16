//! `ktav-fmt` — a comment-preserving formatter CLI for Ktav documents
//! (issue rust#13). Three flags only, hand-rolled: this repository keeps
//! a deliberately lean runtime dependency tree (`serde_json` is dev-only
//! and stays that way), and a CLI argument parser is the one place a
//! new dependency might have been defensible for a richer surface — it
//! isn't here, since there are only three flags to parse.
//!
//! ```text
//! ktav-fmt <file>...        format each file in place
//! ktav-fmt --stdout <file>  print the formatted result, don't touch the file
//! ktav-fmt --check <file>...   exit non-zero if any file isn't already
//!                               formatted; nothing is written
//! ktav-fmt -                 read one document from stdin, write to stdout
//! ```

use std::fs;
use std::io::{self, Read, Write};
use std::process::ExitCode;

enum Mode {
    InPlace,
    Stdout,
    Check,
}

fn main() -> ExitCode {
    let mut mode = Mode::InPlace;
    let mut paths: Vec<String> = Vec::new();
    let mut saw_flag_conflict = false;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--check" => {
                if matches!(mode, Mode::Stdout) {
                    saw_flag_conflict = true;
                }
                mode = Mode::Check;
            }
            "--stdout" => {
                if matches!(mode, Mode::Check) {
                    saw_flag_conflict = true;
                }
                mode = Mode::Stdout;
            }
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            "-" => paths.push(arg),
            other if other.starts_with('-') && other.len() > 1 => {
                eprintln!("ktav-fmt: unrecognized flag '{other}'");
                print_usage();
                return ExitCode::FAILURE;
            }
            other => paths.push(other.to_string()),
        }
    }

    if saw_flag_conflict {
        eprintln!("ktav-fmt: --check and --stdout are mutually exclusive");
        return ExitCode::FAILURE;
    }

    if paths.is_empty() {
        eprintln!("ktav-fmt: no input files (pass a path, or '-' for stdin)");
        print_usage();
        return ExitCode::FAILURE;
    }

    let mut had_error = false;
    let mut found_unformatted = false;

    for path in &paths {
        let is_stdin = path == "-";

        let original = if is_stdin {
            let mut buf = String::new();
            match io::stdin().read_to_string(&mut buf) {
                Ok(_) => buf,
                Err(e) => {
                    eprintln!("ktav-fmt: reading stdin: {e}");
                    had_error = true;
                    continue;
                }
            }
        } else {
            match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("ktav-fmt: {path}: {e}");
                    had_error = true;
                    continue;
                }
            }
        };

        let formatted = match ktav::format_str(&original) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("ktav-fmt: {path}: {e}");
                had_error = true;
                continue;
            }
        };

        match mode {
            Mode::Check => {
                if formatted != original {
                    println!("{path}");
                    found_unformatted = true;
                }
            }
            Mode::Stdout => {
                if let Err(e) = io::stdout().write_all(formatted.as_bytes()) {
                    eprintln!("ktav-fmt: writing stdout: {e}");
                    had_error = true;
                }
            }
            Mode::InPlace => {
                if is_stdin {
                    if let Err(e) = io::stdout().write_all(formatted.as_bytes()) {
                        eprintln!("ktav-fmt: writing stdout: {e}");
                        had_error = true;
                    }
                } else if formatted != original {
                    if let Err(e) = fs::write(path, &formatted) {
                        eprintln!("ktav-fmt: {path}: {e}");
                        had_error = true;
                    }
                }
            }
        }
    }

    if had_error || (matches!(mode, Mode::Check) && found_unformatted) {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn print_usage() {
    eprintln!("Usage: ktav-fmt [--check | --stdout] <file>...");
    eprintln!("       ktav-fmt -                        (read stdin, write stdout)");
    eprintln!();
    eprintln!("  --check   exit non-zero if any input is not already formatted;");
    eprintln!("            nothing is written — unformatted paths are printed");
    eprintln!("  --stdout  print the formatted result instead of writing in place");
    eprintln!("  -h, --help  show this message");
}
