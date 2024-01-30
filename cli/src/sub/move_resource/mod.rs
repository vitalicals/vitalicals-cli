use anyhow::{Context as AnyhowContext, Result};
use clap::Subcommand;
use vital_script_primitives::U256;

use crate::{build_context, Cli};

mod name;
mod vrc20;

use name::*;
use vrc20::*;

#[derive(Debug, Subcommand)]
pub enum MoveSubCommands {
    /// Move name to outpoint.
    Name {
        /// The name to move.
        name: String,
        /// The amount for output.
        amount: u64,
    },
    /// Move name to outpoint.
    Names {
        /// The name to move.
        names: Vec<String>,
        /// The amount for each output.
        amount: u64,
    },
    /// Move vrc20 to outpoint with charge.
    VRC20 {
        /// The name of vrc20
        name: String,
        /// The amount
        amount: u128,
        /// The btc sats for output
        #[arg(long, default_value = "1000")]
        sats: u64,
    },
}

impl MoveSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let mut context = build_context(cli).await.context("build context")?;

        match self {
            MoveSubCommands::Name { name, amount } => {
                move_names(&mut context, &[name.clone()], *amount).await?;
            }
            MoveSubCommands::Names { names, amount } => {
                move_names(&mut context, &names, *amount).await?;
            }
            MoveSubCommands::VRC20 { name, amount, sats } => {
                move_vrc20(&mut context, name, U256::from(*amount), *sats).await?;
            }
        }

        Ok(())
    }
}
