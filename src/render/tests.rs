//! Unit tests for render-internal helpers.

use super::helpers::{needs_raw_marker, push_indent, INDENT};

#[test]
fn ordinary_strings_do_not_need_marker() {
    assert!(!needs_raw_marker("hello"));
    assert!(!needs_raw_marker("a.b.c"));
    assert!(!needs_raw_marker(""));
}

#[test]
fn numeric_strings_need_marker() {
    // Under 0.5.0, numbers are inferred from the lexical form.
    // A string that looks like a number needs `::` to stay a String.
    assert!(needs_raw_marker("8080"));
    assert!(needs_raw_marker("42"));
    assert!(needs_raw_marker("-7"));
    assert!(needs_raw_marker("+5"));
    assert!(needs_raw_marker("0xFF"));
    assert!(needs_raw_marker("3.14"));
    assert!(needs_raw_marker("1e10"));
    assert!(needs_raw_marker("1.5e-3"));
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
fn paren_prefixed_strings_need_marker() {
    // Under 0.5.0, any string starting with `(` is ambiguous with
    // multi-line openers and must use `::`.
    assert!(needs_raw_marker("(foo"));
    assert!(needs_raw_marker("(abc)"));
}

#[test]
fn non_paren_prefixed_strings() {
    // `)` and `))` are not openers.
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
