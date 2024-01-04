//! The context for all cmds

use std::sync::Arc;

use bdk::bitcoin::Network;
use wallet::Wallet;

pub struct Context {
    pub root_path: std::path::PathBuf,
    pub wallet: Arc<Wallet>,
}

impl Context {
    pub fn new(root_path: std::path::PathBuf, wallet: Wallet) -> Self {
        Self { root_path, wallet: Arc::new(wallet) }
    }

    pub fn network(&self) -> Network {
        self.wallet.wallet.network()
    }
}
