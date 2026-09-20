//! CLI surface tests for the `ktav-fmt` binary (issue rust#13): format
//! in place, format to stdout, and `--check` exiting non-zero on
//! unformatted input with nothing written.
//!
//! The binary is behind the `cli` feature (see Cargo.toml), so this file
//! compiles to nothing without it — `CARGO_BIN_EXE_ktav-fmt` only exists
//! when the bin target is built. CI runs the suite with
//! `--all-features`, so these tests do run on every push; a plain
//! `cargo test` skips them silently, which is the one thing to remember
//! when running the suite by hand.
#![cfg(feature = "cli")]

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_ktav-fmt")
}

fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "ktav_fmt_cli_test_{}_{}_{name}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn formats_in_place() {
    let path = write_temp(
        "in_place.ktav",
        "## note\ndb: {host: primary, port: 5432}\n",
    );
    let status = Command::new(bin()).arg(&path).status().unwrap();
    assert!(status.success());
    let out = fs::read_to_string(&path).unwrap();
    assert_eq!(
        out,
        "## note\ndb: {\n    host: primary\n    port: 5432\n}\n"
    );
    fs::remove_file(&path).ok();
}

#[test]
fn stdout_mode_does_not_touch_the_file() {
    let original = "db: {host: primary, port: 5432}\n";
    let path = write_temp("stdout_mode.ktav", original);
    let output = Command::new(bin())
        .arg("--stdout")
        .arg(&path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let printed = String::from_utf8(output.stdout).unwrap();
    assert_eq!(printed, "db: {\n    host: primary\n    port: 5432\n}\n");
    // File on disk is untouched.
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    fs::remove_file(&path).ok();
}

#[test]
fn check_exits_zero_on_already_formatted_input_and_writes_nothing() {
    let already_formatted = "db: {\n    host: primary\n    port: 5432\n}\n";
    let path = write_temp("check_ok.ktav", already_formatted);
    let output = Command::new(bin())
        .arg("--check")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "expected no stdout for an already-formatted file"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        already_formatted,
        "file must be untouched"
    );
    fs::remove_file(&path).ok();
}

#[test]
fn check_exits_nonzero_on_unformatted_input_and_writes_nothing() {
    let unformatted = "db: {host: primary, port: 5432}\n";
    let path = write_temp("check_fail.ktav", unformatted);
    let output = Command::new(bin())
        .arg("--check")
        .arg(&path)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        unformatted,
        "file must be untouched by --check"
    );
    fs::remove_file(&path).ok();
}

#[test]
fn reads_stdin_and_writes_stdout() {
    let mut child = Command::new(bin())
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"db: {host: primary, port: 5432}\n")
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "db: {\n    host: primary\n    port: 5432\n}\n"
    );
}

#[test]
fn errors_on_missing_file_without_panicking() {
    let output = Command::new(bin())
        .arg("/no/such/file.ktav")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(!output.stderr.is_empty());
}

#[test]
fn no_arguments_prints_usage_and_fails() {
    let output = Command::new(bin()).output().unwrap();
    assert!(!output.status.success());
}
