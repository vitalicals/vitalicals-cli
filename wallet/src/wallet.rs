//! The wallet wrapper implementation by bdk

use anyhow::{anyhow, bail, Context, Result};
use std::{path::Path, str::FromStr, time::UNIX_EPOCH};

use bdk::{
    bitcoin::{
        bip32::{self, DerivationPath, ExtendedPrivKey},
        secp256k1::{All, Secp256k1, XOnlyPublicKey},
        Network,
    },
    blockchain::{AnyBlockchain, GetHeight, Progress},
    database::{AnyDatabase, Database},
    descriptor::IntoWalletDescriptor,
    keys::{
        bip39::{Language, Mnemonic, WordCount},
        DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
    },
    miniscript::{self, Descriptor},
    template::Bip86,
    KeychainKind, SyncOptions, Wallet as BdkWallet,
};

use crate::{database::*, file::WalletFile};

/// Wallet
pub struct Wallet {
    pub name: String,
    pub xprv: String,
    pub xpriv: ExtendedPrivKey,
    pub wallet: BdkWallet<AnyDatabase>,
    pub blockchain: AnyBlockchain,
    pub synced: bool,
}

impl Wallet {
    pub fn create_from_wallet(
        name: &str,
        xprv: String,
        wallet: BdkWallet<AnyDatabase>,
        blockchain: AnyBlockchain,
    ) -> Result<Self> {
        let xpriv = ExtendedPrivKey::from_str(xprv.as_str()).context("ExtendedPrivKey from str")?;
        Ok(Self { name: name.to_string(), xprv, xpriv, wallet, blockchain, synced: false })
    }

    pub fn create(
        network: Network,
        endpoint: String,
        path: &std::path::PathBuf,
        name: &str,
        forced_sync: bool,
    ) -> Result<Wallet> {
        // Generate fresh mnemonic
        let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
            Mnemonic::generate((WordCount::Words12, Language::English))
                .map_err(|err| anyhow!("generate Mnemonic failed by {:?}", err))?;
        // Convert mnemonic to string
        let mnemonic_words = mnemonic.to_string();

        Self::create_by_mnemonic(network, endpoint, path, name, mnemonic_words, forced_sync)
    }

    pub fn create_by_mnemonic(
        network: Network,
        endpoint: String,
        root: &std::path::PathBuf,
        name: &str,
        mnemonic_words: String,
        forced_sync: bool,
    ) -> Result<Self> {
        // clean database datas.
        rm_database(network, root, name)?;

        // Parse a mnemonic
        let mnemonic = Mnemonic::parse(&mnemonic_words)?;
        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

        let (wallet, blockchain, synced) = Self::load_wallet(
            network,
            endpoint,
            root,
            name,
            Bip86(xprv, KeychainKind::External),
            Some(Bip86(xprv, KeychainKind::Internal)),
            forced_sync,
        )
        .context("load_wallet")?;

        println!(
            "mnemonic: {}\nrecv desc (pub key): {:#?}\nchng desc (pub key): {:#?}",
            mnemonic_words,
            wallet.get_descriptor_for_keychain(KeychainKind::External).to_string(),
            wallet.get_descriptor_for_keychain(KeychainKind::Internal).to_string()
        );

        let mut res = Self::create_from_wallet(name, xprv.to_string(), wallet, blockchain)?;
        res.save(root)?;

        res.synced = synced;

        Ok(res)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let to_file = WalletFile::from_wallet(self);

        to_file.save(root, &self.name)
    }

    pub fn load(
        network: Network,
        endpoint: String,
        root: &std::path::PathBuf,
        name: &str,
        forced_sync: bool,
    ) -> Result<Self> {
        let from_file = WalletFile::load(root, name, network).context("load file failed")?;
        let xpriv = bip32::ExtendedPrivKey::from_str(from_file.xpriv.as_str()).unwrap();

        let (wallet, blockchain, synced) = Self::load_wallet(
            network,
            endpoint,
            root,
            name,
            Bip86(xpriv, KeychainKind::External),
            Some(Bip86(xpriv, KeychainKind::Internal)),
            forced_sync,
        )
        .context("load wallet")?;

        Ok(Self {
            name: name.to_string(),
            xpriv,
            xprv: from_file.xpriv,
            wallet,
            blockchain,
            synced,
        })
    }

    fn load_wallet<E: IntoWalletDescriptor>(
        network: Network,
        endpoint: String,
        root: &std::path::PathBuf,
        name: &str,
        descriptor: E,
        change_descriptor: Option<E>,
        forced_sync: bool,
    ) -> Result<(BdkWallet<AnyDatabase>, AnyBlockchain, bool)> {
        let database = open_database(network, root, name).context("open_database")?;
        let blockchain = new_electrum_blockchain(endpoint).context("new_electrum_blockchain")?;

        let sync_time = database
            .get_sync_time()
            .map_err(|err| anyhow!("get sync time failed by {}", err))?;

        let wallet = BdkWallet::new(descriptor, change_descriptor, network, database)?;

        let need_sync = forced_sync
            || if let Some(sync_time) = sync_time {
                let current =
                    std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

                let latest_block_height = blockchain
                    .get_height()
                    .map_err(|err| anyhow!("get block height failed by {}", err))?;

                latest_block_height != sync_time.block_time.height
                    || current >= sync_time.block_time.timestamp + 60
            } else {
                false
            };

        if need_sync {
            wallet
                .sync(&blockchain, SyncOptions { progress: Some(Box::new(ProgressLogger {})) })
                .context("sync")?;

            match &*wallet.database() {
                AnyDatabase::Memory(_) => {}
                AnyDatabase::Sled(sled) => {
                    sled.flush().map_err(|err| anyhow!("flush failed: {:?}", err))?;
                }
            }
        }

        Ok((wallet, blockchain, need_sync))
    }
}

impl Wallet {
    pub fn full_derivation_path(&self) -> Result<DerivationPath> {
        let descriptor = self.wallet.get_descriptor_for_keychain(KeychainKind::External);

        let tr = match descriptor {
            Descriptor::Tr(tr) => tr,
            _ => bail!("not tr descriptor"),
        };
        let derivation_path = tr.internal_key().full_derivation_path().unwrap();

        Ok(derivation_path)
    }

    pub fn xpriv(&self) -> &ExtendedPrivKey {
        &self.xpriv
    }

    pub fn flush(&self) -> Result<()> {
        match &*self.wallet.database() {
            AnyDatabase::Memory(_) => {}
            AnyDatabase::Sled(sled) => {
                sled.flush().map_err(|err| anyhow!("flush failed: {:?}", err))?;
            }
        }

        Ok(())
    }

    pub fn derive_x_only_public_key(&self, secp: &Secp256k1<All>) -> Result<XOnlyPublicKey> {
        let derivation_path = self.full_derivation_path().context("get full derivation")?;
        let (internal_key, _) = self
            .xpriv
            .derive_priv(secp, &derivation_path)?
            .to_keypair(secp)
            .x_only_public_key();

        Ok(internal_key)
    }

    pub fn network(&self) -> Network {
        self.wallet.network()
    }
}

#[derive(Debug)]
pub struct ProgressLogger {}

impl Progress for ProgressLogger {
    fn update(
        &self,
        progress: f32,
        message: Option<String>,
    ) -> std::prelude::v1::Result<(), bdk::Error> {
        if let Some(message) = message {
            println!("sync progress {} : {}", message, progress)
        } else {
            println!("sync progress: {}", progress)
        }

        Ok(())
    }
}
