//! The VRC20 Token

use super::Tag;
use crate::{names::Name, U256};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct VRC20 {
    pub name: Tag,
    pub amount: U256,
}

impl VRC20 {
    pub fn new(name: Tag, amount: U256) -> Self {
        Self { name, amount }
    }
}
