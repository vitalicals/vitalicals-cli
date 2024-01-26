//! The VRC721 Token

use parity_scale_codec::{Decode, Encode};

use super::Tag;
use crate::H256;

#[derive(Default, Clone, Encode, Decode, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VRC721 {
    pub name: Tag,
    pub hash: H256,
}

impl core::fmt::Display for VRC721 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{},{}]", self.name, self.hash)
    }
}

impl VRC721 {
    pub fn new(name: Tag, hash: H256) -> Self {
        Self { name, hash }
    }
}
