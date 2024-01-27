mod client;

pub use client::IndexerClient;

pub mod simulator;
pub mod traits;

pub(crate) const TARGET: &str = "interfaces::indexer";
