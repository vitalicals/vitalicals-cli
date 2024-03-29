use alloc::vec::Vec;
use anyhow::{Context as AnyhowContext, Result};

use bitcoin::Transaction;
use vital_script_ops::{instruction::Instruction, parser::Parser};
pub use vital_script_primitives::traits::context::Context as ContextT;
use vital_script_primitives::{
    resources::Resource,
    traits::{
        context::EnvContext as EnvContextT, InputResourcesContext as InputResourcesContextT,
        RunMode,
    },
};

mod env;
mod input;
mod runner;
pub mod script;

pub use env::EnvContext;
use input::InputResourcesContext;
use runner::RunnerContext;

use crate::{traits::EnvFunctions, TARGET};

const CAP_SIZE: usize = 16;

#[derive(Clone)]
pub struct Context<Functions: EnvFunctions> {
    env: EnvContext<Functions>,
    input_resources: InputResourcesContext,
    runner: RunnerContext,
    pub outputs: Vec<(u8, Resource)>,
    mode: RunMode,
}

impl<Functions> ContextT for Context<Functions>
where
    Functions: EnvFunctions,
{
    type Env = EnvContext<Functions>;
    type InputResource = InputResourcesContext;
    type Runner = RunnerContext;

    type Instruction = Instruction;

    fn run_mod(&self) -> RunMode {
        self.mode
    }

    fn env(&self) -> &Self::Env {
        &self.env
    }

    fn env_mut(&mut self) -> &mut Self::Env {
        &mut self.env
    }

    fn input_resource(&self) -> &Self::InputResource {
        &self.input_resources
    }

    fn input_resource_mut(&mut self) -> &mut Self::InputResource {
        &mut self.input_resources
    }

    fn runner(&self) -> &Self::Runner {
        &self.runner
    }

    fn runner_mut(&mut self) -> &mut Self::Runner {
        &mut self.runner
    }

    fn get_ops(&self) -> &[(u8, Vec<u8>)] {
        self.env.get_ops()
    }

    fn get_instructions(&self) -> Result<Vec<Instruction>> {
        let ops_bytes = self.get_ops();
        let ins = ops_bytes
            .iter()
            .map(|(index, ops)| {
                Parser::new(ops).parse().with_context(|| alloc::format!("parse {}", index))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ins.concat())
    }

    fn pre_check(&self) -> Result<()> {
        use vital_script_primitives::traits::Instruction;

        let instructions = self.get_instructions().context("get instructions")?;

        for (index, instruction) in instructions.iter().enumerate() {
            instruction
                .pre_check()
                .with_context(|| alloc::format!("instruction {}", index))?;
        }

        Ok(())
    }

    /// Do post check
    fn post_check(&self) -> Result<()> {
        let uncosted = self.input_resource().uncosted();

        if !uncosted.is_empty() {
            log::warn!(target: TARGET, "the input not all costed yet");
        }

        Ok(())
    }

    fn on_output(&mut self, index: u8, resource: Resource) {
        if self.run_mod() == RunMode::Simulator {
            self.outputs.push((index, resource));
        }
    }
}

impl<Functions> Context<Functions>
where
    Functions: EnvFunctions,
{
    pub fn new(env_interface: Functions, reveal_tx: &Transaction, block_height: u32) -> Self {
        let runner = RunnerContext::new();
        let input_resources = InputResourcesContext::new(CAP_SIZE);
        let env = EnvContext::new(
            env_interface,
            reveal_tx.input.iter().map(|i| i.previous_output).collect::<Vec<_>>(),
            reveal_tx,
            block_height,
        );

        Self { env, input_resources, runner, mode: RunMode::Normal, outputs: Vec::new() }
    }

    pub fn simulator(env_interface: Functions, reveal_tx: &Transaction, block_height: u32) -> Self {
        let runner = RunnerContext::new();
        let input_resources = InputResourcesContext::new(CAP_SIZE);
        let env = EnvContext::new_for_sim(env_interface, reveal_tx, block_height);

        Self { env, input_resources, runner, mode: RunMode::Simulator, outputs: Vec::new() }
    }
}
