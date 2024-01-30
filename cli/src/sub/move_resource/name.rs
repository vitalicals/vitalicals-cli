use anyhow::{anyhow, Context as AnyhowContext, Result};

use vital_script_primitives::resources::{Name, Resource};

use crate::Context;

pub async fn move_names(context: &mut Context, names: &[String], amount: u64) -> Result<()> {
    use vital_script_builder::templates;

    let mut utxos = Vec::with_capacity(names.len());
    let mut move_names = Vec::with_capacity(names.len());

    for name in names.iter() {
        let name = Name::try_from(name.as_str())
            .with_context(|| format!("the '{}' name format is invalid", name))?;

        let name_resource = Resource::name(name.clone());

        // Got the name resource
        let input_name_utxo = context
            .get_owned_resource(&name_resource)
            .ok_or_else(|| anyhow!("move name need required a name resource by {}", name))?;

        move_names.push(name);
        utxos.push(input_name_utxo);
    }

    context.append_reveal_input(&utxos);

    let startup_output_index = 0;
    context
        .set_outputs_from(startup_output_index, names.len(), amount)
        .context("set outputs")?;

    // build script.
    // all begin with 0.
    let scripts_bytes = templates::move_names(&move_names, Some(startup_output_index))
        .context("build scripts failed")?;

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
