//! The VRC20 Token

use parity_scale_codec::{Decode, Encode};

use super::Tag;
use crate::U256;

#[derive(Default, Clone, Encode, Decode, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct VRC20 {
    pub name: Tag,
    pub amount: U256,
}

impl core::fmt::Display for VRC20 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{},{}]", self.name, self.amount)
    }
}

impl VRC20 {
    pub fn new(name: Tag, amount: U256) -> Self {
        Self { name, amount }
    }
}
