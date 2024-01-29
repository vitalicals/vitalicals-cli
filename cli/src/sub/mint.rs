use anyhow::{Context as AnyhowContext, Result};
use clap::Subcommand;

use crate::Cli;

use super::context::{build_context, Context};

#[derive(Debug, Subcommand)]
pub enum MintSubCommands {
    /// Query vitalicals status.
    Name {
        /// The name to mint
        name: String,

        /// The sat amount in BTC to send.
        amount: u64,
    },
}

impl MintSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::Name { name, amount } => {
                let context = build_context(cli).await?.with_amount(*amount);

                mint_name(&context, name.clone()).await?;
            }
        }

        Ok(())
    }
}

async fn mint_name(context: &Context, name: String) -> Result<()> {
    use vital_script_builder::templates;

    // build script.
    let output_index = 0_u32;
    let scripts_bytes = templates::mint_name(output_index, name).context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
