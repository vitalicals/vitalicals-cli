//! The context for all cmds

use anyhow::Result;

use bdk::bitcoin::Network;
use vital_interfaces_indexer::IndexerClient;
use wallet::Wallet;

pub struct Context {
    pub root_path: std::path::PathBuf,
    pub wallet: Wallet,
    pub indexer: IndexerClient,
}

impl Context {
    pub async fn new(root_path: std::path::PathBuf, indexer: &str, wallet: Wallet) -> Result<Self> {
        let indexer = IndexerClient::new(indexer).await?;

        Ok(Self { root_path, wallet, indexer })
    }

    pub fn network(&self) -> Network {
        self.wallet.wallet.network()
    }
}
