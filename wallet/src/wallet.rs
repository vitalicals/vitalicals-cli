//! The wallet wrapper implementation by bdk
//!

use anyhow::{anyhow, Context, Result};

use bdk::bitcoin::Network;
use bdk::database::MemoryDatabase;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::wallet::{AddressIndex, AddressInfo};
use bdk::{miniscript, KeychainKind, Wallet as BdkWallet};

use crate::file::WalletFile;

/// Wallet
pub struct Wallet {
    pub wallet: BdkWallet<MemoryDatabase>,
    pub primary_address: AddressInfo,
    pub funding_address: AddressInfo,
}

impl Wallet {
    pub fn new(
        wallet: BdkWallet<MemoryDatabase>,
        primary_address: AddressInfo,
        funding_address: AddressInfo,
    ) -> Self {
        Self {
            wallet,
            primary_address,
            funding_address,
        }
    }

    pub fn create_from_wallet(wallet: BdkWallet<MemoryDatabase>) -> Result<Self> {
        let primary_address = wallet.get_address(AddressIndex::New)?;
        let funding_address = wallet.get_address(AddressIndex::New)?;

        Ok(Self {
            wallet,
            primary_address,
            funding_address,
        })
    }

    pub fn create(network: Network, path: &std::path::PathBuf) -> Result<Wallet> {
        // Generate fresh mnemonic
        let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
            Mnemonic::generate((WordCount::Words12, Language::English))
                .map_err(|err| anyhow!("generate Mnemonic failed by {:?}", err))?;
        // Convert mnemonic to string
        let mnemonic_words = mnemonic.to_string();

        Self::create_by_mnemonic(network, path, mnemonic_words)
    }

    pub fn create_by_mnemonic(
        network: Network,
        path: &std::path::PathBuf,
        mnemonic_words: String,
    ) -> Result<Self> {
        // Parse a mnemonic
        let mnemonic = Mnemonic::parse(&mnemonic_words)?;
        // Generate the extended key
        let xkey: ExtendedKey = mnemonic.into_extended_key()?;
        // Get xprv from the extended key
        let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

        let wallet = BdkWallet::new(
            Bip84(xprv, KeychainKind::External),
            Some(Bip84(xprv, KeychainKind::Internal)),
            network,
            MemoryDatabase::default(),
        )?;

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

    pub fn load(network: Network, root: &std::path::PathBuf) -> Result<Self> {
        let from_file = WalletFile::load(root, network).context("load file failed")?;

        from_file.into_wallet()
    }
}
