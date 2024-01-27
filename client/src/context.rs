//! The context for all cmds

use std::str::FromStr;

use anyhow::{Context as AnyhowContext, Result};

use bdk::{
    bitcoin::{address::NetworkUnchecked, hashes::Hash as BdkHash, Address, Network},
    LocalUtxo,
};
use bitcoin::{hashes::Hash, OutPoint, Txid};

use vital_interfaces_indexer::{traits::IndexerClientT, IndexerClient};
use vital_script_primitives::resources::Resource;
use wallet::Wallet;

pub struct Context {
    pub root_path: std::path::PathBuf,
    pub wallet: Wallet,
    pub indexer: IndexerClient,
    pub to_address: Option<Address>,
}

impl Context {
    pub async fn new(root_path: std::path::PathBuf, indexer: &str, wallet: Wallet) -> Result<Self> {
        let indexer = IndexerClient::new(indexer).await?;

        Ok(Self { root_path, wallet, indexer, to_address: None })
    }

    pub fn with_to_address(mut self, to: &Option<impl ToString>) -> Result<Self> {
        if let Some(to) = to {
            let to = Address::<NetworkUnchecked>::from_str(to.to_string().as_str())
                .context("parse address failed")?
                .require_network(self.network())
                .context("the address is not for the network")?;

            self.to_address = Some(to);
        }

        Ok(self)
    }

    pub fn network(&self) -> Network {
        self.wallet.wallet.network()
    }

    pub async fn all_resources(&self) -> Result<Vec<(LocalUtxo, Resource)>> {
        let mut res = Vec::new();

        let outpoints = self.wallet.wallet.list_unspent().context("list unspents failed")?;

        for unspent in outpoints.into_iter() {
            log::debug!(
                "unspent {} - {:?} - {} - {}",
                unspent.is_spent,
                unspent.keychain,
                unspent.outpoint,
                unspent.txout.script_pubkey
            );
            let outpoint = unspent.outpoint;

            let resource = self
                .indexer
                .get_resource(&OutPoint {
                    txid: Txid::from_byte_array(*outpoint.txid.as_byte_array()),
                    vout: outpoint.vout,
                })
                .await?;
            if let Some(resource) = resource {
                log::debug!("find {} contain with resource {}", outpoint, resource);
                res.push((unspent, resource));
            }
        }

        Ok(res)
    }

    pub async fn utxo_with_resources(&self) -> Result<Vec<bdk::bitcoin::OutPoint>> {
        Ok(self
            .all_resources()
            .await?
            .into_iter()
            .map(|(unspent, _)| unspent.outpoint)
            .collect())
    }
}
