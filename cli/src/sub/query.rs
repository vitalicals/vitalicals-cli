use anyhow::Result;
use clap::Subcommand;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum QuerySubCommands {
    /// Query vitalicals status.
    FtInfo {
        /// The remote to clone
        id: String,
    },
}

impl QuerySubCommands {
    pub(crate) fn run(&self, _cli: &Cli) -> Result<()> {
        Ok(())
    }
}
