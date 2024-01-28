use anyhow::Result;
use clap::{Parser, Subcommand};

use bdk::bitcoin::Network;

mod sub;
use sub::*;

pub(crate) use sub::{build_context, Context};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "vitalicals-cli")]
#[command(about = "A vitalicals CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: SubCommands,

    /// if show debug log
    #[arg(short, long, default_value = "false")]
    pub debug: bool,

    /// Sets the network.
    #[arg(
        short = 'n',
        long = "network",
        default_value = "testnet",
        value_parser = ["bitcoin", "testnet", "signet", "regtest"],
    )]
    pub network: String,

    /// The url for electrum.
    #[arg(short = 'e', long = "endpoint")]
    pub endpoint: String,

    /// The endpoint for indexer
    #[arg(short = 'i', long = "indexer")]
    pub indexer: String,

    /// Sets the wallet data directory.
    /// Default value : "~/.vitalicals-cli
    #[clap(name = "DATADIR", short = 'd', long = "datadir", default_value = "./.vitalicals-cli")]
    pub datadir: std::path::PathBuf,
}

impl Cli {
    /// Get the network parameters.
    pub fn network(&self) -> Network {
        match self.network.as_str() {
            "bitcoin" => Network::Bitcoin,
            "testnet" => Network::Testnet,
            "signet" => Network::Signet,
            "regtest" => Network::Regtest,
            _ => panic!("Invalid network params {}", self.network),
        }
    }
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Query vitalicals status.
    #[command(subcommand)]
    Query(query::QuerySubCommands),

    /// Mint tokens
    #[command(subcommand)]
    Mint(mint::MintSubCommands),

    /// Deploy tokens
    #[command(subcommand)]
    Deploy(deploy::DeploySubCommands),

    /// Transfer tokens
    #[command(subcommand)]
    Transfer(transfer::TransferSubCommands),

    /// Wallet cmds
    #[command(subcommand)]
    Wallet(wallet::WalletSubCommands),

    /// Wallet cmds
    #[command(subcommand)]
    Utils(utils::UtilsSubCommands),
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.debug { log::LevelFilter::Debug } else { log::LevelFilter::Warn };
    let _ = env_logger::Builder::from_default_env()
        .format_module_path(true)
        .format_level(true)
        .filter_level(log_level)
        .try_init();

    log::debug!("Run cli {:?}", cli);

    std::fs::create_dir_all(cli.datadir.clone()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    match &cli.command {
        SubCommands::Query(cmd) => cmd.run(&cli).await,
        SubCommands::Mint(cmd) => cmd.run(&cli).await,
        SubCommands::Deploy(cmd) => cmd.run(&cli).await,
        SubCommands::Transfer(cmd) => cmd.run(&cli).await,
        SubCommands::Wallet(cmd) => cmd.run(&cli).await,
        SubCommands::Utils(cmd) => cmd.run(&cli).await,
    }
}
