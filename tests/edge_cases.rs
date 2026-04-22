//! Tricky and combinatorial edge-cases. Kept in a single test binary so
//! regressions are obvious at a glance.

#[path = "edge_cases/paren_literals.rs"]
mod paren_literals;

#[path = "edge_cases/keywords_in_maps.rs"]
mod keywords_in_maps;

#[path = "edge_cases/deep_nesting.rs"]
mod deep_nesting;

#[path = "edge_cases/multiline_combinations.rs"]
mod multiline_combinations;

#[path = "edge_cases/options_and_emptiness.rs"]
mod options_and_emptiness;

#[path = "edge_cases/double_round_trip.rs"]
mod double_round_trip;

#[path = "edge_cases/special_strings.rs"]
mod special_strings;

#[path = "edge_cases/typed_markers.rs"]
mod typed_markers;
