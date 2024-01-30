use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{assert_output::InstructionOutputAssert, Instruction},
};
use vital_script_primitives::names::Name;

/// Build a script to move a short name / name to a output index.
pub fn move_name(output_index: u32, name: Name) -> Result<Vec<u8>> {
    if output_index >= u8::MAX as u32 {
        bail!("the output index not supported >= {}", u8::MAX);
    }

    let output_index = output_index as u8;

    let instructions = [
        Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
        Instruction::move_to(output_index, name),
    ]
    .to_vec();

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}

/// Build a script to move a short name / name to a output index.
pub fn move_names_with_index(output_index_name: &[(u32, Name)]) -> Result<Vec<u8>> {
    let outputs = output_index_name
        .iter()
        .map(|(output_index, _)| {
            if *output_index >= u8::MAX as u32 {
                bail!("the output index not supported >= {}", u8::MAX);
            }
            Ok(*output_index as u8)
        })
        .collect::<Result<Vec<_>>>()
        .context("output index")?;

    let mut instructions =
        [Instruction::Output(InstructionOutputAssert { indexs: outputs })].to_vec();

    for (output_index, name) in output_index_name.iter() {
        if *output_index >= u8::MAX as u32 {
            bail!("the output index not supported >= {}", u8::MAX);
        }

        let output_index = *output_index as u8;
        let move_instruction = Instruction::move_to(output_index, *name);

        instructions.push(move_instruction);
    }

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}

/// Build a script to move a short name / name to output index from 0.
pub fn move_names(names: &[Name], start_output_index: Option<u32>) -> Result<Vec<u8>> {
    let start_output_index = start_output_index.unwrap_or_default();

    let outputs = (0..names.len())
        .into_iter()
        .map(|i| {
            let output_index = start_output_index + i as u32;
            if output_index >= u8::MAX as u32 {
                bail!("the output index not supported >= {}", u8::MAX);
            }
            Ok(output_index as u8)
        })
        .collect::<Result<Vec<_>>>()
        .context("output index")?;

    let mut instructions =
        [Instruction::Output(InstructionOutputAssert { indexs: outputs.clone() })].to_vec();

    for (index, name) in names.iter().enumerate() {
        let output_index = outputs[index];
        let move_instruction = Instruction::move_to(output_index, *name);

        instructions.push(move_instruction);
    }

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}
