use std::str::FromStr;

use anyhow::{Context, Result};
use bdk::{
	bitcoin::{address::NetworkUnchecked, Address, Network},
	blockchain::Blockchain,
	FeeRate, SignOptions,
};
use clap::Subcommand;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum UtilsSubCommands {
	/// Send an amount to a given address.
	SendToAddress {
		/// The bitcoin address to send to.
		address: String,

		/// The sat amount in BTC to send.
		amount: u64,

		/// Specify a fee rate in sat/vB.
		#[arg(short, long)]
		fee_rate: Option<f32>,

		/// Signal that this transaction can be replaced by a transaction (BIP 125).
		#[arg(long)]
		replaceable: bool,
	},
}

impl UtilsSubCommands {
	pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
		let network = cli.network();

		match self {
			Self::SendToAddress { address, amount, fee_rate, replaceable } => {
				let address = Address::<NetworkUnchecked>::from_str(address)
					.context("parse address failed")?
					.require_network(network)
					.context("the address is not for the network")?;

				send_to_address(network, cli, address, *amount, fee_rate, *replaceable)
			},
		}
	}
}

fn send_to_address(
	network: Network,
	cli: &Cli,
	address: Address,
	amount: u64,
	fee_rate: &Option<f32>,
	replaceable: bool,
) -> Result<()> {
	let wallet = wallet::Wallet::load(network, cli.endpoint.clone(), &cli.datadir)
		.context("load wallet failed")?;
	let bdk_wallet = &wallet.wallet;
	let bdk_blockchain = &wallet.blockchain;

	let mut builder = bdk_wallet.build_tx();
	builder.set_recipients(vec![(address.script_pubkey(), amount)]);

	if replaceable {
		builder.enable_rbf();
	}

	if let Some(fee_rate) = fee_rate {
		builder.fee_rate(FeeRate::from_sat_per_vb(*fee_rate));
	}

	let (mut psbt, details) = builder.finish().context("build tx failed")?;
	println!("Transaction details: {:#?}", details);
	println!("Unsigned PSBT: {}", serde_json::to_string_pretty(&psbt)?);
	println!("Unsigned PSBT: {}", psbt);

	// Sign and finalize the PSBT with the signing wallet
	bdk_wallet.sign(&mut psbt, SignOptions::default())?;

	bdk_wallet.finalize_psbt(&mut psbt, SignOptions::default())?;

	println!("Signed PSBT: {}", serde_json::to_string_pretty(&psbt)?);
	println!("Signed PSBT: {}", psbt);

	// Broadcast the transaction
	let raw_transaction = psbt.extract_tx();
	let txid = raw_transaction.txid();

	bdk_blockchain.broadcast(&raw_transaction)?;
	println!("Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = txid);

	Ok(())
}
