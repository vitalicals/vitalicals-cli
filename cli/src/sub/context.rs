use anyhow::{Context as AnyhowContext, Result};

use client::context::Context;

use crate::Cli;

pub async fn build_context(cli: &Cli) -> Result<Context> {
    let network = cli.network();

    let wallet = wallet::Wallet::load(network, cli.endpoint.clone(), &cli.datadir)
        .context("load wallet failed")?;

    let context = Context::new(cli.datadir.clone(), &cli.indexer, wallet).await?;

    Ok(context)
}
