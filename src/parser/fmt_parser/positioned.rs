//! The trivia-preserving parser itself: `PFrame` and `PositionedParser`, mirroring `super::parser::Parser`'s line dispatch.

use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

use crate::error::{CompoundKind, Error, ErrorKind, Result, Span};
use crate::whitespace::is_ktav_whitespace;

use crate::parser::bracket::Bracket;
use crate::parser::classify::classify_value_start;
use crate::parser::collecting::{Collecting, MultilineMode};
use crate::parser::inline::{scan_unescaped_colon, ColonScan};
use crate::parser::insert::insert_value;
use crate::parser::parser::{
    bracket_to_compound, classify_root_kind_050, classify_separator, require_sep_end, RootResult,
    Separator,
};
use crate::parser::value_start::ValueStart;

use super::doc::{stamp, trim_trailing_blanks, FmtDoc, PArray, PObject, PValue, TriviaLine};

// ---------------------------------------------------------------------------
// Parser frames
// ---------------------------------------------------------------------------

enum PFrame<'a> {
    Object {
        table: PObject,
        pending_key: Option<&'a str>,
        pending_key_span: Option<Span>,
        pending_key_trivia: Vec<TriviaLine>,
        at_start: bool,
    },
    Array {
        table: PArray,
        pending_item_trivia: Vec<TriviaLine>,
        at_start: bool,
    },
}

impl<'a> PFrame<'a> {
    fn new_object() -> Self {
        PFrame::Object {
            table: PObject {
                pairs: IndexMap::with_capacity_and_hasher(8, FxBuildHasher),
                trailing: Vec::new(),
            },
            pending_key: None,
            pending_key_span: None,
            pending_key_trivia: Vec::new(),
            at_start: true,
        }
    }

    fn new_array() -> Self {
        PFrame::Array {
            table: PArray {
                items: Vec::with_capacity(8),
                trailing: Vec::new(),
            },
            pending_item_trivia: Vec::new(),
            at_start: true,
        }
    }

    fn into_value(self) -> PValue {
        match self {
            PFrame::Object { table, .. } => PValue::Object(table),
            PFrame::Array { table, .. } => PValue::Array(table),
        }
    }

    fn set_started(&mut self) {
        match self {
            PFrame::Object { at_start, .. } | PFrame::Array { at_start, .. } => *at_start = false,
        }
    }

    fn is_at_start(&self) -> bool {
        match self {
            PFrame::Object { at_start, .. } | PFrame::Array { at_start, .. } => *at_start,
        }
    }

    fn trailing_mut(&mut self) -> &mut Vec<TriviaLine> {
        match self {
            PFrame::Object { table, .. } => &mut table.trailing,
            PFrame::Array { table, .. } => &mut table.trailing,
        }
    }
}
// ---------------------------------------------------------------------------
// The parser itself
// ---------------------------------------------------------------------------

pub(super) struct PositionedParser<'a> {
    strict: bool,
    stack: Vec<PFrame<'a>>,
    collecting: Option<Collecting<'a>>,
    opener_offsets: Vec<u32>,
    multiline_opener: Option<u32>,
    root_initialized: bool,
    root_consumed: bool,
    root_inline_value: Option<PValue>,
    root_is_explicit_compound: bool,
    /// Comment / blank lines seen since the last content line, not yet
    /// attached anywhere.
    pending_trivia: Vec<TriviaLine>,
    /// Trivia preceding the document's very first content line, when
    /// that line does not itself become a tree node with a leading-
    /// trivia slot (an inline-root document, or the `{` / `[` opener of
    /// an explicit-compound root). A normal implicit root's leading
    /// trivia lands on its first child instead — see `finish`.
    doc_leading: Vec<TriviaLine>,
}

impl<'a> PositionedParser<'a> {
    pub(super) fn new(strict: bool) -> Self {
        Self {
            strict,
            stack: Vec::with_capacity(8),
            collecting: None,
            opener_offsets: Vec::with_capacity(8),
            multiline_opener: None,
            root_initialized: false,
            root_consumed: false,
            root_inline_value: None,
            root_is_explicit_compound: false,
            pending_trivia: Vec::new(),
            doc_leading: Vec::new(),
        }
    }

    fn current_at_start(&self) -> bool {
        match self.stack.last() {
            Some(frame) => frame.is_at_start(),
            None => !self.root_initialized,
        }
    }

