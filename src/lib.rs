//! # Ktav — a plain configuration format
//!
//! JSON5-shaped, but with no quotes, no commas, and dotted keys for nesting.
//! The root is an Object or an Array, detected from the document's first
//! content line (spec § 5.0.1) — an empty or comment-only document is an
//! empty Object. Native `serde` integration: any type implementing
//! `Serialize` / `Deserialize` (including `#[derive]`-generated ones)
//! round-trips through Ktav out of the box.
//!
//! ## Syntax
//!
//! ```text
//! ## comment            — a line whose first non-whitespace chars are '##'
//! key: value            — scalar; `key` may be a dotted path (a.b.c: 10)
//! key:: value           — scalar; value is ALWAYS a literal string
//! key: { ... }          — multi-line object; `}` closes on its own line
//! key: [ ... ]          — multi-line array; `]` closes on its own line
//! key: {}  /  key: []   — empty compound, inline
//! :: value              — (inside an array) literal-string item
//! ```
//!
//! A single leading `#` (not `##`) is ordinary content, not a comment.
//!
//! ## Structured errors
//!
//! The parser returns [`Error::Structured`] for every parse failure since
//! `0.1.5`. Each [`ErrorKind`] variant carries a 1-based `line` and a
//! byte-offset [`Span`] you can slice the original input with.
//!
//! ```text
//! use ktav::{parse, Error, ErrorKind};
//!
//! // The first line anchors the document as an Object; the malformed
//! // `port:8080` on line 2 then errors with a `MissingSeparatorSpace`
//! // (a first-line `port:8080` would now be a top-level Array
//! // bare-scalar item — spec § 5.0.1).
//! let src = "anchor: ok\nport:8080\n";
//! match parse(src) {
//!     Ok(_) => unreachable!(),
//!     Err(Error::Structured(ErrorKind::MissingSeparatorSpace { line, span, .. })) => {
//!         assert_eq!(line, 2);
//!         // The span covers the offending body chunk glued to the marker.
//!         assert_eq!(span.slice(src), Some("8080"));
//!     }
//!     Err(other) => panic!("unexpected error: {other:?}"),
//! }
//! ```
//! (See `tests/suite/errors/surface/error_accessors.rs` for the executed
//! test.)
//!
//! ## C ABI for language bindings
//!
//! Behind the off-by-default `cabi` feature: [`cabi`] carries the shareable
//! half of the native shim — wire values, the six document operations,
//! envelope-encoded errors, the ABI version, the library naming convention —
//! and [`declare_cabi!`] expands the `#[no_mangle]` symbols inside a
//! binding's cdylib. See `docs/CABI.md`.
//!
//! ## Example
//!
//! See
//! [`tests/suite/api/text/docs/doc_example.rs`](../tests/suite/api/text/docs/doc_example.rs)
//! for the executed version of this snippet — it exercises the full
//! parse → struct → render → parse round-trip:
//!
//! ```text
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Upstream {
//!     host: String,
//!     port: u16,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct Config {
//!     port: u16,
//!     upstreams: Vec<Upstream>,
//! }
//!
//! let text = "\
//! port: 8080
//!
//! upstreams: [
//!     {
//!         host: a.example
//!         port: 1080
//!     }
//!     {
//!         host: b.example
//!         port: 1080
//!     }
//! ]
//! ";
//! let cfg: Config = ktav::from_str(text).unwrap();
//! assert_eq!(cfg.port, 8080);
//! assert_eq!(cfg.upstreams.len(), 2);
//!
//! let back = ktav::to_string(&cfg).unwrap();
//! let round: Config = ktav::from_str(&back).unwrap();
//! assert_eq!(cfg, round);
//! ```
#![allow(clippy::module_inception)]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "cabi")]
pub mod cabi;
pub mod error;

mod document;
pub use document::{parser, render, value};

mod serde_bridge;
pub use serde_bridge::{de, ser, thin};

#[cfg(test)]
mod arena_probe;

// Lets the test-only probe #[path]-include benches/fixtures.rs, which is
// written against the external `ktav` prelude (integration-test style).
#[cfg(test)]
extern crate self as ktav;

#[path = "document/whitespace.rs"]
mod whitespace;

pub use error::{
    CompoundKind, ConflictKind, Error, ErrorEnvelope, ErrorKind, ReasonCode, Result, Span,
};
pub use thin::{parse_events, ParseEvent};
pub use value::{ObjectMap, Value};

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_bridge::{EventCursor, EventDeserializer};

/// Parse a Ktav document from a string into a raw [`Value`]. Useful when
/// you want to inspect or manipulate the document generically. For
/// deserializing into a user type, prefer [`from_str`].
pub fn parse(text: &str) -> Result<Value> {
    parser::parse_str(text)
}

/// Parse a Ktav document like [`parse`], but reject **lossy scalars**:
/// values whose lexical form differs from the canonical form of the
/// number they would be inferred as (§ 3.6 / § 5.2), e.g. `1.10`
/// (→ `1.1`), `+7`, `0x1A`, `1_000`, `5e3`. Type inference would
/// silently rewrite such values; strict mode surfaces them as
/// [`ErrorKind::LossyScalar`] so the author can either append `::`
/// (keep the scalar a String) or write the canonical number.
///
/// A redundant leading zero (`01234`) is **not** rejected here: § 5.2
/// infers no number from it at all, so both entry points agree it is
/// the String `"01234"` and there is nothing to lose.
///
/// Documents accepted by `parse_strict` produce exactly the same
/// [`Value`] tree as [`parse`]. The serde event path ([`from_str`])
/// has no strict variant yet.
pub fn parse_strict(text: &str) -> Result<Value> {
    parser::parse_str_strict(text)
}

/// Parse a Ktav document from a string and deserialize it into `T`.
/// Uses the zero-copy event path: the document is tokenized line by
/// line into a flat `Vec<Event>` — object keys and unmodified scalars
/// are borrowed directly from `s`; only escape-decoded strings and
/// canonical numeric forms are allocated, in a temporary arena freed
/// when this call returns. Serde walks the events linearly without
/// ever materialising an owned tree; compound nesting is bracketed by
/// `BeginObject`/`EndObject` events. Inline compounds (`a: {x: 1}`)
/// are staged, not streamed: the scanner appends their events to a
/// flat per-compound `Vec<Event>` scratch (array items — nested
/// arrays included — land there in source order, each exactly once;
/// only arrays that are directly an object member's value are staged
/// as one bracketed block until the object's insertion order is
/// final), keeps one
/// insertion-ordered key table (`IndexMap`) per object scope for
/// duplicate/conflict detection and dotted-key merge, and copies the
/// finished events into the stream only after the compound validates
/// (through the parser's reusable per-parse staging buffer when the
/// compound is a key's value). No owned `Value` is built, and a
/// compound that fails validation emits no events.
pub fn from_str<T: DeserializeOwned>(s: &str) -> Result<T> {
    // Pre-size the bump arena to avoid re-allocations during event parsing.
    // Each Event is 24 bytes; the pre-allocated count is ~text.len()/4.
    // Adding overhead for the bump metadata and per-object-level BumpVecs.
    let arena_bytes = (s.len() / 4).saturating_mul(24) + 4096;
    let bump = bumpalo::Bump::with_capacity(arena_bytes);
    // parse_events_merged applies the spec § 5.3.2 reopen-merge pass
    // only when the parser reports re-opened dotted-key prefixes;
    // otherwise the raw (zero-copy) stream is used as-is.
    let events = thin::parse_events_merged(s, &bump)?;
    let mut cursor = EventCursor::new(&events);
    T::deserialize(EventDeserializer::new(&mut cursor))
}

/// Parse a Ktav document from a file path and deserialize it into `T`.
///
/// The file's raw bytes are validated as UTF-8 before parsing (spec
/// 0.8 § 6.15): invalid UTF-8 fails as [`Error::InvalidUtf8`] carrying
/// the byte offset of the first invalid sequence, while genuine I/O
/// failures (file not found, permission denied) remain [`Error::Io`].
pub fn from_file<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
    let bytes = fs::read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| Error::InvalidUtf8 {
        valid_up_to: e.valid_up_to(),
    })?;
    from_str(text)
}

