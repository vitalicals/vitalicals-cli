//! The VRC20 Token

use crate::{names::Name, U256};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct VRC20 {
    pub name: Name,
    pub amount: U256,
}
