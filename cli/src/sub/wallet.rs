use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum WalletSubCommands {
    /// Query shadowsats status.
    Query {
        /// The remote to clone
        remote: String,
    },
}

impl WalletSubCommands {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}