/// Serialize `value` as a Ktav document string. Uses the direct text
/// serializer — no `Value` intermediate.
pub fn to_string<T: ?Sized + Serialize>(value: &T) -> Result<String> {
    ser::to_string(value)
}

/// Serialize `value` as a Ktav document and write it to `path`.
pub fn to_file<T: ?Sized + Serialize, P: AsRef<Path>>(value: &T, path: P) -> Result<()> {
    let text = to_string(value)?;
    fs::write(path, text)?;
    Ok(())
}

/// Render a [`Value`] with **every scalar coerced to a String** —
/// typed integers (`:i`), typed floats (`:f`), booleans, and null
/// are flattened to their textual form and emitted via the raw-
/// marker `::`. Compounds (Object / Array) preserve their structure;
/// only leaf scalars are coerced. The output round-trips back
/// through the parser as the same set of String scalars.
///
/// Useful for "everything is a string" dumps — e.g. for downstream
/// consumers that don't understand typed markers, or for diff-
/// friendly canonical text.
pub fn to_string_force_strings(value: &Value) -> Result<String> {
    render::to_string_force_strings(value)
}

/// Emit a canonical Ktav serialisation of `value` (spec § 5.9).
///
/// The top-level value must be an Object or an Array (§ 5.0.1).
/// Returns an error for any other variant, or if a String contains a
/// `CR` byte (not representable in canonical form, § 5.9.7).
///
/// Two writer-conforming implementations fed the same [`Value`] MUST produce
/// identical output (§ 8.2).
pub fn emit_canonical(value: &Value) -> Result<String> {
    render::emit_canonical(value)
}

