use std::str::FromStr;

use anyhow::{Context, Result};
use bdk::{
    bitcoin::{address::NetworkUnchecked, Address, Network},
    blockchain::Blockchain,
    wallet::AddressIndex,
};
use clap::Subcommand;

use btc_p2tr_builder::P2trBuilder;
use vital_script_primitives::{
    types::{
        vrc20::{VRC20MetaData, VRC20MintMeta},
        MetaData,
    },
    U256,
};

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum DeploySubCommands {
    /// Deploy VRC20 by a name
    VRC20 {
        /// The name for vrc20 to deploy
        name: String,

        /// The decimals for the vrc20
        #[arg(long, default_value = "5")]
        decimals: u8,

        /// The nonce for tx.
        #[arg(long, default_value = "0")]
        nonce: u64,

        /// The bworkc for mint.
        #[arg(long, default_value = "0")]
        bworkc: u64,

        /// The amount for each mint
        mint_amount: u128,

        /// The block height can mint
        #[arg(long, default_value = "0")]
        mint_height: u64,

        /// The max count for mint
        max_mints: u64,

        /// The ext datas for vrc20
        #[arg(long)]
        meta_data: Option<String>,

        /// The bitcoin address to send to.
        #[arg(long)]
        to: Option<String>,

        /// The sat amount in BTC to send.
        #[arg(long, default_value = "1000")]
        amount: u64,

        /// Specify a fee rate in sat/vB.
        #[arg(short, long)]
        fee_rate: Option<f32>,
    },
}

impl DeploySubCommands {
    pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
        let network = cli.network();

        match self {
            Self::VRC20 {
                name,
                decimals,
                nonce,
                to,
                amount,
                fee_rate,
                bworkc,
                mint_amount,
                mint_height,
                max_mints,
                meta_data,
            } => {
                let to_address = to
                    .as_ref()
                    .map(|address| {
                        Address::<NetworkUnchecked>::from_str(address.as_str())
                            .context("parse address failed")?
                            .require_network(network)
                            .context("the address is not for the network")
                    })
                    .transpose()?;

                let meta_data =
                    meta_data.as_ref().map(|data| MetaData { raw: data.as_bytes().to_vec() });

                let meta = VRC20MetaData {
                    decimals: *decimals,
                    nonce: *nonce,
                    bworkc: *bworkc,
                    mint: VRC20MintMeta {
                        mint_amount: *mint_amount,
                        mint_height: *mint_height,
                        max_mints: *max_mints,
                    },
                    meta: meta_data,
                };

                deploy_vrc20(network, cli, name.clone(), meta, to_address, *amount, fee_rate)?;
            }
        }

        Ok(())
    }
}

fn deploy_vrc20(
    network: Network,
    cli: &Cli,
    name: String,
    meta: VRC20MetaData,
    to_address: Option<Address>,
    amount: u64,
    fee_rate: &Option<f32>,
) -> Result<()> {
    use vital_script_builder::templates;

    // TODO: check input resource

    // build script.
    let input_index = 1_u32;
    let scripts_bytes =
        templates::deploy_vrc20(input_index, name, meta).context("build scripts failed")?;

    println!("scripts_bytes {}", hex::encode(&scripts_bytes));

    // build tx
    let wallet = wallet::Wallet::load(network, cli.endpoint.clone(), &cli.datadir)
        .context("load wallet failed")?;
    let bdk_wallet = &wallet.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let to_address = if let Some(to) = to_address {
        to
    } else {
        bdk_wallet.get_address(AddressIndex::New).context("new address")?.address
    };

    let builder = P2trBuilder::new(scripts_bytes, to_address, amount, *fee_rate, &wallet)
        .context("builder build")?;

    let (commit_psbt, reveal_psbt) = builder.build().context("build tx error")?;

    let commit_raw_transaction = commit_psbt.extract_tx();
    let commit_txid = commit_raw_transaction.txid();

    println!("tx: {}", serde_json::to_string_pretty(&reveal_psbt.unsigned_tx).expect("to"));

    let reveal_raw_transaction = reveal_psbt.extract_tx();

    println!("tx: {}", serde_json::to_string_pretty(&reveal_raw_transaction).expect("to"));

    let reveal_txid = reveal_raw_transaction.txid();

    bdk_blockchain.broadcast(&commit_raw_transaction)?;
    println!("Commit Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = commit_txid);

    bdk_blockchain.broadcast(&reveal_raw_transaction)?;
    println!("Reveal Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = reveal_txid);

    Ok(())
}
