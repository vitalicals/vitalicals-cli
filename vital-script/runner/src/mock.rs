use std::sync::Mutex;

use alloc::{collections::BTreeMap, sync::Arc};
use anyhow::{Context as AnyhowContext, Result};

use bitcoin::{
    absolute::LockTime, hash_types::Txid, transaction::Version, Amount, OutPoint, ScriptBuf,
    Sequence, Transaction, TxIn, TxOut,
};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert,
        resource_deploy::InstructionVRC20Deploy, Instruction,
    },
    parser::Parser,
};
use vital_script_primitives::{
    resources::{Name, Resource, ResourceType},
    traits::{Context as ContextT, EnvContext, RunMode},
    types::vrc20::{VRC20MetaData, VRC20MintMeta},
};

use crate::{traits::EnvFunctions, Context, Runner, TARGET};

pub fn assert_err_str<T>(res: Result<T>, str: &str, reason: &str) {
    let res = res
        .err()
        .unwrap_or_else(|| panic!("the res should be error by {}", reason))
        .root_cause()
        .to_string();
    if res != str {
        panic!("the err not expected for {}:\n expected: {}\n      got: {}", reason, str, res)
    }
}

#[derive(Debug, Clone)]
pub struct TxMock {
    pub reveal: Transaction,
    pub reveal_txid: Txid,
    pub seq: u32,
    ops_bytes: Vec<(u8, Vec<u8>)>,
}

impl Default for TxMock {
    fn default() -> Self {
        Self::new()
    }
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

