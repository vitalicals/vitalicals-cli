use bdk::bitcoin::{blockdata::block::Header, Amount, BlockHash, ScriptHash, Txid};

use crate::{Id, LocationRef};

pub enum ResourceType {
    FT,
}

pub struct ResourceBaseInfo {
    pub id: Id,
    pub number: u64,
    pub resource_ref: LocationRef,
    pub resource_type: ResourceType,
    pub confirmed: bool,
}

pub struct ResourceMintArgs {
    pub time: u64,
    pub nonce: u64,
    pub bitworkc: String,
    pub max_mints: Amount,
    pub mint_amount: Amount,
    pub mint_height: u64,
    pub mint_bitworkc: String,
    pub request_ticker: String,
}

pub struct ResourceMintInfo {
    pub commit_txid: Txid,
    pub commit_index: u32,
    pub commit_location: LocationRef,
    pub commit_tx_num: u32,
    pub commit_height: u64,
    pub reveal_location_txid: Txid,
    pub reveal_location_index: u32,
    pub reveal_location: LocationRef,
    pub reveal_location_tx_num: u32,
    pub reveal_location_height: u64,
    pub reveal_location_header: Header,
    pub reveal_location_blockhash: BlockHash,
    pub reveal_location_scripthash: ScriptHash,
    pub reveal_location_script: Vec<u8>,
    pub reveal_location_value: Amount,
    pub args: ResourceMintArgs,
}
