//! A simulator by indexer

use anyhow::{bail, Context, Result};
use futures::executor::block_on;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use bitcoin::OutPoint;
use vital_script::{primitives::resources::Resource, runner::traits::EnvFunctions};

use crate::{traits::IndexerClientT, TARGET};

#[derive(Debug, Default, Clone)]
pub struct SimulatorStatus {
    resource_storage: HashMap<OutPoint, Resource>,
    removed_resources: HashSet<OutPoint>,
    storage: HashMap<Vec<u8>, Vec<u8>>,
    removed_storage: HashSet<Vec<u8>>,
}

impl SimulatorStatus {
    fn get_resources(&self, outpoint: &OutPoint) -> Option<Resource> {
        if self.removed_resources.contains(outpoint) {
            return None;
        }

        self.resource_storage.get(outpoint).cloned()
    }

    fn bind_resource(&mut self, outpoint: OutPoint, resource: Resource) -> Result<()> {
        if self.removed_resources.contains(&outpoint) {
            bail!("the resource by {} to set is already removed, one outpoint just can use only once!", outpoint);
        }

        if self.resource_storage.contains_key(&outpoint) {
            bail!("the resource by {} had already set", outpoint);
        }

        self.resource_storage.insert(outpoint, resource);

        Ok(())
    }

    fn unbind_resource(&mut self, outpoint: &OutPoint) -> Result<()> {
        if self.removed_resources.contains(outpoint) {
            bail!("the resource by {} had removed", outpoint);
        }

        self.removed_resources.insert(*outpoint);

        Ok(())
    }

    fn is_resource_removed(&self, outpoint: &OutPoint) -> bool {
        self.removed_resources.contains(outpoint)
    }

    fn storage_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        if self.removed_storage.contains(key) {
            return None;
        }

        self.storage.get(key).cloned()
    }

    fn storage_set(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        if self.removed_storage.contains(&key) {
            self.removed_storage.remove(&key);
        }

        self.storage.insert(key, value);

        Ok(())
    }

    fn is_storage_removed(&self, key: &[u8]) -> bool {
        self.removed_storage.contains(key)
    }
}

#[derive(Debug, Clone)]
pub struct SimulatorEnvInterface<Client>
where
    Client: IndexerClientT,
{
    client: Client,
    cache: Arc<Mutex<SimulatorStatus>>,
}

impl<Client> SimulatorEnvInterface<Client>
where
    Client: IndexerClientT,
{
    pub fn new(client: Client) -> Self {
        Self { client, cache: Arc::new(Mutex::new(SimulatorStatus::default())) }
    }
}

impl<Client> EnvFunctions for SimulatorEnvInterface<Client>
where
    Client: IndexerClientT,
{
    fn get_resources(&self, input_id: &OutPoint) -> Result<Option<Resource>> {
        let cache = self.cache.lock().expect("lock");

        if cache.is_resource_removed(input_id) {
            return Ok(None);
        }

        if let Some(res) = cache.get_resources(input_id) {
            return Ok(Some(res));
        }

        let res = block_on(async { self.client.get_resource(input_id).await })
            .context("get resource by client")?;

        Ok(res)
    }

    fn bind_resource(&self, output: OutPoint, res: Resource) -> Result<()> {
        let res_in_remote = block_on(async { self.client.get_resource(&output).await })
            .context("get resource for check")?;

        if let Some(res_in_remote) = res_in_remote {
            bail!("the resource had been bind {} for {}", res_in_remote, output);
        }

        self.cache
            .lock()
            .expect("lock")
            .bind_resource(output, res)
            .context("bind resource")?;

        Ok(())
    }

    fn unbind_resource(&self, input: &OutPoint) -> Result<()> {
        let res = self.get_resources(input).context("get resources for check")?;

        if let Some(res) = res {
            log::debug!(target: TARGET, "unbind resource {} in {}", input, res);

            self.cache.lock().expect("lock").unbind_resource(input).context("cache")?;
        } else {
            bail!("not resource exists in cache");
        }

        Ok(())
    }

    fn storage_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cache = self.cache.lock().expect("lock");

        if cache.is_storage_removed(key) {
            return Ok(None);
        }

        if let Some(res) = cache.storage_get(key) {
            return Ok(Some(res));
        }

        let res = block_on(async { self.client.get_storage(key).await })
            .context("get resource by client")?;

        Ok(res)
    }

    fn storage_set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        let mut cache = self.cache.lock().expect("lock");

        cache.storage_set(key, value)?;

        Ok(())
    }
}
