use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use bdk::{
    bitcoin::{
        absolute,
        address::{NetworkUnchecked, Payload},
        key::TapTweak,
        psbt::{Input, PartiallySignedTransaction, PsbtSighashType},
        secp256k1::{All, KeyPair, Secp256k1, SecretKey, XOnlyPublicKey},
        sighash::{self, SighashCache, TapSighash, TapSighashType},
        taproot::{self, LeafVersion, TapLeafHash, TaprootBuilder},
        Address, Network, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Witness,
    },
    blockchain::Blockchain,
    wallet::AddressIndex,
    FeeRate, SignOptions,
};
use clap::Subcommand;

use btc_p2tr_builder::P2trBuilder;
use btc_script_builder::InscriptionScriptBuilder;

use crate::Cli;

#[derive(Debug, Subcommand)]
pub enum UtilsSubCommands {
    /// Send an amount to a given address.
    SendToAddress {
        /// The bitcoin address to send to.
        address: String,

        /// The sat amount in BTC to send.
        amount: u64,

        /// Specify a fee rate in sat/vB.
        #[arg(short, long)]
        fee_rate: Option<f32>,

        /// Signal that this transaction can be replaced by a transaction (BIP 125).
        #[arg(long)]
        replaceable: bool,
    },

    /// inscribe to a given address.
    InscribeToAddress {
        /// The bitcoin address to send to.
        #[arg(long)]
        address: Option<String>,

        /// The sat amount in BTC to send.
        amount: u64,

        /// Specify a fee rate in sat/vB.
        #[arg(short, long)]
        fee_rate: Option<f32>,

        /// Signal that this transaction can be replaced by a transaction (BIP 125).
        #[arg(long)]
        replaceable: bool,

        /// The datas to inscribe
        datas: String,
    },
}

impl UtilsSubCommands {
    pub(crate) async fn run(&self, cli: &Cli) -> Result<()> {
        let network = cli.network();

        match self {
            Self::SendToAddress { address, amount, fee_rate, replaceable } => {
                let address = Address::<NetworkUnchecked>::from_str(address)
                    .context("parse address failed")?
                    .require_network(network)
                    .context("the address is not for the network")?;

                send_to_address(network, cli, address, *amount, fee_rate, *replaceable)
            }
            Self::InscribeToAddress { address, amount, fee_rate, replaceable, datas } => {
                let context = crate::build_context(cli)
                    .await?
                    .with_to_address(address)
                    .context("with to address failed")?;

                inscribe_to_address(&context, *amount, fee_rate, *replaceable, datas.as_str()).await
            }
        }
    }
}

