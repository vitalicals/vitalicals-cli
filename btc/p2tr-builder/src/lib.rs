//! Psbt Builder for send p2tr commit tx and reveal tx

use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, Context, Result};
use bdk::{
    bitcoin::{
        absolute,
        bip32::{DerivationPath, ExtendedPrivKey},
        hashes::Hash,
        key::TapTweak,
        psbt::{Input, PartiallySignedTransaction, Psbt, PsbtSighashType},
        secp256k1::{All, KeyPair, Secp256k1, SecretKey, XOnlyPublicKey},
        sighash::{self, SighashCache, TapSighash, TapSighashType},
        taproot::{self, LeafVersion, TapLeafHash, TaprootBuilder, TaprootSpendInfo},
        Address, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Weight, Witness,
    },
    FeeRate, SignOptions,
};

use btc_script_builder::InscriptionScriptBuilder;
use wallet::Wallet;

pub struct P2trBuilder<'a> {
    to_address: Address,
    amount: u64,
    fee_rate: Option<FeeRate>,
    wallet: &'a Wallet,

    internal_key: XOnlyPublicKey,
    reveal_script: ScriptBuf,
    secp: Secp256k1<All>,
    master_xpriv: ExtendedPrivKey,
    derivation_path: DerivationPath,
}

impl<'a> P2trBuilder<'a> {
    pub fn new(
        data: Vec<u8>,
        to: Address,
        amount: u64,
        fee_rate: Option<f32>,
        wallet: &'a Wallet,
    ) -> Result<Self> {
        let secp = Secp256k1::new();

        let internal_key = wallet.derive_x_only_public_key(&secp)?;
        let reveal_script = InscriptionScriptBuilder::new(data)
            .into_script(&internal_key)
            .context("build script")?;

        let master_xpriv = *wallet.xpriv();
        let derivation_path = wallet.full_derivation_path().context("get full derivation")?;

        Ok(Self {
            reveal_script,
            to_address: to,
            amount,
            fee_rate: fee_rate.map(FeeRate::from_sat_per_vb),
            wallet,
            secp,
            internal_key,
            master_xpriv,
            derivation_path,
        })
    }

    fn secp(&self) -> &Secp256k1<All> {
        &self.secp
    }

    /// Generate a commit tx psbt
    /// For this psbt, we just need use the bdk wallet to build a simple transf to the
    /// reveal_script 's p2tr address.
    fn generate_commit_psbt(
        &self,
        commit_script_pubkey: ScriptBuf,
        fee_for_reveal_tx: Option<u64>,
    ) -> Result<(Psbt, OutPoint)> {
        let secp = self.secp();
        let bdk_wallet = &self.wallet.wallet;
        let internal_key = self.internal_key;
        let script_p2tr = self.reveal_script.to_v1_p2tr(secp, internal_key);

        // the total amount send to output, need the amount and fee for next tx.
        let amount_to_trans = self.amount + fee_for_reveal_tx.unwrap_or_default();

        let mut builder = bdk_wallet.build_tx();
        builder.set_recipients(vec![(commit_script_pubkey, amount_to_trans)]);

        if let Some(fee_rate) = &self.fee_rate {
            builder.fee_rate(*fee_rate);
        }

        let (mut psbt, _details) = builder.finish().context("build tx failed")?;

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

        // Broadcast the transaction
        let raw_transaction = psbt.clone().extract_tx();
        let txid = raw_transaction.txid();

        Ok((psbt, OutPoint::new(txid, index)))
    }

