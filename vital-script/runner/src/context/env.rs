use alloc::{collections::BTreeMap, vec::Vec};
use anyhow::{anyhow, bail, Context, Result};
use bitcoin::{hashes::Hash, OutPoint, Transaction, Txid};
use parity_scale_codec::{Decode, Encode};

use vital_script_primitives::{
    resources::{Resource, Tag},
    traits::{context::EnvContext as EnvContextT, MetaDataType},
};

use crate::{traits::EnvFunctions, TARGET};

use super::script::parse_vital_scripts;

const STORAGE_KEY_METADATA: &[u8; 8] = b"metadata";

#[derive(Clone)]
pub struct EnvContext<Functions: EnvFunctions> {
    env: Functions,

    /// The block height
    block_height: u32,

    /// The reveal_tx
    reveal_tx_id: Txid,
    inputs: Vec<OutPoint>,

    ops: Vec<(u8, Vec<u8>)>,

    /// The outputs need to bind to outputs.
    cached_output_resources: BTreeMap<u8, Resource>,
}

impl<Functions: EnvFunctions> EnvContext<Functions> {
    pub fn new(
        env_interface: Functions,
        inputs: Vec<OutPoint>,
        reveal_tx: &Transaction,
        block_height: u32,
    ) -> Self {
        let ops = parse_vital_scripts(reveal_tx).expect("parse vital scripts");

        let reveal_tx_id = reveal_tx.txid();

        Self {
            env: env_interface,
            inputs,
            reveal_tx_id,
            block_height,
            ops,
            cached_output_resources: BTreeMap::new(),
        }
    }

    pub fn new_for_sim(
        env_interface: Functions,
        reveal_tx: &Transaction,
        block_height: u32,
    ) -> Self {
        Self::new(env_interface, Vec::new(), reveal_tx, block_height)
    }

    pub fn new_for_query(env_interface: Functions, block_height: u32) -> Self {
        Self {
            env: env_interface,
            inputs: Default::default(),
            reveal_tx_id: Txid::all_zeros(),
            ops: Default::default(),
            cached_output_resources: BTreeMap::new(),
            block_height,
        }
    }

    fn get_input(&self, input_index: u8) -> Result<OutPoint> {
        Ok(self.inputs[input_index as usize])
    }
}

impl<Functions: EnvFunctions> EnvContextT for EnvContext<Functions> {
    fn get_block_height(&self) -> u32 {
        self.block_height
    }

    /// get current tx id.
    fn get_reveal_tx_id(&self) -> &Txid {
        &self.reveal_tx_id
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn get_ops(&self) -> &[(u8, Vec<u8>)] {
        &self.ops
    }

    fn get_input_resource(&self, index: u8) -> Result<Resource> {
        log::debug!(target: TARGET, "get_input_resource {}", index);

        let out_point = self.get_input(index).context("get input")?;

        log::debug!(target: TARGET, "get_input_resource for {}", out_point);

        let res = self
            .env
            .get_resources(&out_point)
            .context("get")?
            .ok_or_else(|| anyhow!("not found {} {}", index, out_point))?;

        log::debug!(target: TARGET, "get_input_resource {} {}", index, res);

        Ok(res)
    }

    fn get_output_resource(&self, index: u8) -> Option<&Resource> {
        let res = self.cached_output_resources.get(&index);

        log::debug!(target: TARGET, "get_output_resource {} {:?}", index, res);

        res
    }

    fn set_resource_to_output(&mut self, index: u8, resource: Resource) -> Result<()> {
        if let Some(caches) = self.cached_output_resources.get_mut(&index) {
            caches.merge(&resource)?;
        } else {
            self.cached_output_resources.insert(index, resource);
        }

        Ok(())
    }

    fn remove_input_resources(&self, input_indexs: &[u8]) -> Result<()> {
        for index in input_indexs.iter() {
            log::debug!(target: TARGET, "remove_input_resources {}", index);

            self.env
                .unbind_resource(
                    &self
                        .get_input(*index)
                        .with_context(|| alloc::format!("get input index {}", index))?,
                )
                .with_context(|| alloc::format!("unbind {}", index))?;
        }

        Ok(())
    }

    fn apply_output_resources(&mut self) -> Result<()> {
        for (index, resource) in self.cached_output_resources.iter() {
            log::debug!(target: TARGET, "apply_output_resources {} {}", index, resource);
            self.env
                .bind_resource(self.get_output(*index), resource.clone())
                .with_context(|| alloc::format!("bind resource {} to {:?}", index, resource))?;
        }

        Ok(())
    }

    fn set_metadata<T: Encode>(&mut self, name: Tag, typ: MetaDataType, meta: T) -> Result<()> {
        log::debug!(target: TARGET, "set metadata {} {:?}", name, typ);

        let key = [STORAGE_KEY_METADATA.to_vec(), [typ as u8].to_vec(), name.0.to_vec()].concat();
        let value = (typ as u8, meta).encode();

        self.env.storage_set(key, value).context("set metadata failed")
    }

    fn get_metadata<T: Decode>(&self, name: Tag, typ: MetaDataType) -> Result<Option<T>> {
        log::debug!(target: TARGET, "get metadata {} {:?}", name, typ);

        let key = [STORAGE_KEY_METADATA.to_vec(), [typ as u8].to_vec(), name.0.to_vec()].concat();

        let value = self.env.storage_get(&key).context("get metadata failed")?;
        if let Some((typ_in_storage, res)) = value
            .map(|datas| <(u8, T)>::decode(&mut datas.as_slice()))
            .transpose()
            .map_err(|err| anyhow!("decode failed by {:?}", err))?
        {
            if typ_in_storage != typ as u8 {
                bail!("the type not match expected {}, got {}", typ as u8, typ_in_storage);
            }

            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}
