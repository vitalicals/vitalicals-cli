use bdk::bitcoin::{blockdata::block::Header, Amount, BlockHash, ScriptHash, Txid};

use crate::{Id, LocationRef};

pub enum ResourceType {
	FT,
}

pub struct ResourceBaseInfo {
	id: Id,
	number: u64,
	resource_ref: LocationRef,
	resource_type: ResourceType,
	confirmed: bool,
}

pub struct ResourceMintArgs {
	time: u64,
	nonce: u64,
	bitworkc: String,
	max_mints: Amount,
	mint_amount: Amount,
	mint_height: u64,
	mint_bitworkc: String,
	request_ticker: String,
}

pub struct ResourceMintInfo {
	commit_txid: Txid,
	commit_index: u32,
	commit_location: LocationRef,
	commit_tx_num: u32,
	commit_height: u64,
	reveal_location_txid: Txid,
	reveal_location_index: u32,
	reveal_location: LocationRef,
	reveal_location_tx_num: u32,
	reveal_location_height: u64,
	reveal_location_header: Header,
	reveal_location_blockhash: BlockHash,
	reveal_location_scripthash: ScriptHash,
	reveal_location_script: Vec<u8>,
	reveal_location_value: Amount,
	args: ResourceMintArgs,
}
