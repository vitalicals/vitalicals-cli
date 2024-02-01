use std::fs;

use anyhow::Result;
use bdk::bitcoin::OutPoint;

pub fn load_used_utxos(path: &std::path::Path) -> Result<Vec<OutPoint>> {
    let path = path.join("used_utxos.json");
    if !path.exists() {
        return Ok(Vec::new());
    }

    let utxos = serde_json::from_str(fs::read_to_string(path)?.as_str())?;

    Ok(utxos)
}

pub fn save_used_utxos(path: &std::path::Path, utxos: &[OutPoint]) -> Result<()> {
    let path = path.join("used_utxos.json");

    fs::write(path, serde_json::to_string_pretty(utxos)?)?;

    Ok(())
}

pub fn append_used_utxos(path: &std::path::Path, utxos: &[OutPoint]) -> Result<()> {
    let mut current_utxos = load_used_utxos(path)?;
    current_utxos.append(&mut utxos.to_vec());

    save_used_utxos(path, &current_utxos)
}
