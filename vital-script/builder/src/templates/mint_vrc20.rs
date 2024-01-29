use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{assert_output::InstructionOutputAssert, Instruction},
};
use vital_script_primitives::{names::Name, resources::ResourceType};

/// Build a script to mint a short name / name to a output index.
pub fn mint_vrc20(output_index: u32, vrc20_name: Name) -> Result<Vec<u8>> {
    if output_index >= u8::MAX as u32 {
        bail!("the output index not supported >= {}", u8::MAX);
    }

    let output_index = output_index as u8;

    let instructions = [
        Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
        Instruction::mint(output_index, ResourceType::vrc20(vrc20_name)),
    ]
    .to_vec();

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}
