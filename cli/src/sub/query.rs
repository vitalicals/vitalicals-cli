use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum QuerySubCommands {
    /// Query shadowsats status.
    FtInfo {
        /// The remote to clone
        id: String,
    },
}

impl QuerySubCommands {
    pub fn run(&self) -> Result<()> {
        Ok(())
    }
}
