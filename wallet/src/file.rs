//! The wallet stored in file

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

use bdk::bitcoin::Network;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletFile {
    pub network: Network,
    pub xpriv: String,
}

impl WalletFile {
    pub fn from_wallet(wallet: &crate::wallet::Wallet) -> Self {
        Self { network: wallet.wallet.network(), xpriv: wallet.xprv.clone() }
    }

    pub fn path_to_wallet(root: &Path, network: Network) -> std::path::PathBuf {
        let path = root.join(network.to_core_arg());
        std::fs::create_dir_all(path.clone()).unwrap_or_else(|why| {
            println!("! {:?}", why.kind());
        });

        path.join("wallet.json")
    }

    pub fn path_to_tmp_wallet(root: &Path, network: Network) -> std::path::PathBuf {
        use std::time::*;

        let current = SystemTime::now().duration_since(UNIX_EPOCH).expect("time unix error");

        let in_ms = current.as_secs() as u128 * 1000 + current.subsec_millis() as u128;

        root.join(network.to_core_arg()).join(format!("wallet-backup-{}.json", in_ms))
    }

    pub fn load(root: &Path, network: Network) -> Result<Self> {
        let path = Self::path_to_wallet(root, network);

        let res: WalletFile =
            serde_json::from_str(fs::read_to_string(path).context("read file error")?.as_str())
                .context("json from str")?;

        Ok(res)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = Self::path_to_wallet(root, self.network);

        let datas = serde_json::to_string_pretty(self)?;
        if path.exists() {
            let old = Self::load(root, self.network).context("load for rename");

            let need_backup = match old {
                Ok(old) => old.xpriv != self.xpriv,
                Err(_) => true,
            };

            if need_backup {
                log::warn!("the wallet will overwrite the old wallet, so mv to backup.");
                fs::rename(path.clone(), Self::path_to_tmp_wallet(root, self.network))
                    .context("rename to backup")?;
            }
        }

        fs::write(path, datas).context("write")?;

        Ok(())
    }
}
