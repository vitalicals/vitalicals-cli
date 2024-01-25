use std::sync::Mutex;

use alloc::{collections::BTreeMap, sync::Arc};
use anyhow::{bail, Context as AnyhowContext, Result};

use bitcoin::{
    absolute::LockTime, hash_types::Txid, transaction::Version, Amount, OutPoint, ScriptBuf,
    Transaction, TxIn, TxOut,
};
use vital_script_ops::{instruction::Instruction, parser::Parser};
use vital_script_primitives::{
    resources::Resource,
    traits::{Context as ContextT, EnvContext as EnvContextT},
};

use crate::{traits::EnvFunctions, Context};

#[derive(Debug, Clone)]
pub struct TxMock {
    pub tx: Transaction,
    pub txid: Txid,
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

        Self { tx, txid, ops_bytes: Vec::new() }
    }

    pub fn push_input(&mut self, input: OutPoint) {
        let mut txin = TxIn::default();
        txin.previous_output = input;

        // println!("push_input input by index: {:?}", txin);

        self.tx.input.push(txin);
        self.txid = self.tx.txid();
    }

    pub fn push_ops(&mut self, ops_bytes: Vec<u8>) {
        let txin = TxIn::default();

        let new_txin_index = self.tx.input.len();
        assert!(new_txin_index < 0xff);

        self.tx.input.push(txin);
        self.txid = self.tx.txid();

        self.ops_bytes.push((new_txin_index as u8, ops_bytes));
    }

    pub fn push_output(&mut self, amount: u64) {
        let txout = TxOut { value: Amount::from_sat(amount), script_pubkey: ScriptBuf::default() };

        self.tx.output.push(txout);
        self.txid = self.tx.txid();
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

        self.resource_storage.lock().expect("lock").insert(output, res);

        Ok(())
    }

    fn unbind_resource(&self, input: &OutPoint) -> Result<()> {
        assert!(self.resource_storage.lock().expect("lock").contains_key(input));

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
        Self { inner: ContextMockInner::new(env, &tx.tx), tx }
    }
}

impl ContextT for ContextMock {
    type Env = <ContextMockInner as ContextT>::Env;
    type InputResource = <ContextMockInner as ContextT>::InputResource;
    type Runner = <ContextMockInner as ContextT>::Runner;

    type Instruction = Instruction;

    fn env(&mut self) -> &mut Self::Env {
        self.inner.env()
    }

    fn input_resource(&mut self) -> &mut Self::InputResource {
        self.inner.input_resource()
    }

    fn runner(&mut self) -> &mut Self::Runner {
        self.inner.runner()
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
