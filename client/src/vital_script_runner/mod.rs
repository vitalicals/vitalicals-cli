use bitcoin::Transaction;

use anyhow::{bail, Context as AnyhowContext, Result};
use vital_interfaces_indexer::{simulator::SimulatorEnvInterface, IndexerClient};
use vital_script_primitives::resources::Resource;
use vital_script_runner::{parse_vital_scripts, Context as RunnerContext, Runner};

use crate::context::Context;

pub struct LocalRunner<'a> {
    context: &'a Context,
}

impl<'a> LocalRunner<'a> {
    pub fn new(context: &'a Context) -> Self {
        Self { context }
    }

    pub fn new_runner_context(
        &self,
        block_height: u32,
        tx: &Transaction,
    ) -> RunnerContext<SimulatorEnvInterface<IndexerClient>> {
        let env_interface = SimulatorEnvInterface::new(self.context.indexer.clone());

        RunnerContext::simulator(env_interface, tx, block_height)
    }

    pub async fn run(&self, block_height: u32, tx: &Transaction) -> Result<Vec<(u8, Resource)>> {
        // need got commit tx
        let scripts = parse_vital_scripts(tx).context("parse_vital_scripts")?;
        if scripts.len() > 1 {
            todo!("Currently we not support more than one script");
        }

        let mut ctx = self.new_runner_context(block_height, tx);
        if !ctx.is_valid() {
            bail!("context not valid")
        }

        let mut runner = Runner::new();

        runner.run(&mut ctx).context("run")?;

        Ok(ctx.outputs)
    }
}
