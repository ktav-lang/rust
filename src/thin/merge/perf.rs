use std::cell::Cell;

thread_local! {
    static KEY_COMPARISONS: Cell<usize> = const { Cell::new(0) };
}

pub fn record() {
    KEY_COMPARISONS.with(|c| c.set(c.get() + 1));
}

pub fn get() -> usize {
    KEY_COMPARISONS.with(Cell::get)
}

pub fn reset() {
    KEY_COMPARISONS.with(|c| c.set(0));
}
