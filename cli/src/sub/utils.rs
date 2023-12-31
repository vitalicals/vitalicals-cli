use anyhow::Result;
use bdk::wallet::AddressIndex;
use clap::Subcommand;

use bdk::bitcoin::Network;
use bdk::database::MemoryDatabase;
use bdk::keys::{
    bip39::{Language, Mnemonic, WordCount},
    DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
};
use bdk::template::Bip84;
use bdk::{miniscript, KeychainKind, Wallet};

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum UtilsSubCommands {
    /// Query shadowsats status.
    GenerateKey {},
}

impl UtilsSubCommands {
    pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
        let network = cli.network();

        generate_key(network)
    }
}

pub fn generate_key(network: Network) -> Result<()> {
    log::info!("Hello, world!");

    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English)).unwrap();
    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();
    // Parse a mnemonic
    let mnemonic = Mnemonic::parse(&mnemonic_words).unwrap();
    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key().unwrap();
    // Get xprv from the extended key
    let xprv = xkey.into_xprv(network).unwrap();

    // Create a BDK wallet structure using BIP 84 descriptor ("m/84h/1h/0h/0" and "m/84h/1h/0h/1")
    let wallet = Wallet::new(
        Bip84(xprv, KeychainKind::External),
        Some(Bip84(xprv, KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    )?;

    println!(
        "mnemonic: {}\n\nrecv desc (pub key): {:#?}\n\nchng desc (pub key): {:#?}",
        mnemonic_words,
        wallet
            .get_descriptor_for_keychain(KeychainKind::External)
            .to_string(),
        wallet
            .get_descriptor_for_keychain(KeychainKind::Internal)
            .to_string()
    );

    let address = wallet.get_address(AddressIndex::New)?;
    log::info!("address: {:?}", address);

    Ok(())
}
