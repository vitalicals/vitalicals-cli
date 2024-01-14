use anyhow::{bail, Context as AnyhowContext, Result};

pub use vital_script_primitives::traits::context::Context as ContextT;
use vital_script_primitives::{
    resources::Resource,
    traits::{context::EnvContext as EnvContextT, RunnerContext as RunnerContextT},
};

mod env;
mod input;
mod runner;

pub use input::InputResourcesContext;
pub use runner::RunnerContext;

pub struct Context<EnvContext: EnvContextT> {
    env: EnvContext,
    input_resources: InputResourcesContext,
    runner: RunnerContext,
}

impl<EnvContext> ContextT for Context<EnvContext>
where
    EnvContext: EnvContextT,
{
    type Env = EnvContext;
    type InputResource = InputResourcesContext;
    type Runner = RunnerContext;

    fn env(&mut self) -> &mut Self::Env {
        &mut self.env
    }

    fn input_resource(&mut self) -> &mut Self::InputResource {
        &mut self.input_resources
    }

    fn runner(&mut self) -> &mut Self::Runner {
        &mut self.runner
    }
}

impl<EnvContext> Context<EnvContext>
where
    EnvContext: EnvContextT,
{
    pub fn new(
        env: EnvContext,
        input_resources: InputResourcesContext,
        runner: <Self as ContextT>::Runner,
    ) -> Self {
        Self { env, input_resources, runner }
    }

    pub fn pre_check(&self) -> Result<()> {
        // TODO: pre check
        Ok(())
    }

    /// Do post check
    pub fn post_check(&self) -> Result<()> {
        // TODO: post check
        Ok(())
    }

    /// Apply changes to indexer, will do:
    ///   - del all inputs 's resources bind
    ///   - set all outputs 's resources bind
    ///   - storage all uncosted inputs 's resources to space.
    pub fn apply_resources(&mut self) -> Result<()> {
        self.env().apply_resources()
    }
}
