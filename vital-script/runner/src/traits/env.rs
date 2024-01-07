//! The Env trait

use anyhow::Result;

use vital_script_primitives::resources::Resource;

pub trait Env {
    type Id: PartialEq + Eq;

    fn get_resources(&self, input_id: &Self::Id) -> Result<Resource>;

    fn mint_resource(&self, output: &Self::Id, res: Resource) -> Result<()>;
    fn burn_resource(&self, input: &Self::Id, res: Resource) -> Result<()>;
    fn move_resource(&self, input: &Self::Id, output: &Self::Id, res: Resource) -> Result<()>;
}
