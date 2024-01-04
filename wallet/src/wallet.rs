//! The wallet wrapper implementation by bdk

use anyhow::{anyhow, bail, Context, Result};
use std::str::FromStr;

use bdk::{
	bitcoin::{
		bip32::{self, DerivationPath, ExtendedPrivKey},
		secp256k1::{All, Secp256k1, XOnlyPublicKey},
		Network,
	},
	blockchain::AnyBlockchain,
	database::AnyDatabase,
	descriptor::IntoWalletDescriptor,
	keys::{
		bip39::{Language, Mnemonic, WordCount},
		DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey,
	},
	miniscript::{self, Descriptor},
	template::Bip86,
	KeychainKind, SyncOptions, Wallet as BdkWallet,
};

use crate::{database::*, file::WalletFile};

/// Wallet
pub struct Wallet {
	pub xprv: String,
	pub xpriv: ExtendedPrivKey,
	pub wallet: BdkWallet<AnyDatabase>,
	pub blockchain: AnyBlockchain,
}

impl Wallet {
	pub fn create_from_wallet(
		xprv: String,
		wallet: BdkWallet<AnyDatabase>,
		blockchain: AnyBlockchain,
	) -> Result<Self> {
		let xpriv = ExtendedPrivKey::from_str(xprv.as_str()).context("ExtendedPrivKey from str")?;
		Ok(Self { xprv, xpriv, wallet, blockchain })
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
			Bip86(xprv, KeychainKind::External),
			Some(Bip86(xprv, KeychainKind::Internal)),
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
			Bip86(xpriv, KeychainKind::External),
			Some(Bip86(xpriv, KeychainKind::Internal)),
		)
		.context("load wallet")?;

		Ok(Self { xpriv, xprv: from_file.xpriv, wallet, blockchain })
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

impl Wallet {
	pub fn full_derivation_path(&self) -> Result<DerivationPath> {
		let descriptor = self.wallet.get_descriptor_for_keychain(KeychainKind::External);

		let tr = match descriptor {
			Descriptor::Tr(tr) => tr,
			_ => bail!("not tr descriptor"),
		};
		let derivation_path = tr.internal_key().full_derivation_path().unwrap();

		Ok(derivation_path)
	}

	pub fn xpriv(&self) -> &ExtendedPrivKey {
		&self.xpriv
	}

	pub fn derive_x_only_public_key(&self, secp: &Secp256k1<All>) -> Result<XOnlyPublicKey> {
		let derivation_path = self.full_derivation_path().context("get full derivation")?;
		let (internal_key, _) = self
			.xpriv
			.derive_priv(secp, &derivation_path)?
			.to_keypair(secp)
			.x_only_public_key();

		Ok(internal_key)
	}

	pub fn network(&self) -> Network {
		self.wallet.network()
	}
}
