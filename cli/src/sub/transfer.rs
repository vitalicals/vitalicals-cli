use anyhow::Result;
use clap::Subcommand;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum TransferSubCommands {
    /// Query vitalicals status.
    Query {
        /// The remote to clone
        remote: String,
    },
}

impl TransferSubCommands {
    pub(crate) async fn run(&self, _cli: &Cli) -> Result<()> {
        Ok(())
    }
}
