use alloc::vec::Vec;
use anyhow::{Context as AnyhowContext, Result};

use bitcoin::{Block, Transaction, Txid};

use vital_script_runner::{check_is_vital_script, traits::EnvFunctions, Context, Runner};

use crate::TARGET;

pub enum TxRunStatus {
    Success,
    Failed,
}

pub struct TxRunResponse {
    pub status: TxRunStatus,
    pub tx_index: u32,
    pub tx_id: Txid,
    pub resp: (),
}

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

            if !check_is_vital_script(tx) {
                continue;
            }

            let context = Context::new(env_interface.clone(), tx);
            if !context.is_valid() {
                continue;
            }

            let resp = match self.run_tx(context, index, tx) {
                Ok(res) => TxRunResponse {
                    status: TxRunStatus::Success,
                    tx_index: index,
                    tx_id,
                    resp: res,
                },
                Err(_err) => {
                    TxRunResponse { status: TxRunStatus::Failed, tx_index: index, tx_id, resp: () }
                }
            };

            res.push(resp);
        }

        Ok(BlockRunResponse { txs: res })
    }

    fn run_tx<Functions>(
        &self,
        context: Context<Functions>,
        _index: u32,
        _tx: &Transaction,
    ) -> Result<()>
    where
        Functions: EnvFunctions,
    {
        let mut runner = Runner::new(context).context("new runner")?;

        runner.run().context("run")?;

        Ok(())
    }
}
