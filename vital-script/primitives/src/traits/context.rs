use anyhow::{bail, Context as AnyhowContext, Result};

use crate::resources::Resource;

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
}

pub trait RunnerContext {
    fn try_assert_input(&mut self, index: u8) -> Result<()>;
    fn try_assert_output(&mut self, index: u8) -> Result<()>;
    fn is_output_available(&self, index: u8) -> bool;
}

pub trait InputResourcesContext {
    fn push(&mut self, input_index: u8, resource: Resource) -> Result<()>;
    fn cost(&mut self, resource: Resource) -> Result<()>;

    fn all(&self) -> &[u8];
    fn uncosted(&self) -> Vec<(u8, Resource)>;
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
