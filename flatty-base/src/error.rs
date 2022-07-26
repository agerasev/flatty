#[derive(Clone, PartialEq, Debug)]
pub enum Error {
    InsufficientSize,
    BadAlign,
    InvalidState,
}