/// Format a Ktav document: `text -> text`, normalising structure to
/// § 5.9 canonical form (issue rust#13) while preserving every comment
/// line and blank-line grouping from the source. Idempotent:
/// `format_str(&format_str(text)?)? == format_str(text)?`. Suitable for
/// a `pre-commit` hook or a `--check` gate in CI — the point is that a
/// diff no longer depends on whether someone wrote an object inline or
/// as a block.
///
/// [`emit_canonical`] cannot be that formatter directly: comments are
/// not part of the [`Value`] model at all (§ 5.9.2 — "Comments … are
/// never emitted"), so round-tripping through `Value` silently deletes
/// every comment in the document. `format_str` instead parses straight
/// from text into a trivia-carrying tree (never through `Value`) and
/// writes it back out with the same § 5.9 structural rules
/// `emit_canonical` uses, interleaved with the preserved trivia.
///
/// Every comment in the input is preserved verbatim (spec § 3.4: a
/// comment is a whole line, so it always attaches unambiguously to the
/// content line that follows it, or — with nothing left to follow — to
/// the end of the enclosing compound / the document). Four points from
/// that design are worth calling out explicitly:
///
/// - **Blank lines** are preserved as a grouping hint (a blank line
///   between two keys survives), but a run of two or more collapses to
///   exactly one, and leading/trailing blank padding right after an
///   opening bracket or right before a closing one (or at end of file)
///   is dropped. This keeps the transform a fixed point: reformatting
///   already-formatted output never changes it.
/// - **Trivia representation.** Comments and blanks are captured by a
///   dedicated parser fork (`parser::fmt_parser`) that mirrors the
///   `Value`-tree parser's line dispatch line for line — same root
///   detection, same pair/array-item grammar, same dotted-key
///   insertion — and threads a `Vec` of trivia lines through the exact
///   points where the real parser already decides "this line starts a
///   new key/item" or "this line closes a compound". A comment can
///   never appear inside an inline compound or a dotted-key expansion
///   (both are confined to one physical source line, spec § 3.4), so
///   this needs no separate lossless syntax tree and no changes to the
///   hot parsing/serialization paths at all.
/// - **`format_str` of a comment-free document equals [`emit_canonical`]
///   of its parse, *provided the document also has no blank lines.* A
///   comment-free document can still contain grouping blank lines,
///   which `emit_canonical` always drops (blank lines are no more part
///   of the `Value` model than comments are, § 3.5) but `format_str`
///   preserves per the point above — so equality needs the stronger
///   condition "no comments and no blank lines", not just "no
///   comments".
/// - **Key order is never changed.** Canonical form does not reorder
///   keys (§ 5.9 has no sorting rule), and neither does `format_str` —
///   it is a structural-spelling normaliser, not a refactoring tool;
///   reordering keys would make review diffs worse, not better.
///
/// One documented simplification: for a dotted key (`a.b.c: 1`), a
/// preceding comment attaches to the deepest segment (`c`) rather than
/// to the outermost one synthesized for it (`a`) — the two syntheses
/// are otherwise indistinguishable once the key is split, and comments
/// immediately preceding a dotted-key line are rare enough that this
/// corner is not worth extra bookkeeping. The comment is never lost,
/// only nested one level deeper than a literal reading of "attaches to
/// the line below" might expect.
pub fn format_str(text: &str) -> Result<String> {
    let doc = parser::fmt_parser::parse_with_trivia(text)?;
    render::formatted::emit_formatted(&doc)
}
