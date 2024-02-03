mod context;

pub mod deploy;
pub mod mint;
pub mod move_resource;
pub mod query;
pub mod utils;
pub mod wallet;

use anyhow::{Context as AnyhowContext, Result};
use bdk::blockchain::Blockchain;
use btc_p2tr_builder::P2trBuilder;

pub(crate) use context::{build_context, Context};

pub(crate) async fn send_p2tr(context: &Context, scripts_bytes: Vec<u8>) -> Result<()> {
    let wallet = &context.wallet;
    let bdk_blockchain = &wallet.blockchain;

    println!("scripts_bytes {}", hex::encode(&scripts_bytes));

    let builder = P2trBuilder::new(context, scripts_bytes).context("builder build")?;

    let (commit_psbt, reveal_psbt) = builder.build().context("build tx error")?;

    let outpoints_used = commit_psbt
        .unsigned_tx
        .input
        .iter()
        .map(|input| input.previous_output)
        .collect::<Vec<_>>();

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

    if let Err(err) = context.append_used_utxos(&outpoints_used) {
        println!("save used utxos failed by {}, should use --sync to ensure synced", err);
    };

    Ok(())
}
