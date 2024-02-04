//! The context for all cmds

use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use anyhow::{anyhow, bail, Context as AnyhowContext, Result};

use bdk::{
    bitcoin::{
        address::NetworkUnchecked, hashes::Hash as BdkHash, Address, Network, OutPoint, Transaction,
    },
    blockchain::GetHeight,
    database::Database,
    LocalUtxo,
};
use bitcoin::{hashes::Hash, Txid};

use vital_interfaces_indexer::{
    traits::IndexerClientT, vital_env_for_query, IndexerClient, QueryEnvContext,
};
use vital_script_primitives::{
    resources::{Name, Resource, ResourceType},
    U256,
};
use vital_script_runner::*;
use wallet::Wallet;

use crate::{
    parser::tx_from_bdk,
    resource::LocalResource,
    used_utxo::{append_used_utxos, load_used_utxos, save_used_utxos},
    vital_script_runner::LocalRunner,
};

pub struct Context {
    pub root_path: std::path::PathBuf,
    pub wallet: Wallet,
    pub indexer: IndexerClient,
    pub query_env_context: QueryEnvContext,
    pub fee_rate: Option<f32>,
    /// TODO: support replaceable
    pub replaceable: bool,
    pub utxo_resources: BTreeMap<Resource, LocalUtxo>,
    pub utxo_with_resources: Vec<bdk::bitcoin::OutPoint>,
    pub reveal_inputs: Vec<LocalUtxo>,
    pub used_utxos: Vec<bdk::bitcoin::OutPoint>,
    pub outputs: Vec<(Option<Address>, u64)>,
    pub sats_amount: u64,
}

impl Context {
    pub async fn new(root_path: std::path::PathBuf, indexer: &str, wallet: Wallet) -> Result<Self> {
        let indexer = IndexerClient::new(indexer).await?;
        let block_height = wallet.blockchain.get_height().context("get height")?;
        let query_env_context = vital_env_for_query(indexer.clone(), block_height);

        let used_utxos = if wallet.synced {
            let empty = Vec::new();
            save_used_utxos(&root_path, &empty).expect("save failed");
            empty
        } else {
            load_used_utxos(&root_path).expect("load failed")
        };

        let mut res = Self {
            root_path,
            wallet,
            indexer,
            query_env_context,
            fee_rate: None,
            replaceable: false,
            utxo_with_resources: Vec::new(),
            utxo_resources: Default::default(),
            reveal_inputs: Vec::new(),
            // At least one outputs
            outputs: vec![(None, 0)],
            sats_amount: 0,
            used_utxos,
        };

        let utxo_with_resources =
            res.fetch_all_resources().await.context("get utxo with resources failed")?;

        for utxo in utxo_with_resources.into_iter() {
            res.utxo_with_resources.push(utxo.utxo.outpoint);
            res.utxo_resources.insert(utxo.resource, utxo.utxo);
        }

        Ok(res)
    }

    pub fn append_used_utxos(&self, utxos: &[OutPoint]) -> Result<()> {
        append_used_utxos(&self.root_path, utxos)
    }

    pub fn with_fee_rate(mut self, fee_rate: &Option<f32>) -> Self {
        self.fee_rate = *fee_rate;
        self
    }

    pub fn with_replaceable(mut self, replaceable: &bool) -> Self {
        self.replaceable = *replaceable;
        self
    }

    pub fn with_to_address(mut self, to: &Option<impl ToString>) -> Result<Self> {
        if let Some(to) = to {
            let to = Address::<NetworkUnchecked>::from_str(to.to_string().as_str())
                .context("parse address failed")?
                .require_network(self.network())
                .context("the address is not for the network")?;

            self.outputs = self
                .outputs
                .clone()
                .into_iter()
                .map(|(_, amount)| (Some(to.clone()), amount))
                .collect::<Vec<_>>();
        }

        Ok(self)
    }

    pub fn with_sats_amount(mut self, amount: u64) -> Self {
        self.outputs = self
            .outputs
            .clone()
            .into_iter()
            .map(|(output, _)| (output, amount))
            .collect::<Vec<_>>();
        self.sats_amount = amount;

        self
    }

    pub fn set_amount(&mut self, amount: u64) {
        self.outputs = self
            .outputs
            .clone()
            .into_iter()
            .map(|(output, _)| (output, amount))
            .collect::<Vec<_>>();
        self.sats_amount = amount;
    }

    pub fn with_reveal_input(mut self, reveal_inputs: &[LocalUtxo]) -> Self {
        self.reveal_inputs = reveal_inputs.to_vec();

        self
    }

    pub fn append_reveal_input(&mut self, reveal_inputs: &[LocalUtxo]) {
        self.reveal_inputs.append(&mut reveal_inputs.to_vec());
    }

    pub fn set_outputs(&mut self, outputs: &[(Option<Address>, u64)]) {
        self.outputs = outputs.to_vec();
    }

    pub fn append_output(&mut self, to: Option<Address>, amount: u64) {
        self.outputs.push((to, amount));
    }