    fn push_blank_trivia(&mut self) {
        if self.pending_trivia.is_empty() && self.current_at_start() {
            return;
        }
        if matches!(self.pending_trivia.last(), Some(TriviaLine::Blank)) {
            return;
        }
        self.pending_trivia.push(TriviaLine::Blank);
    }

    fn push_comment_trivia(&mut self, trimmed: &str) {
        self.pending_trivia
            .push(TriviaLine::Comment(trimmed.to_string()));
    }

    pub(super) fn finish(mut self, eof_offset: u32) -> Result<FmtDoc> {
        if let Some(c) = &self.collecting {
            let kind = match c.mode {
                MultilineMode::Stripped => CompoundKind::MultilineStripped,
                MultilineMode::Verbatim => CompoundKind::MultilineVerbatim,
            };
            let start = self.multiline_opener.unwrap_or(eof_offset);
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind,
                span: Span::new(start, eof_offset),
            }));
        }
        if self.stack.len() > 1 {
            let kind = match self.stack.last().unwrap() {
                PFrame::Object { .. } => CompoundKind::Object,
                PFrame::Array { .. } => CompoundKind::Array,
            };
            let start = *self.opener_offsets.last().unwrap();
            return Err(Error::Structured(ErrorKind::UnclosedCompound {
                kind,
                span: Span::new(start, eof_offset),
            }));
        }

        let eof_trivia = trim_trailing_blanks(std::mem::take(&mut self.pending_trivia));

        if let Some(mut v) = self.root_inline_value.take() {
            match &mut v {
                PValue::Object(o) => o.trailing.extend(eof_trivia),
                PValue::Array(a) => a.trailing.extend(eof_trivia),
                _ => unreachable!("top-level inline root is always Object or Array"),
            }
            return Ok(FmtDoc {
                leading: self.doc_leading,
                root: v,
            });
        }

        if self.stack.is_empty() {
            // Empty / comments-only document (§ 5.0.1 rule 1): empty
            // Object root; any trivia becomes the document's leading
            // trivia — there is no content anywhere to attach it to.
            let mut leading = self.doc_leading;
            leading.extend(eof_trivia);
            return Ok(FmtDoc {
                leading,
                root: PValue::Object(PObject::default()),
            });
        }

        let mut root = self.stack.pop().unwrap().into_value();
        match &mut root {
            PValue::Object(o) => o.trailing.extend(eof_trivia),
            PValue::Array(a) => a.trailing.extend(eof_trivia),
            _ => unreachable!(),
        }
        Ok(FmtDoc {
            leading: self.doc_leading,
            root,
        })
    }

    pub(super) fn handle_line(
        &mut self,
        raw: &'a str,
        line_num: usize,
        line_start: u32,
    ) -> Result<()> {
        if let Some(ref mut collecting) = self.collecting {
            let trimmed = raw.trim_matches(is_ktav_whitespace);
            if collecting.is_terminator(trimmed) {
                let finished = self.collecting.take().unwrap().finish();
                self.multiline_opener = None;
                return self.attach_scalar_value(
                    PValue::String(finished.into()),
                    line_num,
                    line_start,
                );
            }
            collecting.lines.push(raw);
            return Ok(());
        }

        let trimmed = raw.trim_matches(is_ktav_whitespace);

        if trimmed.is_empty() {
            self.push_blank_trivia();
            return Ok(());
        }
        if trimmed.starts_with("##") {
            self.push_comment_trivia(trimmed);
            return Ok(());
        }

        let trimmed_span = crate::parser::parser::trimmed_span_in(raw, trimmed, line_start);

        if self.root_consumed {
            return Err(Error::Structured(
                ErrorKind::OrphanLineAfterTopLevelInline {
                    line: line_num as u32,
                    span: trimmed_span,
                },
            ));
        }

        if !self.root_initialized {
            self.root_initialized = true;

            if trimmed != "}" && trimmed != "]" {
                match classify_root_kind_050(trimmed, line_num, trimmed_span, self.strict)? {
                    RootResult::InlineObject(value) | RootResult::InlineArray(value) => {
                        self.root_consumed = true;
                        self.doc_leading = std::mem::take(&mut self.pending_trivia);
                        self.root_inline_value = Some(stamp(value));
                        return Ok(());
                    }
                    RootResult::ExplicitObject => {
                        self.doc_leading = std::mem::take(&mut self.pending_trivia);
                        self.root_is_explicit_compound = true;
                        self.stack.push(PFrame::new_object());
                        self.opener_offsets.push(trimmed_span.start);
                        return Ok(());
                    }
                    RootResult::ExplicitArray => {
                        self.doc_leading = std::mem::take(&mut self.pending_trivia);
                        self.root_is_explicit_compound = true;
                        self.stack.push(PFrame::new_array());
                        self.opener_offsets.push(trimmed_span.start);
                        return Ok(());
                    }
                    RootResult::Object => {
                        self.stack.push(PFrame::new_object());
                        self.opener_offsets.push(0);
                    }
                    RootResult::Array => {
                        self.stack.push(PFrame::new_array());
                        self.opener_offsets.push(0);
                    }
                }
            }
        }

        if trimmed == "}" {
            return self.close_frame(Bracket::Object, line_num, trimmed_span);
        }
        if trimmed == "]" {
            return self.close_frame(Bracket::Array, line_num, trimmed_span);
        }

        let trivia = std::mem::take(&mut self.pending_trivia);
        self.stack.last_mut().unwrap().set_started();
        if matches!(self.stack.last(), Some(PFrame::Array { .. })) {
            self.handle_array_item(trimmed, line_num, trimmed_span, trivia)
        } else {
            self.handle_object_pair(raw, trimmed, line_num, line_start, trimmed_span, trivia)
        }
    }

    fn attach_scalar_value(
        &mut self,
        value: PValue,
        line_num: usize,
        line_start: u32,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            PFrame::Object {
                table,
                pending_key,
                pending_key_span,
                pending_key_trivia,
                ..
            } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} multi-line string closed without pending key",
                            line_num
                        ),
                        span: Span::new(line_start, line_start),
                    })
                })?;
                let key_span = pending_key_span
                    .take()
                    .unwrap_or_else(|| Span::new(line_start, line_start));
                let trivia = std::mem::take(pending_key_trivia);
                insert_value(table, key, (trivia, value), line_num, key_span)
            }
            PFrame::Array {
                table,
                pending_item_trivia,
                ..
            } => {
                let trivia = std::mem::take(pending_item_trivia);
                table.items.push((trivia, value));
                Ok(())
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_object_pair(
        &mut self,
        raw: &'a str,
        line: &'a str,
        line_num: usize,
        line_start: u32,
        trimmed_span: Span,
        trivia: Vec<TriviaLine>,
    ) -> Result<()> {
        let trimmed_off_in_raw = (trimmed_span.start - line_start) as usize;

        let colon = match scan_unescaped_colon(line) {
            ColonScan::Found(c) => c,
            ColonScan::UnterminatedQuote => {
                return Err(Error::Structured(ErrorKind::UnterminatedQuotedKey {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
            ColonScan::Absent => {
                return Err(Error::Structured(ErrorKind::MissingSeparator {
                    line: line_num as u32,
                    span: trimmed_span,
                }));
            }
        };

        let key = line[..colon].trim_end_matches(is_ktav_whitespace);
        let key_start = trimmed_span.start;
        let key_end = key_start + key.len() as u32;
        let _ = raw;
        if key.is_empty() {
            return Err(Error::Structured(ErrorKind::EmptyKey {
                line: line_num as u32,
                span: Span::new(key_start, key_start + 1),
            }));
        }

        let after_colon = &line[colon + 1..];
        let after_colon_off_in_line = colon + 1;
        let after_colon_off = line_start + (trimmed_off_in_raw + after_colon_off_in_line) as u32;
        let sep = classify_separator(after_colon);
        let marker_col = colon as u32;

        match sep {
            Separator::Raw(after) => {
                require_sep_end(
                    after,
                    line_num,
                    line_start,
                    marker_col,
                    after_colon_off + 1,
                    trimmed_span,
                )?;
                let value = PValue::String(after.trim_matches(is_ktav_whitespace).into());
                self.insert_object_pair(key, value, line_num, Span::new(key_start, key_end), trivia)
            }
            Separator::Plain(after) => {
                require_sep_end(
                    after,
                    line_num,
                    line_start,
                    marker_col,
                    after_colon_off,
                    trimmed_span,
                )?;
                let key_span = Span::new(key_start, key_end);
                match classify_value_start(after, line_num, trimmed_span, self.strict)? {
                    ValueStart::Scalar(s) => {
                        self.insert_object_pair(key, PValue::String(s), line_num, key_span, trivia)
                    }
                    ValueStart::Null => {
                        self.insert_object_pair(key, PValue::Null, line_num, key_span, trivia)
                    }
                    ValueStart::Bool(b) => {
                        self.insert_object_pair(key, PValue::Bool(b), line_num, key_span, trivia)
                    }
                    ValueStart::Integer(s) => {
                        self.insert_object_pair(key, PValue::Integer(s), line_num, key_span, trivia)
                    }
                    ValueStart::Float(s) => {
                        self.insert_object_pair(key, PValue::Float(s), line_num, key_span, trivia)
                    }
                    ValueStart::EmptyObject => self.insert_object_pair(
                        key,
                        PValue::Object(PObject::default()),
                        line_num,
                        key_span,
                        trivia,
                    ),
                    ValueStart::EmptyArray => self.insert_object_pair(
                        key,
                        PValue::Array(PArray::default()),
                        line_num,
                        key_span,
                        trivia,
                    ),
                    ValueStart::OpenObject => {
                        self.set_pending_key(key, line_num, line_start, key_span, trivia)?;
                        self.stack.push(PFrame::new_object());
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenArray => {
                        self.set_pending_key(key, line_num, line_start, key_span, trivia)?;
                        self.stack.push(PFrame::new_array());
                        self.opener_offsets.push(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineStripped => {
                        self.set_pending_key(key, line_num, line_start, key_span, trivia)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                        self.multiline_opener = Some(trimmed_span.end - 1);
                        Ok(())
                    }
                    ValueStart::OpenMultilineVerbatim => {
                        self.set_pending_key(key, line_num, line_start, key_span, trivia)?;
                        self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                        self.multiline_opener = Some(trimmed_span.end - 2);
                        Ok(())
                    }
                    ValueStart::InlineValue(v) => {
                        self.insert_object_pair(key, stamp(v), line_num, key_span, trivia)
                    }
                }
            }
        }
    }

    fn handle_array_item(
        &mut self,
        line: &str,
        line_num: usize,
        trimmed_span: Span,
        trivia: Vec<TriviaLine>,
    ) -> Result<()> {
        let line_start = trimmed_span.start;

        if let Some(rest) = line.strip_prefix("::") {
            require_sep_end(rest, line_num, line_start, 1, line_start + 2, trimmed_span)?;
            let value = PValue::String(rest.trim_start_matches(is_ktav_whitespace).into());
            return self.push_array_item(value, trivia);
        }

        match classify_value_start(line, line_num, trimmed_span, self.strict)? {
            ValueStart::Scalar(s) => self.push_array_item(PValue::String(s), trivia),
            ValueStart::Null => self.push_array_item(PValue::Null, trivia),
            ValueStart::Bool(b) => self.push_array_item(PValue::Bool(b), trivia),
            ValueStart::Integer(s) => self.push_array_item(PValue::Integer(s), trivia),
            ValueStart::Float(s) => self.push_array_item(PValue::Float(s), trivia),
            ValueStart::EmptyObject => {
                self.push_array_item(PValue::Object(PObject::default()), trivia)
            }
            ValueStart::EmptyArray => {
                self.push_array_item(PValue::Array(PArray::default()), trivia)
            }
            ValueStart::OpenObject => {
                self.set_pending_item(trivia);
                self.stack.push(PFrame::new_object());
                self.opener_offsets.push(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenArray => {
                self.set_pending_item(trivia);
                self.stack.push(PFrame::new_array());
                self.opener_offsets.push(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenMultilineStripped => {
                self.set_pending_item(trivia);
                self.collecting = Some(Collecting::new(MultilineMode::Stripped));
                self.multiline_opener = Some(trimmed_span.end - 1);
                Ok(())
            }
            ValueStart::OpenMultilineVerbatim => {
                self.set_pending_item(trivia);
                self.collecting = Some(Collecting::new(MultilineMode::Verbatim));
                self.multiline_opener = Some(trimmed_span.end - 2);
                Ok(())
            }
            ValueStart::InlineValue(v) => self.push_array_item(stamp(v), trivia),
        }
    }

    fn set_pending_item(&mut self, trivia: Vec<TriviaLine>) {
        if let Some(PFrame::Array {
            pending_item_trivia,
            ..
        }) = self.stack.last_mut()
        {
            *pending_item_trivia = trivia;
        }
    }

    fn insert_object_pair(
        &mut self,
        key: &str,
        value: PValue,
        line_num: usize,
        key_span: Span,
        trivia: Vec<TriviaLine>,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            PFrame::Object { table, .. } => {
                insert_value(table, key, (trivia, value), line_num, key_span)
            }
            PFrame::Array { .. } => unreachable!("dispatched as object"),
        }
    }

    fn push_array_item(&mut self, value: PValue, trivia: Vec<TriviaLine>) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            PFrame::Array { table, .. } => {
                table.items.push((trivia, value));
                Ok(())
            }
            PFrame::Object { .. } => unreachable!("dispatched as array"),
        }
    }

    fn set_pending_key(
        &mut self,
        key: &'a str,
        line_num: usize,
        line_start: u32,
        key_span: Span,
        trivia: Vec<TriviaLine>,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            PFrame::Object {
                pending_key,
                pending_key_span,
                pending_key_trivia,
                ..
            } => {
                if pending_key.is_some() {
                    return Err(Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} pending key already set",
                            line_num
                        ),
                        span: Span::new(line_start, line_start),
                    }));
                }
                *pending_key = Some(key);
                *pending_key_span = Some(key_span);
                *pending_key_trivia = trivia;
                Ok(())
            }
            _ => unreachable!(),
        }
    }

    fn close_frame(
        &mut self,
        expected: Bracket,
        line_num: usize,
        trimmed_span: Span,
    ) -> Result<()> {
        let trailing = trim_trailing_blanks(std::mem::take(&mut self.pending_trivia));

        if self.stack.len() <= 1 {
            if self.stack.len() == 1 && self.root_is_explicit_compound {
                let frame = self.stack.last().unwrap();
                let frame_kind = match frame {
                    PFrame::Object { .. } => Bracket::Object,
                    PFrame::Array { .. } => Bracket::Array,
                };
                if !matches!(
                    (frame_kind, expected),
                    (Bracket::Object, Bracket::Object) | (Bracket::Array, Bracket::Array)
                ) {
                    return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                        line: line_num as u32,
                        span: trimmed_span,
                        expected: bracket_to_compound(frame_kind),
                        found: expected.close(),
                    }));
                }
                self.stack
                    .last_mut()
                    .unwrap()
                    .trailing_mut()
                    .extend(trailing);
                self.root_consumed = true;
                return Ok(());
            }
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: bracket_to_compound(expected),
                found: expected.close(),
            }));
        }
        let mut frame = self.stack.pop().unwrap();
        let _ = self.opener_offsets.pop();
        let frame_kind = match frame {
            PFrame::Object { .. } => Bracket::Object,
            PFrame::Array { .. } => Bracket::Array,
        };
        let matches_expected = matches!(
            (frame_kind, expected),
            (Bracket::Object, Bracket::Object) | (Bracket::Array, Bracket::Array)
        );
        if !matches_expected {
            return Err(Error::Structured(ErrorKind::UnbalancedBracket {
                line: line_num as u32,
                span: trimmed_span,
                expected: bracket_to_compound(frame_kind),
                found: expected.close(),
            }));
        }
        frame.trailing_mut().extend(trailing);
        let value = frame.into_value();
        self.attach_child_value(value, line_num, trimmed_span)
    }

    fn attach_child_value(
        &mut self,
        value: PValue,
        line_num: usize,
        trimmed_span: Span,
    ) -> Result<()> {
        match self.stack.last_mut().unwrap() {
            PFrame::Object {
                table,
                pending_key,
                pending_key_span,
                pending_key_trivia,
                ..
            } => {
                let key = pending_key.take().ok_or_else(|| {
                    Error::Structured(ErrorKind::Other {
                        line: Some(line_num as u32),
                        message: format!(
                            "Line {}: internal error \u{2014} closed compound without pending key",
                            line_num
                        ),
                        span: trimmed_span,
                    })
                })?;
                let key_span = pending_key_span.take().unwrap_or(trimmed_span);
                let trivia = std::mem::take(pending_key_trivia);
                insert_value(table, key, (trivia, value), line_num, key_span)
            }
            PFrame::Array {
                table,
                pending_item_trivia,
                ..
            } => {
                let trivia = std::mem::take(pending_item_trivia);
                table.items.push((trivia, value));
                Ok(())
            }
        }
    }
}
