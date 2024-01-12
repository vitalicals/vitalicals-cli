use anyhow::Result;

use crate::resources::Resource;

pub trait EnvContext {
    fn get_input_resource(&self, index: u8) -> Result<Resource>;
}

pub trait RunnerContext {
    fn try_assert_input(&mut self, index: u8) -> Result<()>;
    fn try_assert_output(&mut self, index: u8) -> Result<()>;
    fn is_output_available(&self, index: u8) -> bool;
}

pub trait InputResourcesContext {
    fn push(&mut self, input_index: u8, resource: Resource) -> Result<()>;
    fn cost(&mut self, resource: Resource) -> Result<()>;
}

pub trait Context {
    type Env: EnvContext;
    type Runner: RunnerContext;
    type InputResource: InputResourcesContext;

    fn env(&mut self) -> &mut Self::Env;
    fn runner(&mut self) -> &mut Self::Runner;
    fn input_resource(&mut self) -> &mut Self::InputResource;
}
