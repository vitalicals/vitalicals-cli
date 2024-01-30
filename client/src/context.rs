//! The context for all cmds

use std::{collections::BTreeMap, str::FromStr};

use anyhow::{anyhow, bail, Context as AnyhowContext, Result};

use bdk::{
    bitcoin::{address::NetworkUnchecked, hashes::Hash as BdkHash, Address, Network},
    LocalUtxo,
};
use bitcoin::{hashes::Hash, OutPoint, Txid};

use vital_interfaces_indexer::{
    traits::IndexerClientT, vital_env_for_query, IndexerClient, QueryEnvContext,
};
use vital_script_primitives::resources::Resource;
use wallet::Wallet;

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
    pub outputs: Vec<(Option<Address>, u64)>,
}

impl Context {
    pub async fn new(root_path: std::path::PathBuf, indexer: &str, wallet: Wallet) -> Result<Self> {
        let indexer = IndexerClient::new(indexer).await?;
        let query_env_context = vital_env_for_query(indexer.clone());

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
        };

        // FIXME: the no use utxo should contains the pending utxo with resources!!!
        let utxo_with_resources =
            res.fetch_all_resources().await.context("get utxo with resources failed")?;

        for utxo in utxo_with_resources.into_iter() {
            res.utxo_with_resources.push(utxo.0.outpoint);
            res.utxo_resources.insert(utxo.1, utxo.0);
        }

        Ok(res)
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

    pub fn with_amount(mut self, amount: u64) -> Self {
        self.outputs = self
            .outputs
            .clone()
            .into_iter()
            .map(|(output, _)| (output, amount))
            .collect::<Vec<_>>();

        self
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

    pub async fn fetch_all_resources(&self) -> Result<Vec<(LocalUtxo, Resource)>> {
        let mut res = Vec::new();

        let outpoints = self.wallet.wallet.list_unspent().context("list unspents failed")?;

        for unspent in outpoints.into_iter() {
            log::debug!(
                "unspent {} - {:?} - {} - {}",
                unspent.is_spent,
                unspent.keychain,
                unspent.outpoint,
                unspent.txout.script_pubkey
            );
            let outpoint = unspent.outpoint;

            let resource = self
                .indexer
                .get_resource(&OutPoint {
                    txid: Txid::from_byte_array(*outpoint.txid.as_byte_array()),
                    vout: outpoint.vout,
                })
                .await?;
            if let Some(resource) = resource {
                log::debug!("find {} contain with resource {}", outpoint, resource);
                res.push((unspent, resource));
            }
        }

        Ok(res)
    }
}
