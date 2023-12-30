use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};

use bdk::bitcoin::{Address, Network, OutPoint, Script};

mod sub;
use sub::*;

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "shadowsats-cli")]
#[command(about = "A shadowsats CLI", long_about = None)]
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

    /// Sets the wallet data directory.
    /// Default value : "~/.shadowsats-cli
    #[clap(
        name = "DATADIR",
        short = 'd',
        long = "datadir",
        default_value = "./.shadowsats-cli"
    )]
    pub datadir: std::path::PathBuf,
}

#[derive(Debug, Subcommand)]
enum SubCommands {
    /// Query shadowsats status.
    #[command(subcommand)]
    Query(query::QuerySubCommands),

    /// Mint tokens
    #[command(subcommand)]
    Mint(mint::MintSubCommands),

    /// Transfer tokens
    #[command(subcommand)]
    Transfer(transfer::TransferSubCommands),

    /// Wallet cmds
    #[command(subcommand)]
    Wallet(wallet::WalletSubCommands),
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.debug {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Warn
    };
    let _ = env_logger::Builder::from_default_env()
        .format_module_path(true)
        .format_level(true)
        .filter_level(log_level)
        .try_init();

    log::debug!("Run cli {:?}", cli);

    std::fs::create_dir_all(cli.datadir.clone()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    match cli.command {
        SubCommands::Query(cmd) => cmd.run(),
        SubCommands::Mint(cmd) => cmd.run(),
        SubCommands::Transfer(cmd) => cmd.run(),
        SubCommands::Wallet(cmd) => cmd.run(),
    }
}
