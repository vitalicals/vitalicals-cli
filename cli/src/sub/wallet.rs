use anyhow::{anyhow, Result};
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
pub enum WalletSubCommands {
    /// Create a wallet for shadowsats cli.
    Create,

    /// Import a mnemonic words [English] to init the wallet.
    Import { mnemonic: String },

    /// Get Balance for wallet.
    Balance,
}

impl WalletSubCommands {
    pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
        Ok(())
    }
}

fn create_wallet(cli: &Cli) -> Result<()> {
    let network = cli.network();

    // Generate fresh mnemonic
    let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|err| anyhow!("generate Mnemonic failed by {:?}", err))?;
    // Convert mnemonic to string
    let mnemonic_words = mnemonic.to_string();
    // Parse a mnemonic
    let mnemonic = Mnemonic::parse(&mnemonic_words)?;
    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    // Get xprv from the extended key
    let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

    // Create a BDK wallet structure using BIP 84 descriptor ("m/84h/1h/0h/0" and "m/84h/1h/0h/1")
    let wallet = Wallet::new(
        Bip84(xprv, KeychainKind::External),
        Some(Bip84(xprv, KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    )?;

    Ok(())
}

fn import_mnemonic(cli: &Cli, mnemonic: String) -> Result<()> {
    let network = cli.network();

    // Parse a mnemonic
    let mnemonic = Mnemonic::parse(&mnemonic)?;
    // Generate the extended key
    let xkey: ExtendedKey = mnemonic.into_extended_key()?;
    // Get xprv from the extended key
    let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

    // Create a BDK wallet structure using BIP 84 descriptor ("m/84h/1h/0h/0" and "m/84h/1h/0h/1")
    let wallet = Wallet::new(
        Bip84(xprv, KeychainKind::External),
        Some(Bip84(xprv, KeychainKind::Internal)),
        network,
        MemoryDatabase::default(),
    )?;

    Ok(())
}

fn balance(cli: &Cli) -> Result<()> {
    let network = cli.network();

    Ok(())
}
