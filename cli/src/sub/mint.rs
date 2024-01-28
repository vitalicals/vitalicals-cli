use anyhow::{Context as AnyhowContext, Result};
use bdk::{blockchain::Blockchain, wallet::AddressIndex};
use clap::Subcommand;

use btc_p2tr_builder::P2trBuilder;

use crate::Cli;

use super::context::{build_context, Context};

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
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::Name { name, to, amount, fee_rate } => {
                let context = build_context(cli)
                    .await?
                    .with_to_address(to)
                    .context("with to address failed")?;

                mint_name(&context, name.clone(), *amount, fee_rate).await?;
            }
        }

        Ok(())
    }
}

async fn mint_name(
    context: &Context,
    name: String,
    amount: u64,
    fee_rate: &Option<f32>,
) -> Result<()> {
    use vital_script_builder::templates;

    // build script.
    let output_index = 0_u32;
    let scripts_bytes = templates::mint_name(output_index, name).context("build scripts failed")?;

    println!("scripts_bytes {}", hex::encode(&scripts_bytes));

    // build tx
    let wallet = &context.wallet;
    let bdk_wallet = &wallet.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let to_address = if let Some(to) = context.to_address.clone() {
        to
    } else {
        bdk_wallet.get_address(AddressIndex::New).context("new address")?.address
    };

    let utxo_with_resources =
        context.utxo_with_resources().await.context("utxo_with_resources failed")?;

    let builder =
        P2trBuilder::new(scripts_bytes, to_address, amount, *fee_rate, wallet, utxo_with_resources)
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
