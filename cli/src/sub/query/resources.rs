use anyhow::{Context, Result};
use bdk::bitcoin::hashes::Hash as bdk_hashes;
use bitcoin::{hashes::Hash, OutPoint, Txid};
use clap::Parser;

use vital_interfaces_indexer::traits::IndexerClientT;

use crate::{sub::context::build_context, Cli};

#[derive(Debug, Parser)]
#[command(name = "query resources", about = "Query resources hold by wallet")]
pub struct QueryResouces {
    /// If use outpoint arg, just return it 's outpoint
    #[arg(long)]
    outpoint: Option<OutPoint>,
}

impl QueryResouces {
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

            let mut resources = Vec::new();

            let outpoints = context.wallet.wallet.list_unspent().context("list unspents failed")?;
            for unspent in outpoints.into_iter() {
                log::debug!(
                    "unspent {} - {:?} - {} - {}",
                    unspent.is_spent,
                    unspent.keychain,
                    unspent.outpoint,
                    unspent.txout.script_pubkey
                );
                let outpoint = unspent.outpoint;

                let resource = context
                    .indexer
                    .get_resource(&OutPoint {
                        txid: Txid::from_byte_array(*outpoint.txid.as_byte_array()),
                        vout: outpoint.vout,
                    })
                    .await?;
                if let Some(resource) = resource {
                    log::debug!("find {} contain with resource {}", outpoint, resource);
                    resources.push((outpoint, resource));
                }
            }

            println!("find {} resources", resources.len());
            for (i, (outpoint, resource)) in resources.into_iter().enumerate() {
                println!("{}. find {} contain with resource {}", i, outpoint, resource);
            }
        }

        Ok(())
    }
}
