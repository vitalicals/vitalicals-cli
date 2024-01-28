//! A wrapper for coin selector to ensure the input index sequence.

use std::collections::HashMap;

use bdk::{
    bitcoin::OutPoint,
    database::Database,
    wallet::coin_selection::{CoinSelectionAlgorithm, DefaultCoinSelectionAlgorithm},
    Utxo,
};

#[derive(Debug)]
pub struct CoinSelector {
    inner: DefaultCoinSelectionAlgorithm,
    reveal_inputs: Vec<OutPoint>,
}

impl CoinSelector {
    pub fn new(reveal_inputs: Vec<OutPoint>) -> Self {
        Self { inner: Default::default(), reveal_inputs }
    }
}

impl<D: Database> CoinSelectionAlgorithm<D> for CoinSelector {
    fn coin_select(
        &self,
        database: &D,
        required_utxos: Vec<bdk::WeightedUtxo>,
        optional_utxos: Vec<bdk::WeightedUtxo>,
        fee_rate: bdk::FeeRate,
        target_amount: u64,
        drain_script: &bdk::bitcoin::Script,
    ) -> Result<bdk::wallet::coin_selection::CoinSelectionResult, bdk::Error> {
        let mut res = self.inner.coin_select(
            database,
            required_utxos,
            optional_utxos,
            fee_rate,
            target_amount,
            drain_script,
        )?;

        let mut before = res
            .selected
            .iter()
            .map(|s| (s.outpoint(), s.clone()))
            .collect::<HashMap<OutPoint, Utxo>>();
        let mut selected = Vec::new();
        for r in self.reveal_inputs.iter() {
            selected.push(before.remove(r).expect("the reveal input should be in selected"))
        }

        for (_, r) in before.into_iter() {
            selected.push(r);
        }

        assert_eq!(res.selected.len(), selected.len());

        res.selected = selected;

        Ok(res)
    }
}
