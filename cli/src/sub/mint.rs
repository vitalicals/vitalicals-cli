use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use clap::Subcommand;
use vital_interfaces_indexer::simulator::SimulatorEnvInterface;
use vital_script::runner::traits::EnvFunctions;
use vital_script_primitives::{resources::Name, traits::EnvContext as EnvContextT};

use crate::{sub::utils::hash_from_str, Cli};

use super::context::{build_context, Context};

#[derive(Debug, Subcommand)]
pub enum MintSubCommands {
    /// Mint Name resource.
    Name {
        /// The name to mint
        name: String,
    },
    /// Mint VRC20 resource by it 's name.
    VRC20 {
        /// The vrc20 's name to mint.
        vrc20_name: String,
    },
    /// Mint VRC721 resource by it 's hash.
    VRC721 { hash: String },
}

impl MintSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let context = build_context(cli).await?;

        match self {
            Self::Name { name } => {
                mint_name(&context, name.clone()).await?;
            }
            Self::VRC20 { vrc20_name } => {
                mint_vrc20(&context, vrc20_name.clone()).await?;
            }
            Self::VRC721 { hash } => {
                mint_vrc721(&context, hash).await?;
            }
        }

        context.wallet.flush()?;

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

async fn mint_vrc20(context: &Context, vrc20_name: String) -> Result<()> {
    use vital_script_builder::templates;

    let name = Name::try_from(vrc20_name.as_str())
        .with_context(|| format!("the vrc20 name {} format invalid", vrc20_name))?;

    let vrc20_metadata = context
        .query_env_context
        .get_vrc20_metadata(name)
        .context("get vrc20 metadata")?
        .ok_or_else(|| anyhow!("not found vrc20 metadata by {}", name))?;

    if vrc20_metadata.mint_count >= vrc20_metadata.meta.mint.max_mints {
        bail!(
            "the vrc20 mint count is {}, and it had reached it 's max mint count {}, so the mint will failed",
            vrc20_metadata.mint_count,  vrc20_metadata.meta.mint.max_mints
        );
    }

    if vrc20_metadata.meta.mint.mint_height > 0 {
        let current_height = context.get_btc_block_height()?;

        if current_height + 1 < vrc20_metadata.meta.mint.mint_height {
            bail!(
                "the vrc20 mint height is {}, and the current height is {}, so the mint will failed",
                vrc20_metadata.meta.mint.mint_height, current_height
            );
        }
    }

    // build script.
    let output_index = 0_u32;
    let scripts_bytes =
        templates::mint_vrc20(output_index, name).context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}

async fn mint_vrc721(context: &Context, hash_str: &str) -> Result<()> {
    use vital_script_builder::templates;

    let hash = hash_from_str(hash_str).context("hash from string")?;

    // check if the hash had mint
    let interface: SimulatorEnvInterface<vital_interfaces_indexer::IndexerClient> =
        SimulatorEnvInterface::new(context.indexer.clone());
    let is_had_mint = interface.vrc721_had_mint(hash).context("vrc721_had_mint")?;
    if is_had_mint {
        bail!("the vrc721 hash `{}` had been minted", hash_str)
    }

    // build script.
    let output_index = 0_u32;
    let scripts_bytes =
        templates::mint_vrc721(output_index, hash).context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
