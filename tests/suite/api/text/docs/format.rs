//! The text formatter (`ktav::format_str`) and its CLI.

#[path = "format/format_canonical_equivalence.rs"]
mod format_canonical_equivalence;
#[path = "format/format_cli.rs"]
mod format_cli;
#[path = "format/format_comments_preserved.rs"]
mod format_comments_preserved;
#[path = "format/format_explicit_root_unclosed.rs"]
mod format_explicit_root_unclosed;

#[path = "format/format_idempotent.rs"]
mod format_idempotent;

#[path = "format/format_scalar_prefix_case.rs"]
mod format_scalar_prefix_case;
