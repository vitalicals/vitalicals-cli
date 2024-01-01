use anyhow::Result;
use clap::Subcommand;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum TransferSubCommands {
	/// Query shadowsats status.
	Query {
		/// The remote to clone
		remote: String,
	},
}

impl TransferSubCommands {
	pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
		Ok(())
	}
}
