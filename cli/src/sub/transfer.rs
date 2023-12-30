use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum TransferSubCommands {
    /// Query shadowsats status.
    Query {
        /// The remote to clone
        remote: String,
    },
}

impl TransferSubCommands {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}
