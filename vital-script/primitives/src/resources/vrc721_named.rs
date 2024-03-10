//! The Named VRC721 Token

use crate::H256;
use parity_scale_codec::{Decode, Encode};

use super::Tag;

#[derive(Default, Clone, Encode, Decode, Debug, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NVRC721 {
    pub name: Tag,
    pub hash: H256,
}

impl core::fmt::Display for NVRC721 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VRC721[{},{}]", self.name, self.hash)
    }
}

impl NVRC721 {
    pub fn new(name: Tag, hash: H256) -> Self {
        Self { name, hash }
    }
}
