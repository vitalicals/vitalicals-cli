//! The VRC721 Token

use crate::H256;
use parity_scale_codec::{Decode, Encode};

#[derive(Default, Clone, Encode, Decode, Debug, PartialOrd, Ord, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VRC721 {
    pub hash: H256,
}

impl core::fmt::Display for VRC721 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VRC721[{}]", self.hash)
    }
}

impl VRC721 {
    pub fn new(hash: H256) -> Self {
        Self { hash }
    }
}
