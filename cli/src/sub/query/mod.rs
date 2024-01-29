use anyhow::Result;
use clap::Subcommand;

mod resources;
mod vrc20;

use crate::Cli;

use self::{resources::QueryResources, vrc20::QueryVrc20Metadata};

#[derive(Debug, Subcommand)]
pub enum QuerySubCommands {
    /// Query status.
    FtInfo {
        /// The remote to clone
        id: String,
    },
    /// Query resources.
    Resources(QueryResources),
    /// Query vrc20 metadata
    Vrc20Metadata(QueryVrc20Metadata),
}

impl QuerySubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::FtInfo { id: _id } => {
                todo!();
            }
            Self::Resources(q) => q.run(cli).await?,
            Self::Vrc20Metadata(q) => q.run(cli).await?,
        }

        Ok(())
    }
}
