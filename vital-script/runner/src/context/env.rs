use alloc::collections::BTreeMap;
use anyhow::{anyhow, bail, Context, Result};
use parity_scale_codec::{Decode, Encode};

use vital_script_primitives::{
    resources::{Resource, Tag},
    traits::{context::EnvContext as EnvContextT, MetaDataType},
};

use crate::traits::EnvFunctions;

const STORAGE_KEY_METADATA: &'static [u8; 8] = b"metadata";

pub struct EnvContext<Functions: EnvFunctions> {
    env: Functions,

    /// The outputs need to bind to outputs.
    cached_output_resources: BTreeMap<u8, Resource>,
}

impl<Functions: EnvFunctions> EnvContext<Functions> {
    pub fn new(env_interface: Functions) -> Self {
        Self { env: env_interface, cached_output_resources: BTreeMap::new() }
    }
}

impl<Functions: EnvFunctions> EnvContextT for EnvContext<Functions> {
    fn get_ops(&self) -> &[(u8, Vec<u8>)] {
        self.env.get_ops()
    }

    fn get_input_resource(&self, index: u8) -> Result<Resource> {
        let out_point = self.env.get_input(index).context("get input")?;

        self.env
            .get_resources(&out_point)
            .context("get")?
            .ok_or_else(|| anyhow!("not found {} {}", index, out_point))
    }

    fn get_output_resource(&self, index: u8) -> Option<&Resource> {
        self.cached_output_resources.get(&index)
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
            self.env
                .unbind_resource(
                    &self
                        .env
                        .get_input(*index)
                        .with_context(|| format!("get input index {}", index))?,
                )
                .with_context(|| format!("unbind {}", index))?;
        }

        Ok(())
    }

    fn apply_output_resources(&mut self) -> Result<()> {
        for (index, resource) in self.cached_output_resources.iter() {
            self.env
                .bind_resource(self.env.get_output(*index).context("get output")?, resource.clone())
                .with_context(|| format!("bind resource {} to {:?}", index, resource))?;
        }

        Ok(())
    }

    fn set_metadata<T: Encode>(&mut self, name: Tag, typ: MetaDataType, meta: T) -> Result<()> {
        let key = [STORAGE_KEY_METADATA.to_vec(), [typ as u8].to_vec(), name.0.to_vec()].concat();
        let value = (typ as u8, meta).encode();

        self.env.storage_set(key, value).context("set metadata failed")
    }

    fn get_metadata<T: Decode>(&self, name: Tag, typ: MetaDataType) -> Result<Option<T>> {
        let key = [STORAGE_KEY_METADATA.to_vec(), [typ as u8].to_vec(), name.0.to_vec()].concat();

        let value = self.env.storage_get(&key).context("get metadata failed")?;
        if let Some((typ_in_storage, res)) = value
            .map(|datas| <(u8, T)>::decode(&mut datas.as_slice()))
            .transpose()
            .context("decode failed")?
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
