//! The Env trait

use anyhow::Result;

use bdk::bitcoin::{hash_types::Txid, OutPoint};

use vital_script_primitives::resources::Resource;

pub trait EnvFunctions: Send + Sync + 'static {
    /// get current tx id.
    fn get_tx_id(&self) -> &Txid;

    /// Get the input 's point by index.
    fn get_input(&self, input_index: u8) -> OutPoint;

    /// Get the output 's point by the index for current tx.
    fn get_output(&self, output_index: u8) -> OutPoint {
        OutPoint { txid: *self.get_tx_id(), vout: output_index as u32 }
    }

    fn get_resources(&self, input_id: &OutPoint) -> Result<Option<Resource>>;
    fn bind_resource(&mut self, output: &OutPoint, res: Resource) -> Result<()>;
    fn unbind_resource(&mut self, input: &OutPoint, res: Resource) -> Result<()>;
}
