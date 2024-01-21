use primitive_types::U256;

use parity_scale_codec::{Decode, Encode};

use super::MetaData;

/// The mint meta data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct VRC20MintMeta {
    pub mint_type: u8,
    pub mint_amount: U256,
    pub mint_height: u64,
    pub max_mints: u64,
}

/// The meta data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct VRC20MetaData {
    pub decimals: u8,
    pub nonce: u64,
    pub bworkc: u64,
    pub max: U256,
    pub mint: VRC20MintMeta,
    pub meta: Option<MetaData>,
}
