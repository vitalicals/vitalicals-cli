use anyhow::{bail, Context as AnyhowContext, Result};
use parity_scale_codec::{Decode, Encode};

use crate::{
    resources::{Resource, ResourceType, Tag},
    types::vrc20::{VRC20MetaData, VRC20StatusData},
};

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum MetaDataType {
    Name = 1_u8,
    VRC20,
    VRC721,
}

pub trait EnvContext {
    fn get_ops(&self) -> &[(u8, Vec<u8>)];

    fn get_input_resource(&self, index: u8) -> Result<Resource>;
    fn get_output_resource(&self, index: u8) -> Option<&Resource>;

    fn set_resource_to_output(&mut self, index: u8, resource: Resource) -> Result<()>;

    /// Del all inputs 's resources bind
    fn remove_input_resources(&self, input_indexs: &[u8]) -> Result<()>;

    /// Apply changes to indexer, will do:
    ///   - del all inputs 's resources bind
    ///   - set all outputs 's resources bind
    ///   - storage all uncosted inputs 's resources to space.
    fn apply_output_resources(&mut self) -> Result<()>;

    fn new_name(&mut self, name: Tag) -> Result<()> {
        let curr = self.get_metadata::<bool>(name, MetaDataType::Name).context("get")?;
        if curr.is_some() {
            bail!("the name had created");
        }

        // set to false, means it not costed
        self.set_metadata(name, MetaDataType::Name, false).context("set")
    }

    fn cost_name(&mut self, name: Tag) -> Result<()> {
        match self.get_metadata::<bool>(name, MetaDataType::Name).context("get")? {
            Some(false) => {
                self.set_metadata(name, MetaDataType::Name, true).context("set")?;
            }
            Some(true) => {
                bail!("the name had costed");
            }
            None => {
                bail!("the name had not created");
            }
        }

        Ok(())
    }

    fn deploy_vrc20(&mut self, name: Tag, meta: VRC20MetaData) -> Result<()> {
        let curr = self.get_vrc20_metadata(name).context("get")?;
        if curr.is_some() {
            bail!("the vrc20 had created");
        }

        // set to false, means it not costed
        self.set_vrc20_metadata(name, VRC20StatusData { mint_count: 0, meta })
            .context("set")
    }

    fn increase_vrc20_mint_count(&mut self, name: Tag) -> Result<()> {
        let status_data = self.get_vrc20_metadata(name).context("get")?;
        if let Some(mut status_data) = status_data {
            if status_data.mint_count >= status_data.meta.mint.max_mints {
                bail!("mint count had reached max");
            }

            status_data.mint_count += 1;
            self.set_vrc20_metadata(name, status_data).context("set")?;
        } else {
            bail!("the vrc20 had created");
        }

        Ok(())
    }

    fn get_vrc20_metadata(&self, name: Tag) -> Result<Option<VRC20StatusData>> {
        self.get_metadata(name, MetaDataType::VRC20)
    }

    fn set_vrc20_metadata(&mut self, name: Tag, meta: VRC20StatusData) -> Result<()> {
        self.set_metadata(name, MetaDataType::VRC20, meta)
    }

    fn set_metadata<T: Encode>(&mut self, name: Tag, typ: MetaDataType, meta: T) -> Result<()>;
    fn get_metadata<T: Decode>(&self, name: Tag, typ: MetaDataType) -> Result<Option<T>>;
}

pub trait RunnerContext {
    fn try_assert_input(&mut self, index: u8) -> Result<()>;
    fn try_assert_output(&mut self, index: u8) -> Result<()>;
    fn is_output_available(&self, index: u8) -> bool;
}

pub trait InputResourcesContext {
    fn push(&mut self, input_index: u8, resource: Resource) -> Result<()>;
    fn cost(&mut self, resource: &Resource) -> Result<()>;

    fn all(&self) -> &[u8];
    fn uncosted(&self) -> Vec<(u8, Resource)>;

    fn get_vrc20(&self, name: Tag) -> Option<Resource>;
}

pub trait Context {
    type Env: EnvContext;
    type Runner: RunnerContext;
    type InputResource: InputResourcesContext;

    fn env(&mut self) -> &mut Self::Env;
    fn runner(&mut self) -> &mut Self::Runner;
    fn input_resource(&mut self) -> &mut Self::InputResource;

    fn send_resource_to_output(&mut self, index: u8, resource: Resource) -> Result<()> {
        // 1. only the output asserted can be send resource into.
        if !self.runner().is_output_available(index) {
            bail!("the output is not asserted");
        }

        // 2. if a output had been sent a resource, need check if item can merged.
        //    if the resource cannot be merged, it will return an error.
        let output_resource = self.env().get_output_resource(index);
        let resource = match output_resource {
            Some(res) => resource.merge_into(res),
            None => Ok(resource),
        }
        .context("the output cannot merge to")?;

        // 3. set the resource to output
        self.env().set_resource_to_output(index, resource).context("set")?;

        Ok(())
    }
}
