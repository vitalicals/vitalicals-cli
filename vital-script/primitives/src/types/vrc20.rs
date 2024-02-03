use parity_scale_codec::{Decode, Encode};

use super::MetaData;

/// The mint meta data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VRC20MintMeta {
    pub mint_amount: u128,
    pub mint_height: u32,
    pub max_mints: u64,
}

/// The meta data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VRC20MetaData {
    pub decimals: u8,
    pub nonce: u64,
    pub bworkc: u64,
    pub mint: VRC20MintMeta,
    pub meta: Option<MetaData>,
}

/// The status data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VRC20StatusData {
    pub mint_count: u64,
    pub meta: VRC20MetaData,
}
