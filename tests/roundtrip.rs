//! Round-trip integration tests: `T → text → T` should yield the same `T`.
#[path = "roundtrip/edge_cases.rs"]
mod edge_cases;
#[path = "roundtrip/errors.rs"]
mod errors;
#[path = "roundtrip/serde.rs"]
mod serde;
