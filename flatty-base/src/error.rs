/// Error that can occur while working with flat types.
#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    /// The byte slice has insufficient size to be interpreted as required type.
    InsufficientSize,
    /// Memory isn't properly aligned to be inerpreted as required type.
    BadAlign,
    /// Enum binary representation contains index that doesn't match any of possible enum states.
    InvalidState,
}
