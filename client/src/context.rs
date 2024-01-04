//! The context for all cmds

use bdk::bitcoin::Network;
use wallet::Wallet;

pub struct Context {
    pub root_path: std::path::PathBuf,
    pub wallet: Wallet,
}

impl Context {
    pub fn new(root_path: std::path::PathBuf, wallet: Wallet) -> Self {
        Self { root_path, wallet }
    }

    pub fn network(&self) -> Network {
        self.wallet.wallet.network()
    }
}
