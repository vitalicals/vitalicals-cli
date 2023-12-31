use anyhow::{Context, Result};
use clap::Subcommand;

use wallet::Wallet;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum WalletSubCommands {
    /// Create a wallet for shadowsats cli.
    Create,

    /// Import a mnemonic words [English] to init the wallet.
    Import { mnemonic: String },

    /// Get Balance for wallet.
    Balance,

    /// Get Address for wallet.
    Address,
}

impl WalletSubCommands {
    pub(crate) fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::Create => {
                create_wallet(cli)?;
            }
            Self::Import { mnemonic } => {
                import_mnemonic(cli, mnemonic.clone())?;
            }
            Self::Balance => {
                balance(cli)?;
            }
            Self::Address => {
                address(cli)?;
            }
        }

        Ok(())
    }
}

fn create_wallet(cli: &Cli) -> Result<()> {
    let network = cli.network();

    Wallet::create(network, cli.endpoint.clone(), &cli.datadir)?;

    Ok(())
}

fn import_mnemonic(cli: &Cli, mnemonic: String) -> Result<()> {
    let network = cli.network();

    Wallet::create_by_mnemonic(network, cli.endpoint.clone(), &cli.datadir, mnemonic)?;

    Ok(())
}

fn balance(cli: &Cli) -> Result<()> {
    let network = cli.network();
    let wallet =
        Wallet::load(network, cli.endpoint.clone(), &cli.datadir).context("load wallet failed")?;

    let balance = wallet.wallet.get_balance().context("get balance failed")?;
    println!("balance: {}", balance);

    Ok(())
}

fn address(cli: &Cli) -> Result<()> {
    let network = cli.network();
    let wallet =
        Wallet::load(network, cli.endpoint.clone(), &cli.datadir).context("load wallet failed")?;

    println!("primary_address: {}", wallet.primary_address);
    println!("funding_address: {}", wallet.funding_address);

    Ok(())
}
