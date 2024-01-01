//! The wallet wrapper implementation by bdk

use anyhow::{anyhow, Context, Result};
use std::str::FromStr;

use bdk::{
	bitcoin::{bip32, Network},
	blockchain::AnyBlockchain,
	database::AnyDatabase,
	descriptor::IntoWalletDescriptor,
	keys::{
		bip39::{Language, Mnemonic, WordCount},
		DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
	},
	miniscript,
	template::Bip84,
	KeychainKind, SyncOptions, Wallet as BdkWallet,
};

use crate::{database::*, file::WalletFile};

/// Wallet
pub struct Wallet {
	pub(crate) xprv: String,
	pub wallet: BdkWallet<AnyDatabase>,
	pub blockchain: AnyBlockchain,
}

impl Wallet {
	pub fn create_from_wallet(
		xprv: String,
		wallet: BdkWallet<AnyDatabase>,
		blockchain: AnyBlockchain,
	) -> Result<Self> {
		Ok(Self { xprv, wallet, blockchain })
	}

	pub fn create(network: Network, endpoint: String, path: &std::path::PathBuf) -> Result<Wallet> {
		// Generate fresh mnemonic
		let mnemonic: GeneratedKey<_, miniscript::Segwitv0> =
			Mnemonic::generate((WordCount::Words12, Language::English))
				.map_err(|err| anyhow!("generate Mnemonic failed by {:?}", err))?;
		// Convert mnemonic to string
		let mnemonic_words = mnemonic.to_string();

		Self::create_by_mnemonic(network, endpoint, path, mnemonic_words)
	}

	pub fn create_by_mnemonic(
		network: Network,
		endpoint: String,
		root: &std::path::PathBuf,
		mnemonic_words: String,
	) -> Result<Self> {
		// clean database datas.
		rm_database(network, root)?;

		// Parse a mnemonic
		let mnemonic = Mnemonic::parse(&mnemonic_words)?;
		// Generate the extended key
		let xkey: ExtendedKey = mnemonic.into_extended_key()?;
		// Get xprv from the extended key
		let xprv = xkey.into_xprv(network).ok_or(anyhow!("not got xprv"))?;

		let (wallet, blockchain) = Self::load_wallet(
			network,
			endpoint,
			root,
			Bip84(xprv, KeychainKind::External),
			Some(Bip84(xprv, KeychainKind::Internal)),
		)
		.context("load_wallet")?;

		println!(
			"mnemonic: {}\nrecv desc (pub key): {:#?}\nchng desc (pub key): {:#?}",
			mnemonic_words,
			wallet.get_descriptor_for_keychain(KeychainKind::External).to_string(),
			wallet.get_descriptor_for_keychain(KeychainKind::Internal).to_string()
		);

		let res = Self::create_from_wallet(xprv.to_string(), wallet, blockchain)?;
		res.save(root)?;

		Ok(res)
	}

	pub fn save(&self, root: &std::path::PathBuf) -> Result<()> {
		let to_file = WalletFile::from_wallet(self);

		to_file.save(root)
	}

	pub fn load(network: Network, endpoint: String, root: &std::path::PathBuf) -> Result<Self> {
		let from_file = WalletFile::load(root, network).context("load file failed")?;
		let xpriv = bip32::ExtendedPrivKey::from_str(from_file.xpriv.as_str()).unwrap();

		let (wallet, blockchain) = Self::load_wallet(
			network,
			endpoint,
			root,
			Bip84(xpriv, KeychainKind::External),
			Some(Bip84(xpriv, KeychainKind::Internal)),
		)
		.context("load wallet")?;

		Ok(Self { xprv: from_file.xpriv, wallet, blockchain })
	}

	fn load_wallet<E: IntoWalletDescriptor>(
		network: Network,
		endpoint: String,
		root: &std::path::PathBuf,
		descriptor: E,
		change_descriptor: Option<E>,
	) -> Result<(BdkWallet<AnyDatabase>, AnyBlockchain)> {
		let database = open_database(network, root).context("open_database")?;
		let blockchain = new_electrum_blockchain(endpoint).context("new_electrum_blockchain")?;

		let wallet = BdkWallet::new(descriptor, change_descriptor, network, database)?;

		wallet.sync(&blockchain, SyncOptions::default()).context("sync")?;

		Ok((wallet, blockchain))
	}
}
