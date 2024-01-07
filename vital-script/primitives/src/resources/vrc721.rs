//! The VRC721 Token

use crate::{names::Name, H256};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct VRC721 {
    pub name: Name,
    pub hash: H256,
}
