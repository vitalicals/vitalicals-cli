use anyhow::{bail, Context as AnyhowContext, Result};

use vital_script_primitives::{resources::Name, U256};

use crate::Context;

pub async fn move_vrc20(
    context: &mut Context,
    name: &String,
    amount: U256,
    sats: u64,
) -> Result<()> {
    use vital_script_builder::templates;

    let vrc20_name =
        Name::try_from(name.clone()).with_context(|| format!("name {} format invalid", name))?;

    let (sum, mut owned_vrc20s) = context
        .fetch_all_vrc20_by_name(vrc20_name)
        .await
        .context("fetch_all_vrc20_by_name")?;

    if sum < amount {
        bail!("the vrc20 {} owned just {}, less then {:?}", name, sum, amount);
    }

    let mut inputs = Vec::with_capacity(owned_vrc20s.len());
    let mut utxos = Vec::with_capacity(owned_vrc20s.len());

    let mut pushed_amount = U256::zero();

    // TODO: we need a way to select inputs
    owned_vrc20s.sort_by(|a, b| {
        a.resource
            .as_vrc20()
            .expect("should be vrc20")
            .amount
            .cmp(&b.resource.as_vrc20().expect("should be vrc20").amount)
    });

    for (index, local) in owned_vrc20s.into_iter().enumerate() {
        if pushed_amount >= amount {
            break;
        }

        if index >= u8::MAX as usize {
            bail!("the index index not supported >= {}", u8::MAX);
        }

        let add_amount = local.resource.as_vrc20().expect("should be vrc20").amount;

        utxos.push(local.utxo);
        inputs.push((index as u8, add_amount));
        pushed_amount += add_amount;
    }

    // build script.
    // all begin with 0.
    let (outputs, scripts_bytes) = templates::move_vrc20s_with_charge(vrc20_name, inputs, amount)
        .context("build scripts failed")?;

    if outputs.len() > 2 {
        bail!("move_vrc20s_with_charge outputs len > 2");
    }

    context.append_reveal_input(&utxos);
    context.set_amount(sats);

    // the move had charge
    if outputs.len() == 2 {
        context.append_output(None, sats)
    }

    // build tx then send
    crate::send_p2tr(context, scripts_bytes).await.context("send_p2tr failed")?;

    Ok(())
}
