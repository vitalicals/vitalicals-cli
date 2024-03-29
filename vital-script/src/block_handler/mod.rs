use alloc::vec::Vec;
use anyhow::{Context as AnyhowContext, Result};

use bitcoin::{Block, Transaction, Txid};

use vital_script_primitives::traits::Context as ContextT;
use vital_script_runner::{
    check_is_vital_script, maybe_vital_commit_tx_with_input_resource, traits::EnvFunctions,
    Context, Runner,
};

use crate::TARGET;

#[derive(Debug, Clone)]
pub enum TxRunStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TxRunResponse {
    pub status: TxRunStatus,
    pub tx_index: u32,
    pub tx_id: Txid,
    pub resp: (),
}

#[derive(Debug, Clone)]
pub struct BlockRunResponse {
    pub txs: Vec<TxRunResponse>,
}

pub struct BlockRunner<'a> {
    block: &'a Block,
    height: u32,
}

impl<'a> BlockRunner<'a> {
    pub fn new(block: &'a Block, height: u32) -> Self {
        Self { block, height }
    }

    pub fn run<Functions>(&self, env_interface: Functions) -> Result<BlockRunResponse>
    where
        Functions: EnvFunctions,
    {
        let mut res = Vec::with_capacity(self.block.txdata.len());

        for (index, tx) in self.block.txdata.iter().enumerate() {
            let index = index as u32;
            let tx_id = tx.txid();
            log::debug!(target: TARGET, "run tx index {}, {} on {}", index, tx_id, self.height);

            if maybe_vital_commit_tx_with_input_resource(tx, &env_interface)
                .context("maybe_vital_commit_tx_with_input_resource")?
            {
                log::info!(target: TARGET, "handle vital commit transaction {}", tx_id);
            }

            if !check_is_vital_script(tx) {
                continue;
            }

            if tx.input.is_empty() {
                log::debug!(target: TARGET, "skip by input is zero");
                continue;
            }

            let commit_txid = tx.input[0].previous_output.txid;
            log::debug!(target: TARGET, "process vital tx with commit txid {}", commit_txid);

            let context = Context::new(env_interface.clone(), tx, self.height);
            if let Err(err) = context.pre_check() {
                log::debug!(target: TARGET, "context is not valid by {}", err);
                continue;
            }

            let resp = match self.run_tx(context, index, tx) {
                Ok(res) => TxRunResponse {
                    status: TxRunStatus::Success,
                    tx_index: index,
                    tx_id,
                    resp: res,
                },
                Err(err) => {
                    log::warn!(target: TARGET, "tx run failed by {}", err);

                    TxRunResponse { status: TxRunStatus::Failed, tx_index: index, tx_id, resp: () }
                }
            };

            res.push(resp);
        }

        Ok(BlockRunResponse { txs: res })
    }

    fn run_tx<Functions>(
        &self,
        mut context: Context<Functions>,
        _index: u32,
        _tx: &Transaction,
    ) -> Result<()>
    where
        Functions: EnvFunctions,
    {
        let mut runner = Runner::new();

        runner.run(&mut context).context("run")?;

        Ok(())
    }
}
