use anyhow::{Context, Result};
use bdk::wallet::AddressIndex;
use clap::Subcommand;

use wallet::{
    consts::{DEFAULT_WALLET_NAME, FEE_WALLET_NAME},
    Wallet,
};

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum WalletSubCommands {
    /// Create a wallet for vitalicals cli.
    Create { wallet: Option<String> },

    /// Import a mnemonic words [English] to init the wallet.
    Import {
        mnemonic: String,

        #[arg(long, default_value = "default")]
        wallet: String,
    },

    /// Get Balance for wallet.
    Balance { wallet: Option<String> },

    /// Get Address for wallet.
    Address {
        index: Option<u32>,

        #[arg(long, default_value = "default")]
        wallet: String,
    },
}

impl WalletSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::Create { wallet: wallet_name } => {
                create_wallet(cli, wallet_name)?;
            }
            Self::Import { mnemonic, wallet: wallet_name } => {
                import_mnemonic(cli, wallet_name, mnemonic.clone())?;
            }
            Self::Balance { wallet: wallet_name } => {
                balance(cli, wallet_name)?;
            }
            Self::Address { index, wallet: wallet_name } => {
                address(cli, index, wallet_name)?;
            }
        }

        Ok(())
    }
}

fn create_wallet(cli: &Cli, wallet_name: &Option<String>) -> Result<()> {
    let network = cli.network();

    if let Some(wallet_name) = wallet_name {
        Wallet::create(network, cli.endpoint.clone(), &cli.datadir, wallet_name, true)?;
    } else {
        Wallet::create(network, cli.endpoint.clone(), &cli.datadir, DEFAULT_WALLET_NAME, true)?;
        Wallet::create(network, cli.endpoint.clone(), &cli.datadir, FEE_WALLET_NAME, true)?;
    }

    Ok(())
}

fn import_mnemonic(cli: &Cli, wallet_name: &String, mnemonic: String) -> Result<()> {
    let network = cli.network();

    Wallet::create_by_mnemonic(
        network,
        cli.endpoint.clone(),
        &cli.datadir,
        wallet_name,
        mnemonic,
        true,
    )?;

    Ok(())
}

fn balance(cli: &Cli, wallet_name: &Option<String>) -> Result<()> {
    let network = cli.network();

    // TODO: support get balance for all wallet
    let wallet_name = wallet_name.clone().unwrap_or("default".to_string());

    let wallet = Wallet::load(network, cli.endpoint.clone(), &cli.datadir, &wallet_name, true)
        .context("load wallet failed")?;

    let balance = wallet.wallet.get_balance().context("get balance failed")?;
    println!("balance: {}", balance);

    Ok(())
}

fn address(cli: &Cli, index: &Option<u32>, wallet_name: &str) -> Result<()> {
    let network = cli.network();
    let wallet = Wallet::load(network, cli.endpoint.clone(), &cli.datadir, wallet_name, true)
        .context("load wallet failed")?;

    let address = if let Some(index) = index {
        wallet.wallet.get_address(AddressIndex::Peek(*index))
    } else {
        wallet.wallet.get_address(AddressIndex::New)
    }
    .context("get address failed")?;

    println!("address: {}", address);

    Ok(())
}
