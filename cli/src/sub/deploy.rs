use anyhow::{anyhow, Context as AnyhowContext, Result};
use bdk::blockchain::Blockchain;
use clap::Subcommand;

use btc_p2tr_builder::P2trBuilder;
use vital_script_primitives::{
    resources::{Name, Resource},
    types::{
        vrc20::{VRC20MetaData, VRC20MintMeta},
        MetaData,
    },
};

use crate::{build_context, Cli, Context};

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
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let context = build_context(cli).await.context("build context")?;

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
                let context = context
                    .with_to_address(to, *amount)
                    .context("with to address")?
                    .with_fee_rate(fee_rate);

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

                deploy_vrc20(&context, name.clone(), meta).await?;
            }
        }

        Ok(())
    }
}

async fn deploy_vrc20(context: &Context, name: String, meta: VRC20MetaData) -> Result<()> {
    use vital_script_builder::templates;

    // TODO: check input resource
    let name = Name::try_from(name.as_str())
        .with_context(|| format!("the '{}' name format is invalid", name))?;
    let name_resource = Resource::name(name);

    // Got the name resource
    let input_name_utxo = context
        .get_owned_resource(&name_resource)
        .ok_or_else(|| anyhow!("deploy vrc20 need required a name resource by {}", name))?;

    // build script.
    // all begin with 0.
    let input_index = 0_u32;
    let scripts_bytes =
        templates::deploy_vrc20(input_index, name, meta).context("build scripts failed")?;

    println!("scripts_bytes {}", hex::encode(&scripts_bytes));

    // build tx
    let wallet = &context.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let builder = P2trBuilder::new(context, scripts_bytes)
        .context("builder build")?
        .with_reveal_input(input_name_utxo);

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
