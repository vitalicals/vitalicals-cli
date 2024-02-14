//! Psbt Builder for send p2tr commit tx and reveal tx

mod coin_selector;

use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use bdk::{
    bitcoin::{
        absolute,
        bip32::{DerivationPath, ExtendedPrivKey},
        hashes::Hash,
        key::TapTweak,
        psbt::{self, Input, PartiallySignedTransaction, Psbt, PsbtSighashType},
        secp256k1::{All, KeyPair, Secp256k1, SecretKey, XOnlyPublicKey},
        sighash::{self, TapSighash, TapSighashType},
        taproot::{self, LeafVersion, TapLeafHash, TaprootBuilder, TaprootSpendInfo},
        Address, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid, Weight, Witness,
    },
    psbt::PsbtUtils,
    wallet::AddressIndex,
    FeeRate, LocalUtxo, SignOptions,
};

use btc_script_builder::InscriptionScriptBuilder;

use client::context::Context;
use wallet::Wallet;

pub struct P2trBuilder<'a> {
    fee_rate: Option<FeeRate>,
    wallet: &'a Wallet,

    internal_key: XOnlyPublicKey,
    reveal_script: ScriptBuf,
    secp: Secp256k1<All>,
    master_xpriv: ExtendedPrivKey,
    derivation_path: DerivationPath,

    no_alive_utxos: Vec<OutPoint>,

    reveal_inputs: Vec<LocalUtxo>,
    outputs: Vec<(Address, u64)>,
}

impl<'a> P2trBuilder<'a> {
    pub fn new(context: &'a Context, data: Vec<u8>) -> Result<Self> {
        let secp = Secp256k1::new();

        let wallet = &context.wallet;
        let no_alive_utxos =
            [context.utxo_with_resources.clone(), context.used_utxos.clone()].concat();

        let outputs = context
            .outputs
            .iter()
            .map(|(to, amount)| {
                let to_address = if let Some(to) = to.clone() {
                    to
                } else {
                    wallet.wallet.get_address(AddressIndex::New).context("new address")?.address
                };

                Ok((to_address, *amount))
            })
            .collect::<Result<Vec<_>>>()?;

        let internal_key = wallet.derive_x_only_public_key(&secp)?;
        let reveal_script = InscriptionScriptBuilder::new(data)
            .into_script_by_key(&internal_key)
            .context("build script")?;

        let master_xpriv = *wallet.xpriv();
        let derivation_path = wallet.full_derivation_path().context("get full derivation")?;

        Ok(Self {
            reveal_script,
            fee_rate: context.fee_rate.map(FeeRate::from_sat_per_vb),
            wallet,
            secp,
            internal_key,
            master_xpriv,
            derivation_path,
            no_alive_utxos,
            reveal_inputs: context.reveal_inputs.clone(),
            outputs,
        })
    }

    fn secp(&self) -> &Secp256k1<All> {
        &self.secp
    }

    fn amount(&self) -> u64 {
        self.outputs.iter().map(|(_, amount)| amount).sum()
    }

    pub fn with_reveal_input(mut self, utxo: LocalUtxo) -> Self {
        self.reveal_inputs.push(utxo);

        self
    }

    /// TODO: support merge amount to output mod.
    #[allow(dead_code)]
    fn reveal_input_amount(&self) -> u64 {
        let mut amount = 0;
        for i in self.reveal_inputs.iter() {
            amount += i.txout.value;
        }
        amount
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
        // TODO: support merge amount to output mod.
        let amount_to_trans = self.amount() + fee_for_reveal_tx.unwrap_or_default();

        let mut builder = bdk_wallet
            .build_tx()
            .coin_selection(coin_selector::CoinSelector::new(Vec::new()));
        builder.ordering(bdk::wallet::tx_builder::TxOrdering::Untouched);

        builder.set_recipients(vec![(commit_script_pubkey, amount_to_trans)]);
        // Note we not use this utxos, because it will cost the resource.
        builder.unspendable(self.no_alive_utxos.clone());

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

        for i in self.reveal_inputs.iter().enumerate() {
            println!("reveal input {} : for {}", i.0, i.1.outpoint);
        }
        println!("commit tx {}", serde_json::to_string_pretty(&raw_transaction).unwrap());

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

        let output = self
            .outputs
            .iter()
            .map(|(to, amount)| TxOut { value: *amount, script_pubkey: to.script_pubkey() })
            .collect::<Vec<_>>();

        let mut input = vec![TxIn {
            previous_output: commit_outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0xFFFFFFFD),
            witness: Witness::default(),
        }];

