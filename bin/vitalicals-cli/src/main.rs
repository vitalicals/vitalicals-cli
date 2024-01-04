// ! The cli for vitalicals

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}
