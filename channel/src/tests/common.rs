use flatty::{flat, portable::le, FlatVec};

#[flat(sized = false, portable = true, default = true)]
pub enum TestMsg {
    #[default]
    A,
    B(le::I32),
    C(FlatVec<le::I32, le::U16>),
}
