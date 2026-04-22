// Shared helpers for integration tests.

pub const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

pub fn fixture(name: &str) -> String {
    format!("{}/{}", FIXTURES, name)
}
