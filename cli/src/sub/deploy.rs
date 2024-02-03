use anyhow::{anyhow, Context as AnyhowContext, Result};
use clap::Subcommand;

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
    },
}

impl DeploySubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let mut context = build_context(cli).await.context("build context")?;

        match self {
            Self::VRC20 {
                name,
                decimals,
                nonce,
                bworkc,
                mint_amount,
                mint_height,
                max_mints,
                meta_data,
            } => {
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

                deploy_vrc20(&mut context, name.clone(), meta).await?;
            }
        }

        context.wallet.flush()?;

        Ok(())
    }
}

async fn deploy_vrc20(context: &mut Context, name: String, meta: VRC20MetaData) -> Result<()> {
    use vital_script_builder::templates;

    // TODO: check input resource
    let name = Name::try_from(name.as_str())
        .with_context(|| format!("the '{}' name format is invalid", name))?;
    let name_resource = Resource::name(name);

    // Got the name resource
    let input_name_utxo = context
        .get_owned_resource(&name_resource)
        .ok_or_else(|| anyhow!("deploy vrc20 need required a name resource by {}", name))?;

    context.append_reveal_input(&[input_name_utxo]);

    // build script.
    // all begin with 0.
    let input_index = 0_u32;
    let scripts_bytes =
        templates::deploy_vrc20(input_index, name, meta).context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
