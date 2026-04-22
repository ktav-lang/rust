//! Unit tests for render-internal helpers.

use super::helpers::{needs_raw_marker, push_indent, INDENT};

#[test]
fn ordinary_strings_do_not_need_marker() {
    assert!(!needs_raw_marker("hello"));
    assert!(!needs_raw_marker("8080"));
    assert!(!needs_raw_marker("a.b.c"));
    assert!(!needs_raw_marker(""));
}

#[test]
fn bracket_starting_strings_need_marker() {
    assert!(needs_raw_marker("[a-z]+"));
    assert!(needs_raw_marker("[::1]"));
    assert!(needs_raw_marker("[]"));
}

#[test]
fn brace_starting_strings_need_marker() {
    assert!(needs_raw_marker("{template}"));
    assert!(needs_raw_marker("{}"));
}

#[test]
fn keyword_strings_need_marker() {
    assert!(needs_raw_marker("true"));
    assert!(needs_raw_marker("false"));
    assert!(needs_raw_marker("null"));
}

#[test]
fn multiline_open_tokens_need_marker() {
    // Strings that would be mistaken for multi-line openers / inline-empty
    // forms must be protected with `::`.
    assert!(needs_raw_marker("("));
    assert!(needs_raw_marker("(("));
    assert!(needs_raw_marker("()"));
    assert!(needs_raw_marker("(())"));
}

#[test]
fn partial_paren_strings_do_not_need_marker() {
    // Only the exact tokens are parser-significant; anything else is
    // already a plain scalar to the parser.
    assert!(!needs_raw_marker("(foo"));
    assert!(!needs_raw_marker("(abc)"));
    assert!(!needs_raw_marker(")"));
    assert!(!needs_raw_marker("))"));
    assert!(!needs_raw_marker("a(b)c"));
}

#[test]
fn keyword_with_leading_whitespace_still_needs_marker() {
    assert!(needs_raw_marker("  true"));
    assert!(needs_raw_marker("\tnull"));
}

#[test]
fn capitalized_keywords_are_ordinary_strings() {
    assert!(!needs_raw_marker("True"));
    assert!(!needs_raw_marker("FALSE"));
    assert!(!needs_raw_marker("Null"));
}

#[test]
fn push_indent_writes_levels() {
    let mut out = String::new();
    push_indent(&mut out, 0);
    assert_eq!(out, "");
    push_indent(&mut out, 1);
    assert_eq!(out, INDENT);
    push_indent(&mut out, 2);
    assert_eq!(out, format!("{0}{0}{0}", INDENT));
}
