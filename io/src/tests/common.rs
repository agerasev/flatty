use flatty::{flat, FlatVec};

#[flat(sized = false, default = true)]
pub enum TestMsg {
    #[default]
    A,
    B(i32),
    C(FlatVec<i32, u16>),
}
