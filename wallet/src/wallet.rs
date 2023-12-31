//! The wallet wrapper implementation by bdk
//!

use anyhow::{anyhow, Context, Result};

use bdk::bitcoin::Network;
use bdk::database::{AnyDatabase, MemoryDatabase};
use bdk::descriptor::IntoWalletDescriptor;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{miniscript, KeychainKind, SyncOptions, Wallet as BdkWallet};

use crate::database::*;
use crate::file::WalletFile;

/// Wallet
pub struct Wallet {
    pub wallet: BdkWallet<AnyDatabase>,
    pub primary_address: AddressInfo,
    pub funding_address: AddressInfo,
}

impl Wallet {
    pub fn new(
        wallet: BdkWallet<AnyDatabase>,
        primary_address: AddressInfo,
        funding_address: AddressInfo,
    ) -> Self {
        Self {
            wallet,
            primary_address,
            funding_address,
        }
    }

    pub fn create_from_wallet(wallet: BdkWallet<AnyDatabase>) -> Result<Self> {
        let primary_address = wallet.get_address(AddressIndex::New)?;
        let funding_address = wallet.get_address(AddressIndex::New)?;

        println!("primary_address {:?}", primary_address);
        println!("funding_address {:?}", funding_address);

        Ok(Self {
            wallet,
            primary_address,
            funding_address,
        })
    }

    pub fn create(network: Network, endpoint: String, path: &std::path::PathBuf) -> Result<Wallet> {
        // Generate fresh mnemonic
        let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
            Mnemonic::generate((WordCount::Words12, Language::English))
                .map_err(|err| anyhow!("generate Mnemonic failed by {:?}", err))?;
        // Convert mnemonic to string
        let mnemonic_words = mnemonic.to_string();

        Self::create_by_mnemonic(network, endpoint, path, mnemonic_words)
    }

    pub fn create_by_mnemonic(
        network: Network,
        endpoint: String,
        path: &std::path::PathBuf,
        mnemonic_words: String,
    ) -> Result<Self> {
        // Parse a mnemonic
        let mnemonic = Mnemonic::parse(&mnemonic_words)?;
        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

        let wallet = Self::load_wallet(
            network,
            endpoint,
            path,
            Bip84(xprv, KeychainKind::External),
            Some(Bip84(xprv, KeychainKind::Internal)),
        )
        .context("load_wallet")?;

        println!(
            "mnemonic: {}\nrecv desc (pub key): {:#?}\nchng desc (pub key): {:#?}",
            mnemonic_words,
            wallet
                .get_descriptor_for_keychain(KeychainKind::External)
                .to_string(),
            wallet
                .get_descriptor_for_keychain(KeychainKind::Internal)
                .to_string()
        );

        let res = Self::create_from_wallet(wallet)?;

        res.save(path)?;

        Ok(res)
    }

    pub fn save(&self, root: &std::path::PathBuf) -> Result<()> {
        let to_file = WalletFile::from_wallet(self);

        to_file.save(root)
    }

    pub fn load(network: Network, endpoint: String, root: &std::path::PathBuf) -> Result<Self> {
        let from_file = WalletFile::load(root, network).context("load file failed")?;

        let primary_address = AddressInfo {
            index: from_file.primary_address.0,
            address: from_file.primary_address.1.assume_checked(),
            keychain: KeychainKind::External,
        };

        let funding_address = AddressInfo {
            index: from_file.funding_address.0,
            address: from_file.funding_address.1.assume_checked(),
            keychain: KeychainKind::External,
        };

        let wallet = Self::load_wallet(
            network,
            endpoint,
            root,
            from_file.descriptor.as_str(),
            Some(from_file.change_descriptor.as_str()),
        )
        .context("load wallet")?;

        Ok(Self {
            wallet,
            primary_address,
            funding_address,
        })
    }

    fn load_wallet<E: IntoWalletDescriptor>(
        network: Network,
        endpoint: String,
        root: &std::path::PathBuf,
        descriptor: E,
        change_descriptor: Option<E>,
    ) -> Result<BdkWallet<AnyDatabase>> {
        let database = open_database(network, root).context("open_database")?;
        let blockchain = new_electrum_blockchain(endpoint).context("new_electrum_blockchain")?;

        let wallet = BdkWallet::new(descriptor, change_descriptor, network, database)?;

        wallet
            .sync(&blockchain, SyncOptions::default())
            .context("sync")?;

        Ok(wallet)
    }
}
