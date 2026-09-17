//! Unit tests for parser-internal helpers.

#[path = "../../../benches/fixtures_ix.rs"]
mod ix_fixtures;

mod fmt_differential_tests;
mod helpers;
mod inline_values;
mod ix_probe;
mod midvalue_braces;
mod quote_prescan;
mod quoted_keys;
mod quoted_keys_parse;

use crate::error::Span;

const S: Span = Span::EMPTY;
