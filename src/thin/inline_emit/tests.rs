use bumpalo::Bump;

use crate::error::Span;
use crate::parser::inline::InlineBody;

use super::super::event::{Event, EventSink};
use super::perf::take_event_transfers;
use super::scan::scan_inline_events;

/// Collecting sink for direct `scan_inline_events` calls.
struct CollectingSink<'a> {
    events: Vec<Event<'a>>,
}

impl<'a> EventSink<'a> for CollectingSink<'a> {
    fn push(&mut self, event: Event<'a>) {
        self.events.push(event);
    }
}

/// `[[[[…1…]]]]` with `depth` bracket pairs.
fn array_chain(depth: usize) -> String {
    let mut body = String::new();
    body.push_str(&"[".repeat(depth));
    body.push('1');
    body.push_str(&"]".repeat(depth));
    body
}

/// Scan one inline array compound; returns (emitted events, transfers).
fn scan_chain(body: &str) -> (usize, u64) {
    let bump = Bump::new();
    let mut sink = CollectingSink { events: Vec::new() };
    take_event_transfers();
    scan_inline_events(
        body,
        InlineBody::Array,
        1,
        Span::new(0, 0),
        &bump,
        &mut sink,
    )
    .expect("chain must scan");
    (sink.events.len(), take_event_transfers())
}

/// R8-F1 pin: in a chain of D nested arrays every event is moved
/// exactly twice — once into the shared scratch buffer, once into
/// the sink. The pre-fix staged-block path moved D² − 1 extra
/// copies (D² + 4D + 1 transfers total: quadratic); any silent
/// regression to per-item blocks breaks the exact count.
#[test]
fn nested_array_chain_moves_each_event_exactly_twice() {
    for depth in [1usize, 2, 3, 4, 8, 16, 32, 64] {
        let (events, transfers) = scan_chain(&array_chain(depth));
        assert_eq!(events, 2 * depth + 1, "events at depth {depth}");
        assert_eq!(
            transfers,
            (2 * (2 * depth + 1)) as u64,
            "transfers at depth {depth}"
        );
    }
}

/// Growth-shape pin independent of the exact per-event accounting:
/// a 4× depth jump must multiply transfers by ~4 (linear), not
/// ~13–16 (quadratic). Headroom 5× keeps the pin deterministic
/// while a quadratic regression (≥ 13×) fails it loudly.
#[test]
fn nested_array_transfers_grow_linearly_with_depth() {
    let (_, small) = scan_chain(&array_chain(16));
    let (_, large) = scan_chain(&array_chain(64));
    assert_eq!(small, 66);
    assert_eq!(large, 258);
    assert!(
        large < 5 * small,
        "transfers grew super-linearly: {small} -> {large}"
    );
}

/// Positive control for the instrument: object-member arrays are
/// still staged as one block (deliberately — § 5.3.2 dotted
/// re-entry ordering), which shows up as block-build plus
/// block-copy transfers ABOVE the once-into-scratch,
/// once-into-sink floor of 2 × events.
#[test]
fn object_member_array_staging_is_visible_to_the_counter() {
    let bump = Bump::new();
    let mut sink = CollectingSink { events: Vec::new() };
    take_event_transfers();
    scan_inline_events(
        "{a: [1, [2]]}",
        InlineBody::Object,
        1,
        Span::new(0, 0),
        &bump,
        &mut sink,
    )
    .expect("object must scan");
    let events = sink.events.len();
    let transfers = take_event_transfers();
    assert_eq!(events, 9); // BeginObject, Key, 6 array events, EndObject
    assert!(transfers > 2 * events as u64);
}