    pub fn set_outputs_from(&mut self, from: u32, count: usize, amount: u64) -> Result<()> {
        let to = self
            .outputs
            .first()
            .ok_or_else(|| anyhow!("the output should not null"))?
            .clone();

        let mut outputs = Vec::with_capacity(count);

        for i in from..from + count as u32 {
            if i >= u8::MAX as u32 {
                bail!("the output index too large for {}, should less then {}", i, u8::MAX);
            }

            outputs.push((to.0.clone(), amount));
        }

        self.outputs = outputs;

        Ok(())
    }

    pub fn network(&self) -> Network {
        self.wallet.wallet.network()
    }

    pub fn get_owned_resource(&self, resource: &Resource) -> Option<LocalUtxo> {
        self.utxo_resources.get(resource).cloned()
    }

    pub async fn run_tx_in_local(
        &self,
        block_height: u32,
        tx: Transaction,
    ) -> Result<Option<Vec<(OutPoint, Resource)>>> {
        let txid = tx.txid();
        let tx = tx_from_bdk(tx);
        if !check_is_vital_script(&tx) {
            return Ok(None);
        }

        let runner = LocalRunner::new(self);
        let res = runner.run(block_height, &tx).await?;

        Ok(Some(
            res.into_iter()
                .map(|(index, resource)| (OutPoint::new(txid, index as u32), resource))
                .collect::<Vec<_>>(),
        ))
    }

    pub async fn try_get_pending_resources(
        &self,
        unspents: &[LocalUtxo],
    ) -> Result<Vec<LocalResource>> {
        let mut resource_pendings = Vec::new();

        let db = self.wallet.wallet.database();

        let mut processed_tx = BTreeSet::new();
        let block_height = self.wallet.blockchain.get_height().context("get block height")?;

        // process pendings
        for unspent in unspents.iter() {
            let txid = unspent.outpoint.txid;
            if processed_tx.contains(&txid) {
                continue;
            }
            processed_tx.insert(txid);

            let tx = db.get_tx(&txid, true).context("get_tx")?;
            if let Some(tx) = tx {
                log::debug!("unspent tx: {} for {:?}", unspent.outpoint, tx.confirmation_time);
                if tx.confirmation_time.is_none() {
                    // process pendings
                    log::debug!(
                        "unspent tx details: {} for {:?}",
                        unspent.outpoint,
                        tx.transaction
                    );

                    let tx = if let Some(tx) = tx.transaction {
                        tx
                    } else {
                        continue;
                    };

                    let outputs =
                        self.run_tx_in_local(block_height, tx).await.context("run_tx_in_local");

                    match outputs {
                        Ok(Some(mut outputs)) => {
                            resource_pendings.append(&mut outputs);
                        }
                        Err(err) => {
                            log::warn!("the pending tx 's vital script run failed by {}", err);
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut res = Vec::with_capacity(resource_pendings.len());

        for (outpoint, resource) in resource_pendings.into_iter() {
            let local = db
                .get_utxo(&outpoint)
                .context("get utxo")?
                .ok_or_else(|| anyhow!("the outpoint from pending should get"))?;

            res.push(LocalResource { utxo: local, resource, pending: true })
        }

        Ok(res)
    }

    pub async fn fetch_all_resources(&self) -> Result<Vec<LocalResource>> {
        let mut res = Vec::new();

        let outpoints = self.wallet.wallet.list_unspent().context("list unspents failed")?;

        // process pendings
        let mut pending_resources = self
            .try_get_pending_resources(&outpoints)
            .await
            .context("try_get_pending_resources")?;

        for unspent in outpoints.into_iter() {
            log::debug!(
                "unspent {} - {:?} - {} - {}",
                unspent.is_spent,
                unspent.keychain,
                unspent.outpoint,
                unspent.txout.value,
            );
            let outpoint = unspent.outpoint;

            let resource = self
                .indexer
                .get_resource(&bitcoin::OutPoint {
                    txid: Txid::from_byte_array(*outpoint.txid.as_byte_array()),
                    vout: outpoint.vout,
                })
                .await?;
            if let Some(resource) = resource {
                log::debug!("find {} contain with resource {}", outpoint, resource);
                res.push(LocalResource { utxo: unspent, resource, pending: false });
            }
        }

        res.append(&mut pending_resources);

        Ok(res)
    }

    pub async fn fetch_all_vrc20_by_name(&self, name: Name) -> Result<(U256, Vec<LocalResource>)> {
        let resource_type = ResourceType::vrc20(name);

        let owned_vrc20s = self
            .fetch_all_resources()
            .await
            .context("fetch all resources")?
            .into_iter()
            .filter(|local| local.resource.resource_type() == resource_type)
            .collect::<Vec<_>>();

        let mut sum = U256::zero();
        for local in owned_vrc20s.iter() {
            let v = local.resource.as_vrc20().context("not vrc20")?;
            sum += v.amount;
        }

        Ok((sum, owned_vrc20s))
    }

    pub fn get_btc_block_height(&self) -> Result<u32> {
        self.wallet.blockchain.get_height().context("get height")
    }
}
