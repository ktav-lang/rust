//! Tricky and combinatorial edge-cases. Kept in a single test binary so
//! regressions are obvious at a glance.
#[path = "edge_cases/literals.rs"]
mod literals;
#[path = "edge_cases/nesting.rs"]
mod nesting;
#[path = "edge_cases/values.rs"]
mod values;
