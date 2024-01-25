//! The types

use alloc::vec::Vec;

use parity_scale_codec::{Decode, Encode};

pub mod vrc20;

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct MetaData {
    raw: Vec<u8>,
}
