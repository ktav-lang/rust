//! Trivia-preserving parser for the formatter ([`crate::format_str`]).
//!
//! Mirrors [`super::parser::Parser`]'s line dispatch exactly (same root
//! detection, same pair / array-item classification, same dotted-key
//! insertion via the shared [`super::insert::insert_value`]), but builds
//! a [`PValue`] tree that additionally carries every comment and blank
//! line from the source as [`TriviaLine`]s attached to the construct
//! that follows them.
//!
//! Kept as a fork rather than a generic parameterisation of `Parser`
//! itself: `Parser` / `Frame` / `insert.rs`'s inline-compound path build
//! a concrete `Value` in several places (`parser::inline`) that would
//! need a second type parameter threaded through hot, allocation-
//! sensitive code for a feature only the formatter needs. Spec § 3.4
//! makes this safe to skip: a comment always owns a whole physical
//! line, so it can never occur inside an inline compound or inside a
//! dotted-key expansion (`a.b.c: 1`) — both are confined to a single
//! source line. Consequently an inline value or a dotted-key's
//! synthesized intermediate objects never need per-node trivia of their
//! own: [`doc::stamp`] wraps a plain `Value` (returned unchanged by
//! `classify_value_start` / `classify_root_kind_050`) with empty trivia
//! throughout, and a fresh intermediate object created by dotted-key
//! descent ([`PObject::descend`]) starts with empty trivia too — the
//! leading comment of a dotted-key line, if any, attaches to the leaf
//! segment only (a documented simplification, see `format_str`'s docs).
//!
//! Trivia attachment rule (resolves issue rust#13 open question 1 and
//! part of question 2): a run of comment / blank lines immediately
//! preceding a content line attaches to whatever that content line
//! produces (a key's value, an array item, or — if the line is a
//! closing `}` / `]` — the compound being closed, as `trailing`
//! trivia). Blank lines are preserved as a grouping hint but a run of
//! 2+ collapses to exactly one, and leading/trailing blank padding at
//! the start of a compound or right before its close is dropped;
//! comments are never dropped or collapsed. See `format_str` for the
//! full write-up.

use crate::error::Result;
use memchr::{memchr, memchr2};

mod doc;
mod positioned;

pub(crate) use doc::{to_plain_value, FmtDoc, PArray, PObject, PValue, TriviaLine};

use self::positioned::PositionedParser;

// ---------------------------------------------------------------------------
// Entry point — mirrors `parser::parse_str::parse_str_impl`'s line
// splitting exactly (LF-only fast path + CR-aware fallback).
// ---------------------------------------------------------------------------

/// Parse `text` into a trivia-carrying [`FmtDoc`]. Same grammar, same
/// errors, as [`crate::parse`] — the only difference is that comments
/// and blank lines are preserved instead of discarded.
pub(crate) fn parse_with_trivia(text: &str) -> Result<FmtDoc> {
    let mut parser = PositionedParser::new(false);
    let bytes = text.as_bytes();
    let start = super::leading_bom_len(text);

    if memchr(b'\r', bytes).is_none() {
        let mut line_start: usize = start;
        let mut line_num: usize = 0;
        loop {
            let end = memchr(b'\n', &bytes[line_start..])
                .map(|p| line_start + p)
                .unwrap_or(bytes.len());
            line_num += 1;
            let line: &str = &text[line_start..end];
            parser.handle_line(line, line_num, line_start as u32)?;
            if end == bytes.len() {
                break;
            }
            line_start = end + 1;
        }
        return parser.finish(bytes.len() as u32);
    }

    let mut line_start: usize = start;
    let mut line_num: usize = 0;
    while line_start <= bytes.len() {
        if line_start == bytes.len() {
            break;
        }
        let pos = memchr2(b'\n', b'\r', &bytes[line_start..])
            .map(|p| line_start + p)
            .unwrap_or(bytes.len());
        let content_end = pos;
        let next_start = if pos < bytes.len() {
            if bytes[pos] == b'\r' {
                if pos + 1 < bytes.len() && bytes[pos + 1] == b'\n' {
                    pos + 2
                } else {
                    pos + 1
                }
            } else {
                pos + 1
            }
        } else {
            bytes.len() + 1
        };
        let line: &str = &text[line_start..content_end];
        line_num += 1;
        parser.handle_line(line, line_num, line_start as u32)?;
        line_start = next_start;
    }
    parser.finish(bytes.len() as u32)
}
