use anyhow::{anyhow, Context, Result};
use clap::Parser;

use vital_script_primitives::{resources::Name, traits::EnvContext};

use crate::{sub::context::build_context, Cli};

#[derive(Debug, Parser)]
#[command(name = "query vrc20 metadata", about = "Query vrc20 metadata by name")]
pub struct QueryVrc20Metadata {
    name: String,
}

impl QueryVrc20Metadata {
    pub async fn run(&self, cli: &Cli) -> Result<()> {
        let context = build_context(cli).await.context("build context")?;

        let name = Name::try_from(self.name.as_str())
            .with_context(|| format!("the vrc20 name {} format invalid", self.name))?;

        let vrc20_metadata = context
            .query_env_context
            .get_vrc20_metadata(name)
            .context("get vrc20 metadata")?
            .ok_or_else(|| anyhow!("not found vrc20 metadata by {}", name))?;

        println!("metadata: {}", serde_json::to_string_pretty(&vrc20_metadata).expect("json"));

        Ok(())
    }
}
