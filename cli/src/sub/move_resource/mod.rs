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
    },
    /// Move name to outpoint.
    Names {
        /// The name to move.
        names: Vec<String>,
    },
    /// Move vrc20 to outpoint with charge.
    VRC20 {
        /// The name of vrc20
        name: String,
        /// The amount
        amount: u128,
    },
}

impl MoveSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let mut context = build_context(cli).await.context("build context")?;

        match self {
            MoveSubCommands::Name { name } => {
                move_names(&mut context, &[name.clone()]).await?;
            }
            MoveSubCommands::Names { names } => {
                move_names(&mut context, names).await?;
            }
            MoveSubCommands::VRC20 { name, amount } => {
                move_vrc20(&mut context, name, U256::from(*amount)).await?;
            }
        }

        context.wallet.flush()?;

        Ok(())
    }
}
