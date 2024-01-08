use anyhow::Result;
use vital_script_primitives::{resources::Resource, traits::context::EnvContext as EnvContextT};

pub struct EnvContext {}

impl EnvContextT for EnvContext {
    fn get_input_resource(&self, _index: u8) -> Result<Resource> {
        todo!();
    }
}
