//! A client for json rpc

use std::sync::Arc;

use anyhow::{Context, Result};
use jsonrpsee::{
    core::client::ClientT,
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};

use bitcoin::OutPoint;
use vital_script::primitives::resources::Resource;

use crate::traits::IndexerClientT;

#[derive(Clone)]
pub struct IndexerClient {
    client: Arc<HttpClient>,
}

impl IndexerClient {
    pub async fn new(target: &str) -> Result<Self> {
        let client = HttpClientBuilder::default().build(target.to_string())?;
        let client = Arc::new(client);

        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl IndexerClientT for IndexerClient {
    async fn get_resource(&self, outpoint: &OutPoint) -> Result<Option<Resource>> {
        let res = self
            .client
            .request("vital.resource", rpc_params![outpoint.txid, outpoint.vout])
            .await
            .with_context(|| format!("request by {}", outpoint))?;

        Ok(res)
    }

    async fn get_storage(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let res = self
            .client
            .request("vital.storage", rpc_params![key.to_vec()])
            .await
            .with_context(|| format!("request by {}", hex::encode(key)))?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use vital_script::primitives::resources::Name;

    use crate::traits::IndexerClientT;

    use super::*;

    #[tokio::test]
    async fn test_get_resource() {
        let cli = IndexerClient::new("http://localhost:9944").await.expect("new");

        let res = cli
            .get_resource(
                &OutPoint::from_str(
                    "e75104215c041dbbe575c2b15b04f244f2ca0f277d2e9d035039b2838133a91e:0",
                )
                .unwrap(),
            )
            .await
            .expect("get_resource");

        println!("res {}", res.clone().unwrap_or_default());

        assert_eq!(res, Some(Resource::name(Name::try_from("myself").unwrap())))
    }
}
