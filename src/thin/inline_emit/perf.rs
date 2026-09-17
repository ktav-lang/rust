// Deterministic instrument for the shared-buffer property: counts
// every `Event` value the scanner moves into a scratch buffer or
// sink, plus the block copies between buffers that nested-array
// staging used to make quadratic (3 + 5 + … + (2D−1) = D² − 1 extra
// copies for a chain of D nested arrays). A parse that lands every
// event in the shared compound scratch exactly once counts each
// event twice — once into the scratch, once into the sink.
// Accumulated in test builds only; in non-test builds
// `record_event_transfers` is a no-op and the counters optimize
// away.
#[cfg(test)]
thread_local! {
    static EVENT_TRANSFERS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Add `n` moved/copied events to this thread's total (test builds only).
#[cfg(test)]
pub(super) fn record_event_transfers(n: u64) {
    EVENT_TRANSFERS.with(|total| total.set(total.get() + n));
}

/// Read and reset this thread's moved/copied-event total.
#[cfg(test)]
pub(crate) fn take_event_transfers() -> u64 {
    EVENT_TRANSFERS.with(|total| {
        let n = total.get();
        total.set(0);
        n
    })
}

#[cfg(not(test))]
#[inline(always)]
pub(super) fn record_event_transfers(n: u64) {
    let _ = n;
}
