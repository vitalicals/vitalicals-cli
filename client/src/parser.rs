use bitcoin::Transaction;

pub fn tx_from_bdk(tx: bdk::bitcoin::Transaction) -> Transaction {
    serde_json::from_value(serde_json::to_value(tx).expect("in should ok")).expect("out should ok")
}