        for resource_input in self.reveal_inputs.iter() {
            input.push(TxIn {
                previous_output: resource_input.outpoint,
                script_sig: ScriptBuf::new(),
                sequence: Sequence(0xFFFFFFFD),
                witness: Witness::default(),
            });
        }

        let next_tx =
            Transaction { version: 1, lock_time: absolute::LockTime::ZERO, input, output };

        let mut psbt = PartiallySignedTransaction::from_unsigned_tx(next_tx)?;

        // clean the inputs, will push by update func
        psbt.inputs = Vec::with_capacity(psbt.unsigned_tx.input.len());

        self.update_psbt_taproot_input(
            &mut psbt,
            secp,
            taproot_spend_info,
            commit_script_pubkey,
            send_amount,
        )
        .context("update_psbt_taproot_input")?;

        self.update_psbt_resource_inputs(&mut psbt)
            .context("update_psbt_resource_inputs")?;

        psbt.version = 1;

        // println!("psbt input {}", serde_json::to_string_pretty(&psbt.unsigned_tx.input).unwrap());
        // println!("psbt input update {}", serde_json::to_string_pretty(&psbt.inputs).unwrap());

        self.sign_psbt_inputs(&mut psbt, secp).context("sign_psbt_inputs")?;

        println!(
            "psbt input sign_psbt_inputs {}",
            serde_json::to_string_pretty(&psbt.inputs).unwrap()
        );

