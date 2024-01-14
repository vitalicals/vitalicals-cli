use alloc::collections::BTreeMap;
use anyhow::{anyhow, Context, Result};
use vital_script_primitives::{resources::Resource, traits::context::EnvContext as EnvContextT};

use crate::traits::EnvFunctions;

pub struct EnvContext<Functions: EnvFunctions> {
    env: Functions,

    /// The outputs need to bind to outputs.
    cached_output_resources: BTreeMap<u8, Resource>,
}

impl<Functions: EnvFunctions> EnvContextT for EnvContext<Functions> {
    fn get_input_resource(&self, index: u8) -> Result<Resource> {
        let out_point = self.env.get_input(index);

        Ok(self
            .env
            .get_resources(&out_point)
            .context("get")?
            .ok_or_else(|| anyhow!("not found {} {}", index, out_point))?)
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

    fn apply_resources(&mut self) -> Result<()> {
        todo!()
    }
}
