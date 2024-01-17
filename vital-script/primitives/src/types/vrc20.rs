use primitive_types::U256;

use parity_scale_codec::{Decode, Encode};

/// The mint meta data for vrc20
#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
pub struct VRC20MintMeta {
    pub mint_type: u8,
    pub mint_amount: U256,
    pub mint_height: u64,
    pub max_mints: u64,
}
