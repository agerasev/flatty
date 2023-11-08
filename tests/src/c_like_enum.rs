#![allow(dead_code)]

use flatty::flat;

#[derive(Clone, Debug, Default)]
#[flat]
pub enum TagOnly {
    #[default]
    A,
    B,
}
