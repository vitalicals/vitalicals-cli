use anyhow::{bail, Context as AnyhowContext, Result};

use vital_script_ops::{instruction::Instruction, parser::Parser};
pub use vital_script_primitives::traits::context::Context as ContextT;
use vital_script_primitives::traits::{
    context::EnvContext as EnvContextT, InputResourcesContext as InputResourcesContextT,
};

mod env;
mod input;
mod runner;

use env::EnvContext;
use input::InputResourcesContext;
use runner::RunnerContext;

use crate::traits::EnvFunctions;

const CAP_SIZE: usize = 16;

pub struct Context<Functions: EnvFunctions> {
    env: EnvContext<Functions>,
    input_resources: InputResourcesContext,
    runner: RunnerContext,
}

impl<Functions> ContextT for Context<Functions>
where
    Functions: EnvFunctions,
{
    type Env = EnvContext<Functions>;
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

impl<Functions> Context<Functions>
where
    Functions: EnvFunctions,
{
    pub fn new(env_interface: Functions) -> Self {
        let runner = RunnerContext::new();
        let input_resources = InputResourcesContext::new(CAP_SIZE);
        let env = EnvContext::new(env_interface);

        Self { env, input_resources, runner }
    }

    pub fn is_valid(&self) -> bool {
        self.env.is_valid()
    }

    pub fn get_instructions(&self) -> Result<Vec<Instruction>> {
        let ops_bytes = self.env.get_ops();
        let ins = ops_bytes
            .iter()
            .map(|(index, ops)| {
                Parser::new(ops).parse().with_context(|| format!("parse {}", index))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ins.concat())
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
        // del all inputs 's resources bind
        let all = self.input_resource().all().to_vec();
        self.env().remove_input_resources(&all).context("remove")?;

        // set all outputs 's resources bind
        self.env().apply_output_resources().context("apply")?;

        // storage all uncosted inputs 's resources to space.
        // TODO: impl

        Ok(())
    }
}
