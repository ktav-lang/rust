//! R8-F6/R10-F1 measurement-only counters (test-only; the probe surface used by the ix_probe tests).

use std::cell::RefCell;

#[derive(Clone, Copy, Default)]
struct State {
    bypass: bool,
    kc_calls: u64,
    kc_hits: u64,
    kc_steps: u64,
    kc_steps_max: u64,
    oca_calls: u64,
    oca_hits: u64,
    oca_steps: u64,
    oca_steps_max: u64,
    sort_calls: u64,
    sort_elems: u64,
    sort_cmps: u64,
    bodies: u64,
    body_bytes: u64,
    pairs_total: u64,
    pairs_max: u64,
    hq_calls: u64,
    hq_bytes: u64,
    hq_max: u64,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// RAII: restores `bypass = false` when dropped (even on unwind),
/// so the bypassed mode cannot leak into another test sharing this
/// thread under `--test-threads=1`.
pub(crate) struct BypassGuard;

impl Drop for BypassGuard {
    fn drop(&mut self) {
        with_state(|s| s.bypass = false);
    }
}

/// R8-F6 A/B switch: when set, both lookup sites return `None`
/// without searching, forcing callers down the live-dispatch
/// fallback (the memo is a proven pure memo — parser::tests
/// `memo_bounds_are_a_pure_memo_of_the_live_dispatches` — so the
/// parse result is byte-identical; only the index is skipped).
/// Recording and sorting still happen. NOTE (R9-F3): this measures
/// the memo's benefit (cached vs live re-scan), not the isolated
/// cost of the binary search. Returns an RAII guard restoring
/// `bypass = false` on drop.
pub(crate) fn set_bypass(on: bool) -> BypassGuard {
    with_state(|s| s.bypass = on);
    BypassGuard
}

pub(crate) fn bypass_engaged() -> bool {
    with_state(|s| s.bypass)
}

/// One lookup's binary-search step count (loop iterations = element
/// comparisons) plus whether it was a call/hit at all.
pub(crate) fn record_kc_lookup(steps: usize) {
    with_state(|s| {
        s.kc_calls += 1;
        s.kc_steps += steps as u64;
        s.kc_steps_max = s.kc_steps_max.max(steps as u64);
    });
}

pub(crate) fn record_kc_hit() {
    with_state(|s| s.kc_hits += 1);
}

pub(crate) fn record_oca_lookup(steps: usize) {
    with_state(|s| {
        s.oca_calls += 1;
        s.oca_steps += steps as u64;
        s.oca_steps_max = s.oca_steps_max.max(steps as u64);
    });
}

pub(crate) fn record_oca_hit() {
    with_state(|s| s.oca_hits += 1);
}

pub(crate) fn record_sort(elems: usize, cmps: usize) {
    with_state(|s| {
        s.sort_calls += 1;
        s.sort_elems += elems as u64;
        s.sort_cmps += cmps as u64;
    });
}

pub(crate) fn record_body(input_len: usize, pairs: usize) {
    with_state(|s| {
        s.bodies += 1;
        s.body_bytes += input_len as u64;
        s.pairs_total += pairs as u64;
        s.pairs_max = s.pairs_max.max(pairs as u64);
    });
}

/// R10-F1: one quote-presence prescan (`has_quote_bytes`): `len` is
/// the slice length handed to the check (up to three `contains`
/// passes each). These are the PREscans — the repeated full-subtree
/// quote checks this module's lookup counters do not see.
pub(crate) fn record_quote_prescan(len: usize) {
    with_state(|s| {
        s.hq_calls += 1;
        s.hq_bytes += len as u64;
        s.hq_max = s.hq_max.max(len as u64);
    });
}

#[derive(Default, Clone, Copy)]
pub(crate) struct Snapshot {
    pub kc_calls: u64,
    pub kc_hits: u64,
    pub kc_steps: u64,
    pub kc_steps_max: u64,
    pub oca_calls: u64,
    pub oca_hits: u64,
    pub oca_steps: u64,
    pub oca_steps_max: u64,
    pub sort_calls: u64,
    pub sort_elems: u64,
    pub sort_cmps: u64,
    pub bodies: u64,
    pub body_bytes: u64,
    pub pairs_total: u64,
    pub pairs_max: u64,
    pub hq_calls: u64,
    pub hq_bytes: u64,
    pub hq_max: u64,
}

pub(crate) fn snapshot() -> Snapshot {
    STATE.with(|s| {
        let s = s.borrow();
        Snapshot {
            kc_calls: s.kc_calls,
            kc_hits: s.kc_hits,
            kc_steps: s.kc_steps,
            kc_steps_max: s.kc_steps_max,
            oca_calls: s.oca_calls,
            oca_hits: s.oca_hits,
            oca_steps: s.oca_steps,
            oca_steps_max: s.oca_steps_max,
            sort_calls: s.sort_calls,
            sort_elems: s.sort_elems,
            sort_cmps: s.sort_cmps,
            bodies: s.bodies,
            body_bytes: s.body_bytes,
            pairs_total: s.pairs_total,
            pairs_max: s.pairs_max,
            hq_calls: s.hq_calls,
            hq_bytes: s.hq_bytes,
            hq_max: s.hq_max,
        }
    })
}

pub(crate) fn reset() {
    with_state(|s| *s = State::default());
}
