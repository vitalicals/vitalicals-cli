use anyhow::Result;

use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum MintSubCommands {
    /// Query shadowsats status.
    Query {
        /// The remote to clone
        remote: String,
    },
}

impl MintSubCommands {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}
