//! The Env trait

use alloc::vec::Vec;
use anyhow::{Context, Result};

use bitcoin::OutPoint;

use vital_script_primitives::{consts::STORAGE_KEY_VRC721, resources::Resource, H256};

pub trait EnvFunctions: Clone {
    fn get_resources(&self, input_id: &OutPoint) -> Result<Option<Resource>>;
    fn bind_resource(&self, output: OutPoint, res: Resource) -> Result<()>;
    fn unbind_resource(&self, input: &OutPoint) -> Result<()>;

    fn storage_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn storage_set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;

    fn vrc721_had_mint(&self, hash: H256) -> Result<bool> {
        let key = [STORAGE_KEY_VRC721.to_vec(), hash.0.to_vec()].concat();

        // TODO: use a storage to store [u8; 32] -> bool map
        Ok(self.storage_get(&key).context("get metadata failed")?.is_some())
    }
}
