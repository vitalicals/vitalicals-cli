use std::fs::DirEntry;

use anyhow::{Context, Result};
use bdk::wallet::AddressIndex;
use clap::Subcommand;

use wallet::{
    consts::{DEFAULT_WALLET_NAME, FEE_WALLET_NAME},
    Wallet, WalletFile,
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
    },

    /// List all alive wallets.
    List,
}

impl WalletSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        match self {
            Self::Create { wallet: wallet_name } => {
                if cli.wallet.is_some(){
                    panic!("should not use --wallet");
                }

                create_wallet(cli, wallet_name)?;
            }
            Self::Import { mnemonic, wallet: wallet_name } => {
                if cli.wallet.is_some(){
                    panic!("should not use --wallet");
                }

                import_mnemonic(cli, wallet_name, mnemonic.clone())?;
            }
            Self::Balance { wallet: wallet_name } => {
                let wallet = match (cli.wallet.clone(), wallet_name.clone()){
                    (None, None) => DEFAULT_WALLET_NAME.to_string(),
                    (Some(wallet), None) => wallet,
                    (None, Some(wallet)) => wallet,
                    (Some(w1), Some(w2)) => {
                        if w1 == w2{
                            w1
                        }else{
                            panic!("The `--wallet` use {}, but args use wallet {}, just use one arg to select wallet!", w1, w2);
                        }
                    }
                };

                balance(cli, &Some(wallet))?;
            }
            Self::Address { index } => {
                let wallet = cli.wallet.clone().unwrap_or(DEFAULT_WALLET_NAME.to_string());
                address(cli, index, &wallet)?;
            }
            Self::List => {
                list(cli)?;
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

fn import_mnemonic(cli: &Cli, wallet_name: &str, mnemonic: String) -> Result<()> {
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
    let wallet_name = wallet_name.clone().unwrap_or(DEFAULT_WALLET_NAME.to_string());

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

    wallet.flush()?;

    Ok(())
}

fn list(cli: &Cli) -> Result<()> {
    let network = cli.network();
    let root = cli.datadir.join(network.to_core_arg());

    match std::fs::read_dir(root) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if let Ok(Some(name)) = try_get_wallet_name(cli, &entry) {
                            println!("{}", name);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

fn try_get_wallet_name(cli: &Cli, dir: &DirEntry) -> Result<Option<String>> {
    let metadata = dir.metadata()?;
    if !metadata.is_dir() {
        return Ok(None)
    }

    let path = dir.file_name().to_string_lossy().to_string();
    let network = cli.network();

    let _wallet = WalletFile::load(&cli.datadir, &path, network)?;

    Ok(Some(path))
}
