use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use bdk::{
    bitcoin::{
        absolute,
        address::{NetworkUnchecked, Payload},
        key::TapTweak,
        psbt::{Input, PartiallySignedTransaction, PsbtSighashType},
        secp256k1::{All, KeyPair, Secp256k1, SecretKey, XOnlyPublicKey},
        sighash::{self, SighashCache, TapSighash, TapSighashType},
        taproot::{self, LeafVersion, TapLeafHash, TaprootBuilder},
        Address, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    blockchain::Blockchain,
    wallet::AddressIndex,
    FeeRate, SignOptions,
};
use clap::Subcommand;

use btc_p2tr_builder::P2trBuilder;
use btc_script_builder::InscriptionScriptBuilder;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum MintSubCommands {
    /// Query vitalicals status.
    Name {
        /// The name to mint
        name: String,

        /// The bitcoin address to send to.
        #[arg(long)]
        to: Option<String>,

        /// The sat amount in BTC to send.
        amount: u64,

        /// Specify a fee rate in sat/vB.
        #[arg(short, long)]
        fee_rate: Option<f32>,
    },
}

impl MintSubCommands {
    pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
        let network = cli.network();

        match self {
            Self::Name { name, to, amount, fee_rate } => {
                let to_address = to
                    .as_ref()
                    .map(|address| {
                        Address::<NetworkUnchecked>::from_str(address.as_str())
                            .context("parse address failed")?
                            .require_network(network)
                            .context("the address is not for the network")
                    })
                    .transpose()?;

                mint_name(network, cli, name.clone(), to_address, *amount, fee_rate)?;
            }
        }

        Ok(())
    }
}

fn mint_name(
    network: Network,
    cli: &Cli,
    name: String,
    to_address: Option<Address>,
    amount: u64,
    fee_rate: &Option<f32>,
) -> Result<()> {
    use vital_script_builder::templates;

    // build script.
    let output_index = 0_u32;
    let scripts_bytes = templates::mint_name(output_index, name).context("build scripts failed")?;

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

    let reveal_raw_transaction = reveal_psbt.extract_tx();
    let reveal_txid = reveal_raw_transaction.txid();

    bdk_blockchain.broadcast(&commit_raw_transaction)?;
    println!("Commit Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = commit_txid);

    bdk_blockchain.broadcast(&reveal_raw_transaction)?;
    println!("Reveal Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = reveal_txid);

    Ok(())
}