        psbt.inputs.iter_mut().for_each(|input| {
            let mut script_witness: Witness = Witness::new();
            if let Some(tap_key_sig) = input.tap_key_sig {
                script_witness.push(tap_key_sig.to_vec());
            }
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

    fn taproot_spend_info(&self, script: ScriptBuf) -> Result<TaprootSpendInfo> {
        let taproot_spend_info = TaprootBuilder::new()
            .add_leaf(0, script)
            .context("TaprootBuilder add_leaf ")?
            .finalize(self.secp(), self.internal_key)
            .map_err(|_| anyhow!("TaprootBuilder error"))?;

        Ok(taproot_spend_info)
    }

    pub fn build(self) -> Result<(Psbt, Psbt)> {
        let taproot_spend_info = self
            .taproot_spend_info(self.reveal_script.clone())
            .context("taproot_spend_info for vital script input")?;

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
                self.amount(),
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

        let amount = self.amount() + fee_for_reveal;

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

    /// this will push taproot input into index 0 for vital script
    fn update_psbt_taproot_input(
        &self,
        psbt: &mut PartiallySignedTransaction,
        secp: &Secp256k1<All>,
        taproot_spend_info: &TaprootSpendInfo,
        commit_script_pubkey: ScriptBuf,
        send_amount: u64,
    ) -> Result<()> {
        let internal_key = self.internal_key;

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

        psbt.inputs.push(input);

        Ok(())
    }

    /// this will push taproot input from index 1 for vital script
    fn update_psbt_resource_inputs(&self, psbt: &mut PartiallySignedTransaction) -> Result<()> {
        for input_index in 1..psbt.unsigned_tx.input.len() {
            let input = &psbt.unsigned_tx.input[input_index];
            let reveal_input = &self.reveal_inputs[input_index - 1];

            assert_eq!(input.previous_output, reveal_input.outpoint);

            let psbt_input = self
                .wallet
                .wallet
                .get_psbt_input(reveal_input.clone(), None, false)
                .context("get_psbt_input")?;
            // println!("psbt_input {}", serde_json::to_string_pretty(&psbt_input).unwrap());

            psbt.inputs.push(psbt_input);
        }

        Ok(())
    }

    fn sighash(
        psbt: &PartiallySignedTransaction,
        input_index: usize,
        extra: Option<taproot::TapLeafHash>,
    ) -> Result<(TapSighash, TapSighashType)> {
        if input_index >= psbt.inputs.len() || input_index >= psbt.unsigned_tx.input.len() {
            bail!("InputIndexOutOfRange");
        }

        let psbt_input = &psbt.inputs[input_index];

        let sighash_type = psbt_input
            .sighash_type
            .unwrap_or_else(|| TapSighashType::Default.into())
            .taproot_hash_ty()
            .map_err(|_| anyhow!("InvalidSighash"))?;
        let witness_utxos =
            (0..psbt.inputs.len()).map(|i| psbt.get_utxo_for(i)).collect::<Vec<_>>();
        let mut all_witness_utxos = vec![];

        let mut cache = sighash::SighashCache::new(&psbt.unsigned_tx);
        let is_anyone_can_pay = psbt::PsbtSighashType::from(sighash_type).to_u32() & 0x80 != 0;
        let prevouts = if is_anyone_can_pay {
            sighash::Prevouts::One(
                input_index,
                witness_utxos[input_index].as_ref().ok_or(anyhow!("MissingWitnessUtxo"))?,
            )
        } else if witness_utxos.iter().all(Option::is_some) {
            all_witness_utxos.extend(witness_utxos.iter().filter_map(|x| x.as_ref()));
            sighash::Prevouts::All(&all_witness_utxos)
        } else {
            return Err(anyhow!("MissingWitnessUtxo"));
        };

        // Assume no OP_CODESEPARATOR
        let extra = extra.map(|leaf_hash| (leaf_hash, 0xFFFFFFFF));

        Ok((
            cache.taproot_signature_hash(input_index, &prevouts, None, extra, sighash_type)?,
            sighash_type,
        ))
    }

    fn sign_psbt_inputs(
        &self,
        psbt: &mut PartiallySignedTransaction,
        secp: &Secp256k1<All>,
    ) -> Result<()> {
        // Sign and finalize the PSBT with the signing wallet
        let internal_key = self.internal_key;

        // SIGNER
        for (leaf_hashes, (_, path)) in psbt.inputs[0].tap_key_origins.clone().values() {
            let secret_key = self.master_xpriv.derive_priv(secp, path)?.to_priv().inner;

            for lh in leaf_hashes {
                let (hash, hash_ty) = Self::sighash(psbt, 0, Some(*lh)).context("sighash")?;

                sign_psbt_taproot(
                    &secret_key,
                    internal_key,
                    Some(*lh),
                    &mut psbt.inputs[0],
                    hash,
                    hash_ty,
                    secp,
                );
            }
        }

        for input_index in 1..psbt.inputs.len() {
            for (key, (_, (_, path))) in &psbt.inputs[input_index].tap_key_origins.clone() {
                let (hash, hash_ty) = Self::sighash(psbt, input_index, None).context("sighash")?;

                let secret_key = self.master_xpriv.derive_priv(secp, path)?.to_priv().inner;

                sign_psbt_taproot(
                    &secret_key,
                    *key,
                    None,
                    &mut psbt.inputs[input_index],
                    hash,
                    hash_ty,
                    secp,
                );
            }
        }

        Ok(())
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
    use bdk::bitcoin::secp256k1::Message;

    let keypair = KeyPair::from_seckey_slice(secp, secret_key.as_ref()).unwrap();
    let keypair = match leaf_hash {
        None => keypair.tap_tweak(secp, psbt_input.tap_merkle_root).to_inner(),
        Some(_) => keypair, // no tweak for script spend
    };

    let msg = &Message::from(hash);
    let sig = secp.sign_schnorr(msg, &keypair);
    secp.verify_schnorr(&sig, msg, &XOnlyPublicKey::from_keypair(&keypair).0)
        .expect("invalid or corrupted schnorr signature");

    let final_signature = taproot::Signature { sig, hash_ty };

    if let Some(lh) = leaf_hash {
        psbt_input.tap_script_sigs.insert((pubkey, lh), final_signature);
    } else {
        psbt_input.tap_key_sig = Some(final_signature);
    }
}
