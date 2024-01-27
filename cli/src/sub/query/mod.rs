use anyhow::Result;
use clap::Subcommand;

mod resources;

use crate::Cli;

use self::resources::QueryResouces;

#[derive(Debug, Subcommand)]
pub enum QuerySubCommands {
    /// Query vitalicals status.
    FtInfo {
        /// The remote to clone
        id: String,
    },
    /// Query resources.
    Resources(QueryResouces),
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
