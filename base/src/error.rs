#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub pos: usize,
}

/// Error that can occur while working with flat types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The byte slice has insufficient size to be interpreted as required type.
    InsufficientSize,
    /// Memory isn't properly aligned to be interpreted as required type.
    BadAlign,
    /// Enum binary representation contains index that doesn't match any of possible enum states.
    InvalidEnumState,
    /// Any other error.
    Other,
}

impl Error {
    pub fn offset(mut self, offset: usize) -> Self {
        self.pos += offset;
        self
    }
}
