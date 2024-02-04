//! The Env trait

use alloc::vec::Vec;
use anyhow::Result;

use bitcoin::OutPoint;

use vital_script_primitives::resources::Resource;

pub trait EnvFunctions: Clone {
    fn get_resources(&self, input_id: &OutPoint) -> Result<Option<Resource>>;
    fn bind_resource(&self, output: OutPoint, res: Resource) -> Result<()>;
    fn unbind_resource(&self, input: &OutPoint) -> Result<()>;

    fn storage_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn storage_set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
}
