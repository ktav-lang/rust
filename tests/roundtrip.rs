//! Round-trip integration tests: `T → text → T` should yield the same `T`.
//! Also hosts the serde behavior parity groups (de, ser, edge cases), the
//! error contract, and the spec-conformance suite.
#[path = "suite/errors.rs"]
mod errors;
#[path = "suite/serde.rs"]
mod serde;
#[path = "suite/spec.rs"]
mod spec_conformance;
