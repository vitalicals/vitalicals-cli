use anyhow::{anyhow, Context as AnyhowContext, Result};

use vital_script_primitives::resources::Resource;

use crate::{sub::utils::hash_from_str, Context};

pub async fn move_vrc721s(context: &mut Context, hashs_str: &[String]) -> Result<()> {
    use vital_script_builder::templates;

    let hashs = hashs_str
        .iter()
        .map(|hash_str| hash_from_str(hash_str).context("hash from string"))
        .collect::<Result<Vec<_>>>()?;

    let mut utxos = Vec::with_capacity(hashs.len());
    for hash in hashs.iter() {
        let vrc721_resource = Resource::vrc721(*hash);

        // Got the vrc721 resource
        let input_vrc721_utxo = context
            .get_owned_resource(&vrc721_resource)
            .ok_or_else(|| anyhow!("move vrc721 need required a vrc721 resource by {}", hash))?;

        utxos.push(input_vrc721_utxo);
    }

    context.append_reveal_input(&utxos);

    let startup_output_index = 0;
    context
        .set_outputs_from(startup_output_index, hashs.len(), context.sats_amount)
        .context("set outputs")?;

    // build script.
    // all begin with 0.
    let scripts_bytes = templates::move_vrc721s(&hashs, Some(startup_output_index))
        .context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
