#[derive(Clone, PartialEq, Debug)]
pub enum InterpretError {
    InsufficientSize,
    BadAlign,
    InvalidState,
}
