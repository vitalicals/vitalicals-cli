use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{assert_output::InstructionOutputAssert, Instruction},
};
use vital_script_primitives::names::{Name, ShortName, NAME_LEN_MAX, SHORT_NAME_LEN_MAX};

/// Build a script to mint a short name / name to a output index.
pub fn mint_name(output_index: u32, name: impl Into<String>) -> Result<Vec<u8>> {
    if output_index >= u8::MAX as u32 {
        bail!("the output index not supported >= {}", u8::MAX);
    }

    let output_index = output_index as u8;

    let mut instructions = vec![Instruction::Output(InstructionOutputAssert { indexs: vec![0] })];

    let name: String = name.into();
    let mint_instruction = if name.len() <= SHORT_NAME_LEN_MAX {
        let name = ShortName::try_from(name).context("the name format not valid")?;
        Instruction::mint(output_index, name)
    } else if name.len() <= NAME_LEN_MAX {
        let name = Name::try_from(name).context("the name format not valid")?;
        Instruction::mint(output_index, name)
    } else {
        bail!("not support long name with length {}, need <= {}", name.len(), NAME_LEN_MAX);
    };

    instructions.push(mint_instruction);

    let ops_bytes =
        ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

    Ok(ops_bytes)
}
