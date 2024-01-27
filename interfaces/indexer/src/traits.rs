use anyhow::Result;

use bitcoin::OutPoint;
use vital_script::primitives::resources::Resource;

/// A Trait for indexer
#[async_trait::async_trait]
pub trait IndexerClientT: Clone + Send + Sync {
    /// Get resource by outpoint
    async fn get_resource(&self, outpoint: &OutPoint) -> Result<Option<Resource>>;

    /// Get vital storage by key-value pair
    async fn get_storage(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
}
