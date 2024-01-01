// ! The cli for shadowsats

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
	cli::run().await
}
