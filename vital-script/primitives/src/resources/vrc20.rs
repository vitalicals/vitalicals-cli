//! The VRC20 Token

use parity_scale_codec::{Decode, Encode};

use super::Tag;
use crate::U256;

#[derive(Default, Clone, Encode, Decode, Debug, PartialEq, Eq)]
pub struct VRC20 {
    pub name: Tag,
    pub amount: U256,
}

impl VRC20 {
    pub fn new(name: Tag, amount: U256) -> Self {
        Self { name, amount }
    }
}
