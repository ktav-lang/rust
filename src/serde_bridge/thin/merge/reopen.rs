use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;

use super::super::{Event, EventStream};
#[cfg(test)]
use super::perf;
use super::table::{Buf, Item};

/// Flatten work item: append `Ev` to the output, or splice in buffer
/// `buf` bracketed by `open` / its matching close.
enum Work<'a> {
    Ev(Event<'a>),
    /// Emit `open`, splice buffer `buf`, then emit `close`.
    Enter {
        buf: usize,
        open: Event<'a>,
        close: Event<'a>,
    },
}

fn matching_close(open: Event<'_>) -> Event<'_> {
    match open {
        Event::BeginObject => Event::EndObject,
        Event::BeginArray => Event::EndArray,
        _ => unreachable!("open is always BeginObject/BeginArray"),
    }
}

fn is_open(ev: &Event<'_>) -> bool {
    matches!(ev, Event::BeginObject | Event::BeginArray)
}

/// Rebuild `src` folding re-opened object blocks into the buffer of
/// their first appearance (spec 0.7 § 5.3.2). Merging only removes
/// events, so the output is pre-sized to `src.len()`.
pub(crate) fn merge_reopened<'a>(src: &EventStream<'a>, bump: &'a Bump) -> EventStream<'a> {
    let mut bufs: Vec<Buf<'a>> = vec![Buf {
        items: BumpVec::with_capacity_in(src.len(), bump),
        seen: BumpVec::new_in(bump),
        index: None,
    }];
    // Index 0 is the document root, pre-created above. `root_open`
    // records its kind (implicit or explicit § 5.0.1 root) so the
    // flatten step can re-emit the bracket pair around the root.
    let mut root_open: Option<Event<'a>> = None;
    // Explicit stack of open buffer indices; the root frame is pushed
    // when the bare root opener is seen.
    let mut stack: Vec<usize> = Vec::with_capacity(8);

    let mut i = 0;
    while i < src.len() {
        match src[i] {
            // A bare Begin with no preceding Key is the document root
            // opener (§ 5.0.1). Its matching close pops the root frame.
            ev @ (Event::BeginObject | Event::BeginArray) if stack.is_empty() => {
                root_open = Some(ev);
                stack.push(0);
                i += 1;
            }
            // A bare Begin with a frame already open is an anonymous
            // compound value (an array element — only array elements
            // lack a key). It is never a § 5.3.2 reopen target (the
            // parser's persistent path table only tracks keyed paths),
            // so it always gets a fresh child buffer — same treatment
            // as the keyed array-valued arm below, minus the `seen`
            // entry and the `Key` item.
            ev @ (Event::BeginObject | Event::BeginArray) => {
                let idx = bufs.len();
                bufs.push(new_buf(bump));
                bufs[stack_top(&stack)]
                    .items
                    .push(Item::Obj { buf: idx, open: ev });
                stack.push(idx);
                i += 1;
            }
            Event::Key(k) => {
                let value = src.get(i + 1).copied();
                match value {
                    // Key + compound opener.
                    Some(open @ Event::BeginArray) => {
                        // Arrays never merge: keys cannot repeat inside
                        // an array frame and array-valued keys are
                        // rejected as duplicates by the parser (§ 6.3),
                        // so this is always fresh. No `seen` entry.
                        let idx = bufs.len();
                        bufs.push(new_buf(bump));
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        bufs[stack_top(&stack)]
                            .items
                            .push(Item::Obj { buf: idx, open });
                        stack.push(idx);
                        i += 2;
                    }
                    Some(open @ Event::BeginObject) => {
                        let top = stack_top(&stack);
                        match bufs[top].seen_find(k) {
                            Some(target) => {
                                // § 5.3.2 reopen: the persistent path
                                // table in the parser guarantees a seen
                                // hit is Object-shaped (reopens of
                                // leaf/scalar/array blocks are rejected
                                // as conflicts before any events are
                                // produced). Splice nothing here — the
                                // block's content walks into the target
                                // buffer, and its EndObject just pops
                                // this frame.
                                stack.push(target);
                            }
                            None => {
                                let idx = bufs.len();
                                bufs.push(new_buf(bump));
                                bufs[top].seen_insert(bump, k, idx);
                                bufs[top].items.push(Item::Ev(Event::Key(k)));
                                bufs[top].items.push(Item::Obj { buf: idx, open });
                                stack.push(idx);
                            }
                        }
                        i += 2;
                    }
                    // Key + scalar.
                    Some(scalar) => {
                        debug_assert!(!is_open(&scalar));
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        bufs[stack_top(&stack)].items.push(Item::Ev(scalar));
                        i += 2;
                    }
                    // Malformed stream cannot happen from the parser;
                    // keep the last Key harmlessly in release.
                    None => {
                        bufs[stack_top(&stack)].items.push(Item::Ev(Event::Key(k)));
                        i += 1;
                    }
                }
            }
            // Bare closers close one frame; any other bare event (a
            // scalar array item) lands in the top buffer.
            Event::EndObject | Event::EndArray => {
                stack.pop();
                i += 1;
            }
            ev => {
                bufs[stack_top(&stack)].items.push(Item::Ev(ev));
                i += 1;
            }
        }
    }

    // Flatten depth-first with an explicit work stack.
    let mut out: EventStream<'a> = BumpVec::with_capacity_in(src.len(), bump);
    if let Some(open) = root_open {
        out.push(open);
        let mut work: Vec<Work<'a>> = Vec::with_capacity(16);
        push_buf_items(&mut work, &bufs[0]);
        while let Some(w) = work.pop() {
            match w {
                Work::Ev(ev) => out.push(ev),
                Work::Enter { buf, open, close } => {
                    out.push(open);
                    // Children first, then the matching close: push the
                    // close first so it is popped after the (reversed)
                    // children are exhausted.
                    work.push(Work::Ev(close));
                    push_buf_items(&mut work, &bufs[buf]);
                }
            }
        }
        out.push(matching_close(open));
    }
    out
}

#[cfg(test)]
#[test]
fn empty_child_buffer_has_no_arena_allocation() {
    let bump = Bump::new();
    let before = bump.allocated_bytes();
    let buffer = new_buf(&bump);
    assert_eq!(bump.allocated_bytes(), before);
    assert!(buffer.items.is_empty());
    assert!(buffer.seen.is_empty());
}

fn new_buf<'a>(bump: &'a Bump) -> Buf<'a> {
    Buf {
        items: BumpVec::new_in(bump),
        seen: BumpVec::new_in(bump),
        index: None,
    }
}

#[inline]
fn stack_top(stack: &[usize]) -> usize {
    *stack
        .last()
        .expect("stream is balanced; root frame always open")
}

/// One `&str` equality between two child keys. Counts comparisons in
/// test builds so tests can assert lookup-cost growth deterministically.
#[inline]
pub(super) fn key_eq(a: &str, b: &str) -> bool {
    #[cfg(test)]
    perf::record();
    a == b
}

/// Push `buf`'s items onto the work stack in reverse so they flatten in
/// document order.
fn push_buf_items<'a>(work: &mut Vec<Work<'a>>, b: &Buf<'a>) {
    for item in b.items.iter().rev() {
        work.push(match item {
            Item::Ev(ev) => Work::Ev(*ev),
            Item::Obj { buf, open } => Work::Enter {
                buf: *buf,
                open: *open,
                close: matching_close(*open),
            },
        });
    }
}
