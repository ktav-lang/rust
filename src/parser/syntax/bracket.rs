//! Which kind of bracket we expect to close.

#[derive(Copy, Clone)]
pub(in crate::parser) enum Bracket {
    Object,
    Array,
}

impl Bracket {
    pub(in crate::parser) fn close(self) -> char {
        match self {
            Bracket::Object => '}',
            Bracket::Array => ']',
        }
    }
}
