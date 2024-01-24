use std::sync::Mutex;

use alloc::{collections::BTreeMap, sync::Arc};
use anyhow::{bail, Result};

use bitcoin::{
    absolute::LockTime, hash_types::Txid, transaction::Version, Amount, OutPoint, ScriptBuf,
    Transaction, TxIn, TxOut,
};
use vital_script_primitives::resources::Resource;

use crate::traits::EnvFunctions;

#[derive(Debug)]
pub struct TxMock {
    tx: Transaction,
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
    pub current_tx: Arc<TxMock>,
    pub resource_storage: Arc<Mutex<BTreeMap<OutPoint, Resource>>>,
    pub storage: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
}

impl EnvMock {
    pub fn new(tx: TxMock) -> Self {
        Self {
            current_tx: Arc::new(tx),
            resource_storage: Arc::new(Mutex::new(BTreeMap::new())),
            storage: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub fn next_psbt(&mut self, tx: TxMock) {
        self.current_tx = Arc::new(tx);
    }
}

impl EnvFunctions for EnvMock {
    /// get current tx id.
    fn get_tx_id(&self) -> &Txid {
        &self.current_tx.txid
    }

    fn is_valid(&self) -> bool {
        true
    }

    fn get_ops(&self) -> &[(u8, Vec<u8>)] {
        &self.current_tx.ops_bytes
    }

    /// Get the input 's point by index.
    fn get_input(&self, input_index: u8) -> Result<OutPoint> {
        // println!("get input by index: {}", input_index);

        let input_len = self.current_tx.tx.input.len();
        if input_index as usize >= input_len {
            bail!("the index not exists in the input expect {}, got {}", input_index, input_len);
        }

        let input = &self.current_tx.tx.input[input_index as usize];

        // println!("get input by index: {:?}", input);

        Ok(input.previous_output)
    }

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
