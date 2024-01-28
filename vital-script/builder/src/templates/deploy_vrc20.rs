use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{
        assert_input::InstructionInputAssert, resource_deploy::InstructionVRC20Deploy, Instruction,
    },
};
use vital_script_primitives::{names::Name, resources::Resource, types::vrc20::VRC20MetaData};

/// Build a script to mint a short name / name to a output index.
pub fn deploy_vrc20(input_index: u32, name: Name, meta: VRC20MetaData) -> Result<Vec<u8>> {
    if input_index >= u8::MAX as u32 {
        bail!("the output index not supported >= {}", u8::MAX);
    }

    let input_index = input_index as u8;

    let name_resource = Resource::Name(name);

    let mut instructions = [Instruction::Input(InstructionInputAssert {
        index: input_index,
        resource: name_resource,
    })]
    .to_vec();

    let deploy_instruction =
        Instruction::Deploy(InstructionVRC20Deploy { name_input: input_index, name, meta });

    instructions.push(deploy_instruction);

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}
