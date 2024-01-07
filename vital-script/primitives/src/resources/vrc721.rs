//! The VRC721 Token

use super::Tag;
use crate::{names::Name, H256};

#[derive(Default, Debug, PartialEq, Eq)]
pub struct VRC721 {
    pub name: Tag,
    pub hash: H256,
}

impl VRC721 {
    pub fn new(name: Tag, hash: H256) -> Self {
        Self { name, hash }
    }
}
