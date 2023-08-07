/// Flat type operation error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    /// An offset from the beginning of flat type memory where error occur.
    pub pos: usize,
}

/// Error kinds that can occur while working with flat types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// The byte slice has insufficient size to be interpreted as required type.
    InsufficientSize,
    /// Memory isn't properly aligned to be interpreted as required type.
    BadAlign,
    /// Enum binary representation contains index that doesn't match any of possible enum states.
    InvalidEnumTag,
    /// Invalid binary representation of the type.
    InvalidData,
    /// Any other error.
    Other,
}

impl Error {
    /// Clone `self` and add `offset` to [`Self::pos`].
    pub fn offset(mut self, offset: usize) -> Self {
        self.pos += offset;
        self
    }
}
