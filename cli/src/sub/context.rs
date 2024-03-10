use anyhow::{Context as AnyhowContext, Result};

pub use client::context::Context;

use crate::Cli;

pub async fn build_context(cli: &Cli) -> Result<Context> {
    let network = cli.network();

    let wallet = wallet::Wallet::load(
        network,
        cli.endpoint.clone(),
        &cli.datadir,
        &cli.wallet,
        !cli.no_sync,
    )
    .context("load wallet failed")?;

    let context = Context::new(cli.datadir.clone(), &cli.indexer, wallet)
        .await?
        .with_fee_rate(&cli.fee_rate)
        .with_replaceable(&cli.replaceable)
        .with_to_address(&cli.to)
        .context("with address")?
        .with_sats_amount(cli.sats);

    Ok(context)
}
