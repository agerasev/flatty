pub struct Error {
    pub kind: ErrorKind,
    pub position: usize,
}

/// Error that can occur while working with flat types.
#[derive(Clone, PartialEq, Debug)]
pub enum ErrorKind {
    /// The byte slice has insufficient size to be interpreted as required type.
    InsufficientSize { actual: usize, required: usize },
    /// Memory isn't properly aligned to be interpreted as required type.
    BadAlign { required: usize },
    /// Enum binary representation contains index that doesn't match any of possible enum states.
    InvalidEnumState { bad_state: usize },
}
