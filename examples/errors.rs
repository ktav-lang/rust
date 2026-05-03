//! Demonstrates the structured-error API: every parse failure carries
//! a typed `ErrorKind` with a byte-offset `Span`, so editor integrations
//! and tooling can highlight the exact offending range instead of a
//! whole line.
//!
//! Run with:
//!
//!     cargo run -p ktav --example errors

use ktav::{parse, Error, ErrorKind};

fn main() {
    // Each input below triggers a different error category. We dispatch
    // on the structured variant and print line / span / category.
    for src in [
        "port:8080\n",           // MissingSeparatorSpace
        "port: 80\nport: 443\n", // DuplicateKey
        "port:i twelve\n",       // InvalidTypedScalar
        ": orphan\n",            // EmptyKey
        "key: {\n  inner: 1\n",  // UnclosedCompound (object)
        "}\n",                   // UnbalancedBracket
        "x: {foo}\n",            // InlineNonEmptyCompound
        "just-some-text\n",      // MissingSeparator
    ] {
        match parse(src) {
            Ok(_) => unreachable!("input is invalid: {src:?}"),
            Err(Error::Structured(kind)) => describe(src, &kind),
            Err(other) => println!("other: {other}"),
        }
    }
}

fn describe(src: &str, kind: &ErrorKind) {
    let span = kind.span();
    let (line, column) = span.line_col(src);
    let slice = span.slice(src).unwrap_or("");
    let category = match kind {
        ErrorKind::MissingSeparatorSpace { .. } => "MissingSeparatorSpace",
        ErrorKind::InvalidTypedScalar { .. } => "InvalidTypedScalar",
        ErrorKind::DuplicateKey { .. } => "DuplicateKey",
        ErrorKind::KeyPathConflict { .. } => "KeyPathConflict",
        ErrorKind::EmptyKey { .. } => "EmptyKey",
        ErrorKind::InvalidKey { .. } => "InvalidKey",
        ErrorKind::UnclosedCompound { .. } => "UnclosedCompound",
        ErrorKind::UnbalancedBracket { .. } => "UnbalancedBracket",
        ErrorKind::InlineNonEmptyCompound { .. } => "InlineNonEmptyCompound",
        ErrorKind::MissingSeparator { .. } => "MissingSeparator",
        ErrorKind::Other { .. } => "Other",
        _ => "<unknown — ErrorKind is #[non_exhaustive]>",
    };
    println!(
        "{:<24} line {} col {} bytes {}..{} -> {:?}",
        category, line, column, span.start, span.end, slice
    );
}
