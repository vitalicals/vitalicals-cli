use std::path::PathBuf;

use anyhow::{Context, Result};

use bdk::{
	bitcoin::Network,
	blockchain::{
		AnyBlockchain, AnyBlockchainConfig, ConfigurableBlockchain, ElectrumBlockchainConfig,
	},
	database::{any::SledDbConfiguration, AnyDatabase, AnyDatabaseConfig, ConfigurableDatabase},
};

pub(crate) fn new_electrum_blockchain(endpoint: String) -> Result<AnyBlockchain> {
	let config = {
		AnyBlockchainConfig::Electrum(ElectrumBlockchainConfig {
			url: endpoint,
			socks5: None,
			retry: 5,
			timeout: None,
			stop_gap: 10,
			validate_domain: true,
		})
	};

	Ok(AnyBlockchain::from_config(&config)?)
}

/// Open the wallet database.
pub(crate) fn open_database(network: Network, root: &PathBuf) -> Result<AnyDatabase> {
	let wallet_name = network.to_core_arg();

	let database_path = prepare_wallet_db_dir(wallet_name, root)?;

	let config = AnyDatabaseConfig::Sled(SledDbConfiguration {
		path: database_path.into_os_string().into_string().expect("path string"),
		tree_name: wallet_name.to_string(),
	});

	let database = AnyDatabase::from_config(&config)?;
	log::debug!("database opened successfully");
	Ok(database)
}

pub(crate) fn rm_database(network: Network, root: &PathBuf) -> Result<()> {
	let wallet_name = network.to_core_arg();

	let database_path = prepare_wallet_db_dir(wallet_name, root)?;
	std::fs::remove_dir_all(database_path)?;

	Ok(())
}

/// Prepare wallet database directory.
fn prepare_wallet_db_dir(wallet_name: &str, root: &PathBuf) -> Result<PathBuf> {
	let mut db_dir = prepare_wallet_dir(wallet_name, root)?;

	db_dir.push("wallet.sled");

	if !db_dir.exists() {
		log::info!("Creating database directory {}", db_dir.as_path().display());
		std::fs::create_dir(&db_dir).context("create dir")?;
	}

	Ok(db_dir)
}

fn prepare_wallet_dir(wallet_name: &str, root: &PathBuf) -> Result<PathBuf> {
	let mut dir = root.to_owned();

	dir.push(wallet_name);

	if !dir.exists() {
		log::info!("Creating wallet directory {}", dir.as_path().display());
		std::fs::create_dir(&dir).context("create dir")?;
	}

	Ok(dir)
}
