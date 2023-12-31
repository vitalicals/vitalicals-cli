//! The wallet stored in file

use anyhow::{anyhow, Context, Result};
use bdk::bitcoin::address::NetworkUnchecked;
use bdk::bitcoin::Address;
use serde::{Deserialize, Serialize};
use std::fs;

use bdk::database::MemoryDatabase;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{bitcoin::Network, Wallet};
use bdk::{miniscript, KeychainKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub network: Network,
    pub descriptor: String,
    pub change_descriptor: String,
    pub primary_address: (u32, Address<NetworkUnchecked>),
    pub funding_address: (u32, Address<NetworkUnchecked>),
}

impl WalletFile {
    pub fn from_wallet(wallet: &crate::wallet::Wallet) -> Self {
        let primary = &wallet.primary_address;
        let funding = &wallet.funding_address;

        Self {
            network: wallet.wallet.network(),
            descriptor: wallet
                .wallet
                .get_descriptor_for_keychain(KeychainKind::External)
                .to_string(),
            change_descriptor: wallet
                .wallet
                .get_descriptor_for_keychain(KeychainKind::Internal)
                .to_string(),
            primary_address: (
                primary.index,
                Address::new(primary.address.network, primary.address.payload.clone()),
            ),
            funding_address: (
                funding.index,
                Address::new(funding.address.network, funding.address.payload.clone()),
            ),
        }
    }

    pub fn path_to_wallet(root: &std::path::PathBuf, network: Network) -> std::path::PathBuf {
        let path = root.join(network.to_core_arg());
        std::fs::create_dir_all(path.clone()).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });

        path.join("wallet.json")
    }

    pub fn path_to_tmp_wallet(root: &std::path::PathBuf, network: Network) -> std::path::PathBuf {
        use std::time::*;

        let current = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time unix error");

        let in_ms = current.as_secs() as u128 * 1000 + current.subsec_millis() as u128;

        root.join(network.to_core_arg())
            .join(format!("wallet-backup-{}.json", in_ms))
    }

    pub fn load(root: &std::path::PathBuf, network: Network) -> Result<Self> {
        let path = Self::path_to_wallet(root, network);

        let res: WalletFile = serde_json::from_str(
            fs::read_to_string(path)
                .context("read file error")?
                .as_str(),
        )
        .context("json from str")?;

        Ok(res)
    }

    pub fn save(&self, root: &std::path::PathBuf) -> Result<()> {
        let path = Self::path_to_wallet(root, self.network);

        let datas = serde_json::to_string_pretty(self)?;
        if path.exists() {
            let old = Self::load(root, self.network).context("load for rename")?;
            if old.descriptor != self.descriptor || old.change_descriptor != self.change_descriptor
            {
                log::warn!("the wallet will overwrite the old wallet, so mv to backup.");
                fs::rename(path.clone(), Self::path_to_tmp_wallet(root, self.network))
                    .context("rename to backup")?;
            }
        }

        fs::write(path, datas).context("write")?;

        Ok(())
    }
}
