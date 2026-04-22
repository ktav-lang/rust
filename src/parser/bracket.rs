//! Which kind of bracket we expect to close.

#[derive(Copy, Clone)]
pub(super) enum Bracket {
    Object,
    Array,
}

impl Bracket {
    pub(super) fn open(self) -> char {
        match self {
            Bracket::Object => '{',
            Bracket::Array => '[',
        }
    }

    pub(super) fn close(self) -> char {
        match self {
            Bracket::Object => '}',
            Bracket::Array => ']',
        }
    }
}