    fn generate_reveal_psbt(
        &self,
        commit_outpoint: OutPoint,
        commit_script_pubkey: ScriptBuf,
        taproot_spend_info: &TaprootSpendInfo,
        send_amount: u64,
    ) -> Result<Psbt> {
        let secp = self.secp();
        let internal_key = self.internal_key;

        let next_tx = Transaction {
            version: 1,
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: commit_outpoint,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD),
                witness: Witness::default(),
            }],
            output: vec![TxOut {
                value: self.amount,
                script_pubkey: self.to_address.script_pubkey(),
            }],
        };

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(next_tx)?;

        let leaf_hash = self.reveal_script.tapscript_leaf_hash();
        let mut origins = BTreeMap::new();
        origins.insert(
            internal_key,
            (vec![leaf_hash], (self.master_xpriv.fingerprint(secp), self.derivation_path.clone())),
        );

        let ty = PsbtSighashType::from_str("SIGHASH_ALL")?;
        let mut tap_scripts = BTreeMap::new();
        tap_scripts.insert(
            taproot_spend_info
                .control_block(&(self.reveal_script.clone(), LeafVersion::TapScript))
                .unwrap(),
            (self.reveal_script.clone(), LeafVersion::TapScript),
        );

        let input = Input {
            witness_utxo: {
                Some(TxOut { value: send_amount, script_pubkey: commit_script_pubkey })
            },
            tap_key_origins: origins,
            tap_merkle_root: taproot_spend_info.merkle_root(),
            sighash_type: Some(ty),
            tap_internal_key: Some(internal_key),
            tap_scripts,
            ..Default::default()
        };

        psbt.version = 1;
        psbt.inputs = vec![input];

        // Sign and finalize the PSBT with the signing wallet
        let unsigned_tx = psbt.unsigned_tx.clone();
        let input_value = psbt.inputs[0].witness_utxo.as_ref().unwrap().value;
        let input_script_pubkey =
            psbt.inputs[0].witness_utxo.as_ref().unwrap().script_pubkey.clone();

        // SIGNER
        for (x_only_pubkey, (leaf_hashes, (_, derivation_path))) in
            &psbt.inputs[0].tap_key_origins.clone()
        {
            let secret_key = self.master_xpriv.derive_priv(secp, &derivation_path)?.to_priv().inner;
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
                    secp,
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

        Ok(psbt)
    }

    pub fn build(self) -> Result<(Psbt, Psbt)> {
        let taproot_spend_info = TaprootBuilder::new()
            .add_leaf(0, self.reveal_script.clone())
            .context("TaprootBuilder add_leaf ")?
            .finalize(self.secp(), self.internal_key)
            .map_err(|_| anyhow!("TaprootBuilder error"))?;

        let commit_script_pubkey = ScriptBuf::new_v1_p2tr(
            self.secp(),
            taproot_spend_info.internal_key(),
            taproot_spend_info.merkle_root(),
        );

        // In first time, we need fee to be calculated.
        let reveal_psbt = self
            .generate_reveal_psbt(
                OutPoint::new(Txid::all_zeros(), 0),
                commit_script_pubkey.clone(),
                &taproot_spend_info.clone(),
                self.amount,
            )
            .context("generate_reveal_psbt")?;

        // fee for reveal
        let reveal_tx = reveal_psbt.extract_tx();
        let fee_rate = self.fee_rate.unwrap_or_default();

        // Segwit transactions' header is 2WU larger than legacy txs' header,
        // as they contain a witness marker (1WU) and a witness flag (1WU) (see BIP144).
        // At this point we really don't know if the resulting transaction will be segwit
        // or legacy, so we just add this 2WU to the fee_amount - overshooting the fee amount
        // is better than undershooting it.
        // If we pass a fee_amount that is slightly higher than the final fee_amount, we
        // end up with a transaction with a slightly higher fee rate than the requested one.
        // If, instead, we undershoot, we may end up with a feerate lower than the requested one
        // - we might come up with non broadcastable txs!
        let fee_for_reveal =
            fee_rate.fee_wu(reveal_tx.weight()) + fee_rate.fee_wu(Weight::from_wu(2));

        let (commit_psbt, commit_outpoint) = self
            .generate_commit_psbt(commit_script_pubkey.clone(), Some(fee_for_reveal))
            .context("generate_commit_psbt")?;

        let amount = self.amount + fee_for_reveal;

        let reveal_psbt = self
            .generate_reveal_psbt(
                commit_outpoint,
                commit_script_pubkey,
                &taproot_spend_info,
                amount,
            )
            .context("generate_reveal_psbt")?;

        Ok((commit_psbt, reveal_psbt))
    }
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
