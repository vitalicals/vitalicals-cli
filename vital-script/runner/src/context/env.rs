use alloc::{collections::BTreeMap, vec::Vec};
use anyhow::{anyhow, bail, Context, Result};
use bitcoin::{OutPoint, Transaction, Txid};
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

    commit_tx: Transaction,
    commit_txid: Txid,

    /// The reveal_tx
    reveal_tx: Transaction,
    reveal_tx_id: Txid,

    ops: Vec<(u8, Vec<u8>)>,

    /// The outputs need to bind to outputs.
    cached_output_resources: BTreeMap<u8, Resource>,
}

impl<Functions: EnvFunctions> EnvContext<Functions> {
    pub fn new(env_interface: Functions, commit_tx: &Transaction, reveal_tx: &Transaction) -> Self {
        let ops = parse_vital_scripts(reveal_tx).expect("parse vital scripts");

        let commit_txid = commit_tx.txid();
        let reveal_tx_id = reveal_tx.txid();

        Self {
            env: env_interface,
            commit_tx: commit_tx.clone(),
            commit_txid,
            reveal_tx: reveal_tx.clone(),
            reveal_tx_id,
            ops,
            cached_output_resources: BTreeMap::new(),
        }
    }

    fn get_input(&self, input_index: u8) -> Result<OutPoint> {
        if self.commit_tx.input.len() <= input_index as usize {
            bail!("Input index out of range")
        }

        Ok(self.commit_tx.input[input_index as usize].previous_output)
    }
}

impl<Functions: EnvFunctions> EnvContextT for EnvContext<Functions> {
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
