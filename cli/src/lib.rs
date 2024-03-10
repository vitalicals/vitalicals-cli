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
    #[arg(long, default_value = "false")]
    pub debug: bool,

    /// Sets the network.
    #[arg(
        short = 'n',
        long = "network",
        default_value = "bitcoin",
        value_parser = ["bitcoin", "testnet", "signet", "regtest"],
    )]
    pub network: String,

    /// The url for electrum.
    #[arg(short = 'e', long = "endpoint", default_value = "127.0.0.1:50002")]
    pub endpoint: String,

    /// The endpoint for indexer
    #[arg(short = 'i', long = "indexer", default_value = "http://localhost:9944")]
    pub indexer: String,

    /// Sets the wallet data directory.
    /// Default value : "~/.vitalicals-cli
    #[clap(name = "DATADIR", short = 'd', long = "datadir", default_value = "./.vitalicals-cli")]
    pub datadir: std::path::PathBuf,

    /// The bitcoin address to send to, if not set, will create a new address.
    #[arg(long)]
    to: Option<String>,

    /// The btc sats for output
    #[arg(long, default_value = "600")]
    sats: u64,

    /// Specify a fee rate in sat/vB.
    #[arg(short, long)]
    fee_rate: Option<f32>,

    /// Signal that this transaction can be replaced by a transaction (BIP 125).
    #[arg(long)]
    replaceable: bool,

    /// If need forced sync
    #[arg(long, default_value = "false")]
    no_sync: bool,

    /// The name of wallet for vital resources
    #[arg(long)]
    wallet: Option<String>,

    /// The name of wallet for fee
    #[arg(long)]
    fee_wallet: Option<String>,
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
    Move(move_resource::MoveSubCommands),

    /// Wallet cmds
    #[command(subcommand)]
    Wallet(wallet::WalletSubCommands),

    /// Wallet cmds
    #[command(subcommand)]
    Utils(utils::UtilsSubCommands),

    /// Version
    Version {
        #[arg(long)]
        json: bool,
    },
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.debug { log::LevelFilter::Debug } else { log::LevelFilter::Warn };
    let _ = env_logger::Builder::from_default_env()
        .format_module_path(true)
        .format_level(true)
        .filter_level(log_level)
        // .parse_filters("bdk::blockchain::script_sync=info")
        .try_init();

    log::debug!("Run cli {:?}", cli);

    std::fs::create_dir_all(cli.datadir.clone()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    match &cli.command {
        SubCommands::Query(cmd) => cmd.run(&cli).await,
        SubCommands::Mint(cmd) => cmd.run(&cli).await,
        SubCommands::Deploy(cmd) => cmd.run(&cli).await,
        SubCommands::Move(cmd) => cmd.run(&cli).await,
        SubCommands::Wallet(cmd) => cmd.run(&cli).await,
        SubCommands::Utils(cmd) => cmd.run(&cli).await,
        SubCommands::Version { json } => print_version(*json),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct VersionInfo {
    build_timestamp: String,
    cargo_debug: String,
    git_describe: String,
    git_branch: String,
    git_commit_message: String,
    git_commit_timestamp: String,
    git_sha: String,
    git_dirty: String,
    rustc_channel: String,
    rustc_semver: String,
    rustc_host_triple: String,
}

impl Default for VersionInfo {
    fn default() -> Self {
        Self {
            build_timestamp: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
            cargo_debug: env!("VERGEN_CARGO_DEBUG").to_string(),
            git_describe: env!("VERGEN_GIT_DESCRIBE").to_string(),
            git_branch: env!("VERGEN_GIT_BRANCH").to_string(),
            git_commit_message: env!("VERGEN_GIT_COMMIT_MESSAGE").to_string(),
            git_commit_timestamp: env!("VERGEN_GIT_COMMIT_TIMESTAMP").to_string(),
            git_sha: env!("VERGEN_GIT_SHA").to_string(),
            git_dirty: env!("VERGEN_GIT_DIRTY").to_string(),
            rustc_channel: env!("VERGEN_RUSTC_CHANNEL").to_string(),
            rustc_semver: env!("VERGEN_RUSTC_SEMVER").to_string(),
            rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE").to_string(),
        }
    }
}

impl VersionInfo {
    fn print(&self) {
        if self.git_dirty == "true" {
            println!("{}-dirty", self.git_describe);
        } else {
            println!("{}", self.git_describe);
        }
    }
}

fn print_version(json: bool) -> Result<()> {
    let info = VersionInfo::default();

    if json {
        println!("{}", serde_json::to_string_pretty(&info).expect("the json should be ok"));
    } else {
        info.print();
    }

    Ok(())
}
