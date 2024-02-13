use std::sync::Mutex;

use alloc::{collections::BTreeMap, sync::Arc};
use anyhow::{Context as AnyhowContext, Result};

use bitcoin::{
    absolute::LockTime, hash_types::Txid, transaction::Version, Amount, OutPoint, ScriptBuf,
    Transaction, TxIn, TxOut,
};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions, instruction::Instruction, parser::Parser,
};
use vital_script_primitives::{
    resources::Resource,
    traits::{Context as ContextT, RunMode},
};

use crate::{traits::EnvFunctions, Context, Runner, TARGET};

pub fn init_logger() {
    let _ = env_logger::Builder::from_default_env()
        .format_module_path(true)
        .format_level(true)
        .filter_level(log::LevelFilter::Info)
        .parse_filters(format!("{}=debug", crate::TARGET).as_str())
        .parse_filters("vital::ops=debug")
        .try_init();
}

#[derive(Debug, Clone)]
pub struct TxMock {
    pub reveal: Transaction,
    pub reveal_txid: Txid,
    ops_bytes: Vec<(u8, Vec<u8>)>,
}

impl TxMock {
    pub fn new() -> Self {
        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: Vec::new(),
            output: Vec::new(),
        };
        let txid = tx.txid();

        Self { reveal: tx, reveal_txid: txid, ops_bytes: Vec::new() }
    }

    pub fn with_input(mut self, input: OutPoint) -> Self {
        self.push_input(input);
        self
    }

    pub fn push_input(&mut self, input: OutPoint) {
        let mut txin = TxIn::default();
        txin.previous_output = input;

        // println!("push_input input by index: {:?}", txin);

        self.reveal.input.push(txin);
        self.reveal_txid = self.reveal.txid();
    }

    pub fn with_ops(mut self, ops_bytes: Vec<u8>) -> Self {
        self.push_ops(ops_bytes);
        self
    }

    pub fn push_ops(&mut self, ops_bytes: Vec<u8>) {
        let txin = TxIn::default();

        let new_txin_index = self.reveal.input.len();
        assert!(new_txin_index < 0xff);

        self.reveal.input.push(txin);
        self.reveal_txid = self.reveal.txid();

        self.ops_bytes.push((new_txin_index as u8, ops_bytes));
    }

    pub fn with_output(mut self, amount: u64) -> Self {
        self.push_output(amount);
        self
    }

    pub fn push_output(&mut self, amount: u64) {
        let txout = TxOut { value: Amount::from_sat(amount), script_pubkey: ScriptBuf::default() };

        self.reveal.output.push(txout);
        self.reveal_txid = self.reveal.txid();
    }
}

#[derive(Debug, Clone)]
pub struct EnvMock {
    pub resource_storage: Arc<Mutex<BTreeMap<OutPoint, Resource>>>,
    pub storage: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl EnvMock {
    pub fn new() -> Self {
        Self {
            resource_storage: Arc::new(Mutex::new(BTreeMap::new())),
            storage: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }
}

impl EnvFunctions for EnvMock {
    fn get_resources(&self, input_id: &OutPoint) -> Result<Option<Resource>> {
        Ok(self.resource_storage.lock().expect("lock").get(input_id).cloned())
    }

    fn bind_resource(&self, output: OutPoint, res: Resource) -> Result<()> {
        assert!(!self.resource_storage.lock().expect("lock").contains_key(&output));

        log::debug!(target: TARGET, "bind_resource {} to {}", res, output);

        self.resource_storage.lock().expect("lock").insert(output, res);

        Ok(())
    }

    fn unbind_resource(&self, input: &OutPoint) -> Result<()> {
        assert!(self.resource_storage.lock().expect("lock").contains_key(input));

        log::debug!(target: TARGET, "unbind_resource {}", input);

        self.resource_storage.lock().expect("lock").remove(input);

        Ok(())
    }

    fn storage_get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        // println!("storage_get {:?} {:?}", self.storage, key);

        Ok(self.storage.lock().expect("lock").get(key).cloned())
    }

    fn storage_set(&self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        // println!("storage_set {:?}", self.storage);

        self.storage.lock().expect("lock").insert(key, value);

        Ok(())
    }
}

pub type ContextMockInner = Context<EnvMock>;

pub struct ContextMock {
    inner: ContextMockInner,
    tx: TxMock,
}

impl ContextMock {
    pub fn new(tx: TxMock, env: EnvMock) -> Self {
        log::info!("new context mock {}", tx.reveal_txid);

        Self { inner: ContextMockInner::new(env, &tx.reveal, 10000), tx }
    }
}

impl ContextT for ContextMock {
    type Env = <ContextMockInner as ContextT>::Env;
    type InputResource = <ContextMockInner as ContextT>::InputResource;
    type Runner = <ContextMockInner as ContextT>::Runner;

    type Instruction = Instruction;

    fn run_mod(&self) -> RunMode {
        RunMode::Normal
    }

    fn env(&self) -> &Self::Env {
        self.inner.env()
    }

    fn input_resource(&self) -> &Self::InputResource {
        self.inner.input_resource()
    }

    fn runner(&self) -> &Self::Runner {
        self.inner.runner()
    }

    fn env_mut(&mut self) -> &mut Self::Env {
        self.inner.env_mut()
    }

    fn input_resource_mut(&mut self) -> &mut Self::InputResource {
        self.inner.input_resource_mut()
    }

    fn runner_mut(&mut self) -> &mut Self::Runner {
        self.inner.runner_mut()
    }

    fn get_ops(&self) -> &[(u8, Vec<u8>)] {
        &self.tx.ops_bytes
    }

    fn get_instructions(&self) -> Result<Vec<Self::Instruction>> {
        let ops_bytes = self.get_ops();
        let ins = ops_bytes
            .iter()
            .map(|(index, ops)| {
                Parser::new(ops).parse().with_context(|| format!("parse {}", index))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ins.concat())
    }
    fn pre_check(&self) -> Result<()> {
        Ok(())
    }
    fn post_check(&self) -> Result<()> {
        Ok(())
    }
}

pub struct TestCtx {
    ops_bytes: Vec<Vec<u8>>,
    tx: TxMock,
}

impl TestCtx {
    pub fn new() -> Self {
        Self { ops_bytes: Vec::new(), tx: TxMock::new() }
    }

    pub fn with_instructions(mut self, ins: Vec<Instruction>) -> Self {
        let ops_bytes = ScriptBuilderFromInstructions::build(ins).expect("build should ok");

        self.ops_bytes.push(ops_bytes);
        self
    }

    pub fn with_input(mut self, input: OutPoint) -> Self {
        self.tx.push_input(input);
        self
    }

    pub fn with_ops(mut self) -> Self {
        self.tx.push_ops(self.ops_bytes.first().expect("no ops in ctx").clone());
        self
    }

    pub fn with_output(mut self, amount: u64) -> Self {
        self.tx.push_output(amount);
        self
    }

    pub fn run(&mut self, env_interface: &EnvMock) -> Result<ContextMock> {
        let mut context = ContextMock::new(self.tx.clone(), env_interface.clone());
        Runner::new().run(&mut context)?;

        Ok(context)
    }
}
