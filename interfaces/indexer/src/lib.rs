mod client;

pub use client::IndexerClient;

pub mod simulator;
pub mod traits;

use vital_script::runner::EnvContext;

pub(crate) const TARGET: &str = "interfaces::indexer";

pub type QueryEnvContext = EnvContext<simulator::SimulatorEnvInterface<IndexerClient>>;

pub fn vital_env_for_query(indexer: IndexerClient) -> QueryEnvContext {
    let env_interface = simulator::SimulatorEnvInterface::new(indexer);

    EnvContext::new_for_query(env_interface)
}
