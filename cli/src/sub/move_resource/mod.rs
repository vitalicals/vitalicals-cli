use anyhow::Result;
use clap::Subcommand;

use crate::Cli;

mod name;

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
    pub(crate) async fn run(&self, _cli: &Cli) -> Result<()> {
        Ok(())
    }
}
