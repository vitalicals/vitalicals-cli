use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert, Instruction,
    },
};
use vital_script_primitives::{resources::Resource, H256};

pub fn move_vrc721s(inputs: &[H256], start_output_index: Option<u32>) -> Result<Vec<u8>> {
    let start_output_index = start_output_index.unwrap_or_default();

    let outputs = (0..inputs.len())
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

    // input assert
    for (input_index, hash) in inputs.iter().enumerate() {
        let input_index = input_index + 1; // all inputs is from 1

        if input_index >= u8::MAX as usize {
            bail!("the input index too large");
        }

        instructions.push(Instruction::Input(InstructionInputAssert {
            index: input_index as u8,
            resource: Resource::vrc721(*hash),
        }))
    }

    // move instruction
    for (index, hash) in inputs.iter().enumerate() {
        let output_index = outputs[index];
        let move_instruction = Instruction::move_to(output_index, Resource::vrc721(*hash));

        instructions.push(move_instruction);
    }

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}
