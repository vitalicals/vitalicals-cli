use anyhow::{Context, Result};
use bitcoin::OutPoint;
use clap::Parser;

use vital_interfaces_indexer::traits::IndexerClientT;

use crate::{sub::context::build_context, Cli};

#[derive(Debug, Parser)]
#[command(name = "query resources", about = "Query resources hold by wallet")]
pub struct QueryResources {
    /// If use outpoint arg, just return it 's outpoint
    #[arg(long)]
    outpoint: Option<OutPoint>,
}

impl QueryResources {
    pub async fn run(&self, cli: &Cli) -> Result<()> {
        let context = build_context(cli).await.context("build context")?;

        if let Some(outpoint) = self.outpoint {
            log::debug!("query resource by {}", outpoint);

            let resource = context.indexer.get_resource(&outpoint).await?;
            if let Some(resource) = resource {
                println!("find {} contain with resource {}", outpoint, resource);
            } else {
                println!("not found resource by {}", outpoint);
            }
        } else {
            log::debug!("query resources");

            let resources =
                context.fetch_all_resources().await?.into_iter().enumerate().collect::<Vec<_>>();
            println!("find {} resources", resources.len());
            for (i, local) in resources.into_iter() {
                if local.pending {
                    println!(
                        "{}. find pending {} contain with resource {}",
                        i, local.utxo.outpoint, local.resource
                    );
                } else {
                    println!(
                        "{}. find {} contain with resource {}",
                        i, local.utxo.outpoint, local.resource
                    );
                }
            }
        }

        Ok(())
    }
}
