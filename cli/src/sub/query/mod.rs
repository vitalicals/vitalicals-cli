use anyhow::Result;
use clap::Subcommand;

mod resources;

use crate::Cli;

use self::resources::QueryResources;

#[derive(Debug, Subcommand)]
pub enum QuerySubCommands {
    /// Query status.
    FtInfo {
        /// The remote to clone
        id: String,
    },
    /// Query resources.
    Resources(QueryResources),
}

impl QuerySubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::FtInfo { id: _id } => {
                todo!();
            }
            Self::Resources(q) => q.run(cli).await?,
        }

        Ok(())
    }
}
