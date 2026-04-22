//! Round-trip integration tests: `T → text → T` should yield the same `T`.

#[path = "roundtrip/keywords.rs"]
mod keywords;
#[path = "roundtrip/multiline.rs"]
mod multiline;
#[path = "roundtrip/scalars.rs"]
mod scalars;
#[path = "roundtrip/structures.rs"]
mod structures;
#[path = "roundtrip/typed.rs"]
mod typed;
