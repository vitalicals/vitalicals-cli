use anyhow::{Context as AnyhowContext, Result};
use clap::Subcommand;

use crate::{build_context, Cli};

mod name;
use name::*;

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
        }

        Ok(())
    }
}
