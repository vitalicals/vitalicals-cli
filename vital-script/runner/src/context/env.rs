use alloc::collections::BTreeMap;
use anyhow::{anyhow, Context, Result};
use vital_script_primitives::{resources::Resource, traits::context::EnvContext as EnvContextT};

use crate::traits::EnvFunctions;

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
}