fn send_to_address(
    network: Network,
    cli: &Cli,
    address: Address,
    amount: u64,
    fee_rate: &Option<f32>,
    replaceable: bool,
) -> Result<()> {
    let wallet = wallet::Wallet::load(network, cli.endpoint.clone(), &cli.datadir)
        .context("load wallet failed")?;
    let bdk_wallet = &wallet.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let mut builder = bdk_wallet.build_tx();
    builder.set_recipients(vec![(address.script_pubkey(), amount)]);

    if replaceable {
        builder.enable_rbf();
    }

    if let Some(fee_rate) = fee_rate {
        builder.fee_rate(FeeRate::from_sat_per_vb(*fee_rate));
    }

    let (mut psbt, details) = builder.finish().context("build tx failed")?;
    println!("Transaction details: {:#?}", details);
    println!("Unsigned PSBT: {}", serde_json::to_string_pretty(&psbt)?);
    println!("Unsigned PSBT: {}", psbt);

    // Sign and finalize the PSBT with the signing wallet
    bdk_wallet.sign(&mut psbt, SignOptions::default())?;

    bdk_wallet.finalize_psbt(&mut psbt, SignOptions::default())?;

    println!("Signed PSBT: {}", serde_json::to_string_pretty(&psbt)?);
    println!("Signed PSBT: {}", psbt);

    // Broadcast the transaction
    let raw_transaction = psbt.extract_tx();
    let txid = raw_transaction.txid();

    bdk_blockchain.broadcast(&raw_transaction)?;
    println!("Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = txid);

    Ok(())
}

async fn inscribe_to_address(
    context: &crate::Context,
    amount: u64,
    fee_rate: &Option<f32>,
    _replaceable: bool,
    datas: &str,
) -> Result<()> {
    let wallet = &context.wallet;
    let bdk_wallet = &wallet.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let to_address = if let Some(to) = context.to_address.clone() {
        to
    } else {
        bdk_wallet.get_address(AddressIndex::New).context("new address")?.address
    };

    let utxo_with_resources =
        context.utxo_with_resources().await.context("utxo_with_resources failed")?;

    let builder = P2trBuilder::new(
        hex::decode(datas).context("decode datas")?,
        to_address,
        amount,
        *fee_rate,
        wallet,
        utxo_with_resources,
    )
    .context("builder build")?;

    let (commit_psbt, reveal_psbt) = builder.build().context("build tx error")?;

    let commit_raw_transaction = commit_psbt.extract_tx();
    let commit_txid = commit_raw_transaction.txid();

    println!("reveal_psbt: {}", serde_json::to_string_pretty(&reveal_psbt.unsigned_tx.input)?);
    println!("reveal_psbt: {}", serde_json::to_string_pretty(&reveal_psbt.inputs)?);

    let reveal_raw_transaction = reveal_psbt.extract_tx();
    println!(
        "reveal_psbt raw_transaction: {}",
        serde_json::to_string_pretty(&reveal_raw_transaction)?
    );

    let reveal_txid = reveal_raw_transaction.txid();

    bdk_blockchain.broadcast(&commit_raw_transaction)?;
    println!("Commit Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = commit_txid);

    bdk_blockchain.broadcast(&reveal_raw_transaction)?;
    println!("Reveal Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = reveal_txid);

    Ok(())
}

#[allow(dead_code)]
fn inscribe_to_address_impl(
    network: Network,
    cli: &Cli,
    to_address: Option<Address>,
    amount: u64,
    fee_rate: &Option<f32>,
    _replaceable: bool,
    datas: &str,
) -> Result<()> {
    let wallet = wallet::Wallet::load(network, cli.endpoint.clone(), &cli.datadir)
        .context("load wallet failed")?;
    let bdk_wallet = &wallet.wallet;
    let bdk_blockchain = &wallet.blockchain;

    let secp = Secp256k1::new();

    let master_xpriv = wallet.xpriv();
    let derivation_path = wallet.full_derivation_path().context("get full derivation")?;
    let internal_key = wallet.derive_x_only_public_key(&secp)?;

    let reveal_script = InscriptionScriptBuilder::new(hex::decode(datas).context("decode datas")?)
        .into_script_by_key(&internal_key)
        .context("build script")?;
    let script_p2tr = reveal_script.to_v1_p2tr(&secp, internal_key);

    println!("script_p2tr {}", script_p2tr);

    let to_address = if let Some(to) = to_address {
        to
    } else {
        bdk_wallet.get_address(AddressIndex::New).context("new address")?.address
    };

    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, reveal_script.clone())
        .context("TaprootBuilderadd_leaf ")?
        .finalize(&secp, internal_key)
        .map_err(|_| anyhow!("TaprootBuilder error"))?;

    let script_pubkey = ScriptBuf::new_v1_p2tr(
        &secp,
        taproot_spend_info.internal_key(),
        taproot_spend_info.merkle_root(),
    );

    let payload = Payload::p2tr(&secp, internal_key, taproot_spend_info.merkle_root());
    let commit_address = Address::new(network, payload);
    println!("to {} then to {}", commit_address, to_address);

    // commit transaction
    let commit_outpoint = {
        let mut builder = bdk_wallet.build_tx();
        builder.set_recipients(vec![(script_pubkey.clone(), amount)]);

        if let Some(fee_rate) = fee_rate {
            builder.fee_rate(FeeRate::from_sat_per_vb(*fee_rate));
        }

        let (mut psbt, _details) = builder.finish().context("build tx failed")?;

        println!("unsigned_tx PSBT: {}", serde_json::to_string_pretty(&psbt.unsigned_tx.output)?);
        println!("unsigned_tx PSBT: {}", serde_json::to_string_pretty(&psbt.outputs)?);

        let index = {
            let mut res = 0;

            let outputs = &psbt.unsigned_tx.output;
            for (index, output) in outputs.iter().enumerate() {
                if script_p2tr.to_string() == output.script_pubkey.to_string() {
                    res = index;
                }
            }

            res as u32
        };

        // Sign and finalize the PSBT with the signing wallet
        bdk_wallet.sign(&mut psbt, SignOptions::default())?;
        bdk_wallet.finalize_psbt(&mut psbt, SignOptions::default())?;

        println!("Signed PSBT: {}", serde_json::to_string_pretty(&psbt.unsigned_tx.output)?);
        println!("Signed PSBT: {}", serde_json::to_string_pretty(&psbt.outputs)?);

        // Broadcast the transaction
        let raw_transaction = psbt.extract_tx();
        println!("raw_transaction: {}", serde_json::to_string_pretty(&raw_transaction)?);
        let txid = raw_transaction.txid();

        bdk_blockchain.broadcast(&raw_transaction)?;
        println!("Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = txid);

        OutPoint::new(txid, index)
    };

    print!("output {}", commit_outpoint);

    // reveal the transaction
    {
        let next_tx = Transaction {
            version: 1,
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: commit_outpoint,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD),
                witness: Witness::default(),
            }],
            output: vec![TxOut { value: 800, script_pubkey: to_address.script_pubkey() }],
        };

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(next_tx)?;

        let leaf_hash = reveal_script.tapscript_leaf_hash();
        let mut origins = BTreeMap::new();
        origins.insert(
            internal_key,
            (vec![leaf_hash], (master_xpriv.fingerprint(&secp), derivation_path)),
        );

        let ty = PsbtSighashType::from_str("SIGHASH_ALL")?;
        let mut tap_scripts = BTreeMap::new();
        tap_scripts.insert(
            taproot_spend_info
                .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
                .unwrap(),
            (reveal_script, LeafVersion::TapScript),
        );

        let input = Input {
            witness_utxo: { Some(TxOut { value: amount, script_pubkey }) },
            tap_key_origins: origins,
            tap_merkle_root: taproot_spend_info.merkle_root(),
            sighash_type: Some(ty),
            tap_internal_key: Some(internal_key),
            tap_scripts,
            ..Default::default()
        };

        psbt.version = 1;
        psbt.inputs = vec![input];

        println!("unsigned_tx PSBT: {}", serde_json::to_string_pretty(&psbt.unsigned_tx.input)?);
        println!("unsigned_tx PSBT: {}", serde_json::to_string_pretty(&psbt.inputs)?);

        // Sign and finalize the PSBT with the signing wallet
        let unsigned_tx = psbt.unsigned_tx.clone();
        let input_value = psbt.inputs[0].witness_utxo.as_ref().unwrap().value;
        let input_script_pubkey =
            psbt.inputs[0].witness_utxo.as_ref().unwrap().script_pubkey.clone();

        // SIGNER
        for (x_only_pubkey, (leaf_hashes, (_, derivation_path))) in
            &psbt.inputs[0].tap_key_origins.clone()
        {
            let secret_key = master_xpriv.derive_priv(&secp, &derivation_path)?.to_priv().inner;
            for lh in leaf_hashes {
                let hash_ty = TapSighashType::All;
                let hash = SighashCache::new(&unsigned_tx).taproot_script_spend_signature_hash(
                    0,
                    &sighash::Prevouts::All(&[TxOut {
                        value: input_value,
                        script_pubkey: input_script_pubkey.clone(),
                    }]),
                    *lh,
                    hash_ty,
                )?;
                sign_psbt_taproot(
                    &secret_key,
                    *x_only_pubkey,
                    Some(*lh),
                    &mut psbt.inputs[0],
                    hash,
                    hash_ty,
                    &secp,
                );
            }
        }

        psbt.inputs.iter_mut().for_each(|input| {
            let mut script_witness: Witness = Witness::new();
            for (_, signature) in input.tap_script_sigs.iter() {
                script_witness.push(signature.to_vec());
            }
            for (control_block, (script, _)) in input.tap_scripts.iter() {
                script_witness.push(script.to_bytes());
                script_witness.push(control_block.serialize());
            }
            input.final_script_witness = Some(script_witness);

            // Clear all the data fields as per the spec.
            input.partial_sigs = BTreeMap::new();
            input.sighash_type = None;
            input.redeem_script = None;
            input.witness_script = None;
            input.bip32_derivation = BTreeMap::new();
            input.tap_script_sigs = BTreeMap::new();
            input.tap_scripts = BTreeMap::new();
            input.tap_key_sig = None;
        });

        println!("sign PSBT: {}", serde_json::to_string_pretty(&psbt.unsigned_tx.input)?);
        println!("sign PSBT: {}", serde_json::to_string_pretty(&psbt.inputs)?);

        // Broadcast the transaction
        let raw_transaction = psbt.extract_tx();

        println!("raw_transaction: {}", serde_json::to_string_pretty(&raw_transaction)?);

        let txid = raw_transaction.txid();

        bdk_blockchain.broadcast(&raw_transaction)?;
        println!("Transaction broadcast! TXID: {txid}.\nExplorer URL: https://mempool.space/testnet/tx/{txid}", txid = txid);
    }

    Ok(())
}

// Calling this with `leaf_hash` = `None` will sign for key-spend
fn sign_psbt_taproot(
    secret_key: &SecretKey,
    pubkey: XOnlyPublicKey,
    leaf_hash: Option<TapLeafHash>,
    psbt_input: &mut Input,
    hash: TapSighash,
    hash_ty: TapSighashType,
    secp: &Secp256k1<All>,
) {
    let keypair = KeyPair::from_seckey_slice(secp, secret_key.as_ref()).unwrap();
    let keypair = match leaf_hash {
        None => keypair.tap_tweak(secp, psbt_input.tap_merkle_root).to_inner(),
        Some(_) => keypair, // no tweak for script spend
    };

    let sig = secp.sign_schnorr(&hash.into(), &keypair);

    let final_signature = taproot::Signature { sig, hash_ty };

    if let Some(lh) = leaf_hash {
        psbt_input.tap_script_sigs.insert((pubkey, lh), final_signature);
    } else {
        psbt_input.tap_key_sig = Some(final_signature);
    }
}
