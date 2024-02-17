//! The VRC721 Token

use crate::H256;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct VRC721 {
    pub hash: H256,
}

impl VRC721 {
    pub fn new(hash: H256) -> Self {
        Self { hash }
    }
}