        Self { reveal: tx, reveal_txid: txid, ops_bytes: Vec::new(), seq: 0 }
    }

    /// Add a ext count, just for make txid not eq.
    pub fn with_ext(mut self, c: u32) -> Self {
        self.seq = c;
        self
    }

    pub fn with_input(mut self, input: OutPoint) -> Self {
        self.push_input(input);
        self
    }

    pub fn push_input(&mut self, input: OutPoint) {
        let txin =
            TxIn { previous_output: input, sequence: Sequence(self.seq), ..Default::default() };

        // println!("push_input input by index: {:?}", txin);

        self.reveal.input.push(txin);
        self.reveal_txid = self.reveal.txid();
    }

    pub fn with_ops(mut self, ops_bytes: Vec<u8>) -> Self {
        self.push_ops(ops_bytes);
        self
    }

    pub fn push_ops(&mut self, ops_bytes: Vec<u8>) {
        let txin = TxIn { sequence: Sequence(self.seq), ..Default::default() };

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

impl Default for EnvMock {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvMock {
    pub fn new() -> Self {
        Self {
            resource_storage: Arc::new(Mutex::new(BTreeMap::new())),
            storage: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn get_outpoint(&self, resource: &Resource) -> Option<OutPoint> {
        let storage = self.resource_storage.lock().expect("lock");

        for (outpoint, res) in storage.iter() {
            if resource == res {
                return Some(*outpoint);
            }
        }

        None
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
    pub ops_bytes: Vec<Vec<u8>>,
    tx: TxMock,
    env_interface: EnvMock,

    /// A count auto increment for make txid not eq.
    count: u32,
}

impl TestCtx {
    pub fn new(env_interface: &EnvMock) -> Self {
        Self {
            ops_bytes: Vec::new(),
            tx: TxMock::new(),
            env_interface: env_interface.clone(),
            count: 1,
        }
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
        let bytes = self.ops_bytes.first().expect("no ops in ctx").clone();
        log::debug!(target: TARGET, "with ops bytes: {}", hex::encode(&bytes));
        self.tx.push_ops(bytes);
        self
    }

    pub fn with_ops_bytes(mut self, ops_bytes: &[u8]) -> Self {
        let bytes = ops_bytes.to_vec();
        log::debug!(target: TARGET, "with ops bytes: {}", hex::encode(&bytes));
        self.ops_bytes = vec![bytes.clone()];
        self.tx.push_ops(bytes);
        self
    }

    pub fn with_output(mut self, amount: u64) -> Self {
        self.tx.push_output(amount);
        self
    }

    pub fn run(&mut self) -> Result<ContextMock> {
        let mut context = ContextMock::new(self.tx.clone(), self.env_interface.clone());
        Runner::new().run(&mut context)?;

        Ok(context)
    }

    pub fn get_name_outpoint(&self, name: impl Into<String>) -> Option<OutPoint> {
        self.env_interface
            .get_outpoint(&Resource::Name(Name::try_from(name.into()).expect("name failed")))
    }

    pub fn mint_name(&mut self, name: impl Into<String>) {
        let mint_name = Name::try_from(name.into()).expect("the name format not supported");

        let ops_bytes = ScriptBuilderFromInstructions::build(vec![
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::mint(0, ResourceType::name(mint_name)),
        ])
        .expect("build ops_bytes should ok");

        let mut tx_mock = TxMock::new().with_ext(self.count);
        tx_mock.push_ops(ops_bytes);
        tx_mock.push_output(1000);

        self.count += 1;

        let mut context = ContextMock::new(tx_mock, self.env_interface.clone());
        Runner::new().run(&mut context).expect("run failed");
    }

    pub fn deploy_vrc20(&mut self, name: impl Into<String>, mint_amount: u128) {
        self.deploy_vrc20_with_max(name, mint_amount, 500)
    }

    pub fn deploy_vrc20_with_max(
        &mut self,
        name: impl Into<String>,
        mint_amount: u128,
        max_count: u64,
    ) {
        let name = name.into();

        // if not mint name, just mint it.
        if self.get_name_outpoint(name.clone()).is_none() {
            self.mint_name(name.clone());
        }

        let mint_name = Name::try_from(name.clone()).unwrap();

        // 2. deploy a vrc20 by the name
        let ops_bytes = ScriptBuilderFromInstructions::build(vec![
            Instruction::Input(InstructionInputAssert {
                index: 0, // Note this tx will push input first into the tx
                resource: Resource::Name(mint_name),
            }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::Deploy(InstructionVRC20Deploy {
                name_input: 0,
                name: mint_name,
                meta: VRC20MetaData {
                    decimals: 5,
                    nonce: 1000000,
                    bworkc: 1000000,
                    mint: VRC20MintMeta { mint_amount, mint_height: 0, max_mints: max_count },
                    meta: None,
                },
            }),
        ])
        .expect("build should ok");

        log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

        let mut tx_mock = TxMock::new().with_ext(self.count);
        tx_mock.push_input(self.get_name_outpoint(name).expect("not found name outpoint"));
        tx_mock.push_ops(ops_bytes);
        tx_mock.push_output(2000);

        self.count += 1;

        let mut context = ContextMock::new(tx_mock, self.env_interface.clone());
        Runner::new().run(&mut context).expect("run failed");
    }

    pub fn mint_vrc20(&mut self, name: impl Into<String>) -> OutPoint {
        let mint_name = Name::try_from(name.into()).unwrap();

        let ops_bytes = ScriptBuilderFromInstructions::build(vec![
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::mint(0, ResourceType::vrc20(mint_name)),
        ])
        .expect("build should ok");

        log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

        let mut tx_mock3 = TxMock::new().with_ext(self.count);
        tx_mock3.push_output(2000);
        tx_mock3.push_ops(ops_bytes);

        self.count += 1;

        let mut context = ContextMock::new(tx_mock3, self.env_interface.clone());
        Runner::new().run(&mut context).expect("run failed");

        let outpoint = context.env().get_output(0);
        let res = self
            .env_interface
            .get_resources(&outpoint)
            .expect("get resources failed")
            .expect("no found resource");

        assert_eq!(res.resource_type(), ResourceType::vrc20(mint_name));

        outpoint
    }
}
