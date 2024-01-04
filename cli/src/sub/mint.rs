use anyhow::Result;
use clap::Subcommand;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum MintSubCommands {
    /// Query vitalicals status.
    Query {
        /// The remote to clone
        remote: String,
    },
}

impl MintSubCommands {
    pub(crate) fn run(&self, _cli: &Cli) -> Result<()> {
        Ok(())
    }
}
