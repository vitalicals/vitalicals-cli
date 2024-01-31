use std::collections::BTreeMap;

use anyhow::{bail, Context, Result};
use vital_script_ops::{
    builder::instruction::ScriptBuilderFromInstructions,
    instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert, Instruction,
    },
};
use vital_script_primitives::{
    names::Name,
    resources::{Resource, VRC20},
    U256,
};

use super::Outputs;

pub fn move_vrc20_with_charge(
    name: Name,
    input_index: u8,
    in_amount: U256,
    to_amount: U256,
) -> Result<(Outputs, Vec<u8>)> {
    let mut builder = MoveVrc20InstructionBuilder::new();

    if in_amount < to_amount {
        bail!("the input {} less then output amount {}", in_amount, to_amount);
    }

    let output_index = 0;
    let mut outputs = vec![output_index];
    let charge = in_amount - to_amount;

    builder.append_input(input_index, name, in_amount);
    builder.append_output(output_index, name, to_amount);

    if !charge.is_zero() {
        outputs.push(output_index + 1);
        builder.append_output(output_index + 1, name, charge);
    }

    let res = builder.build().context("build")?;

    Ok((outputs, res))
}

pub fn move_vrc20s_with_charge(
    name: Name,
    inputs: Vec<(u8, U256)>,
    to_amount: U256,
) -> Result<(Outputs, Vec<u8>)> {
    let mut builder = MoveVrc20InstructionBuilder::new();

    let input_sum = {
        let mut sum = U256::zero();
        for (_, amount) in inputs.iter() {
            sum += *amount;
        }

        sum
    };

    if input_sum < to_amount {
        bail!("the input {} less then output amount {}", input_sum, to_amount);
    }

    let output_index = 0;
    let mut outputs = vec![output_index];
    let charge = input_sum - to_amount;

    for (input, amount) in inputs.into_iter() {
        builder.append_input(input, name, amount);
    }

    builder.append_output(output_index, name, to_amount);

    if !charge.is_zero() {
        outputs.push(output_index + 1);
        builder.append_output(output_index + 1, name, charge);
    }

    let res = builder.build().context("build")?;

    Ok((outputs, res))
}

pub fn merge_vrc20s(name: Name, inputs: Vec<(u8, U256)>) -> Result<Vec<u8>> {
    let to_amount = {
        let mut sum = U256::zero();
        for (_, amount) in inputs.iter() {
            sum += *amount;
        }

        sum
    };

    let mut builder = MoveVrc20InstructionBuilder::new();

    let output_index = 0;
    for (index, amount) in inputs.into_iter() {
        builder.append_input(index, name, amount);
    }
    builder.append_output(output_index, name, to_amount);

    builder.build()
}

pub fn split_vrc20s(name: Name, input_index: u8, outputs: Vec<U256>) -> Result<Vec<u8>> {
    let input_amount = {
        let mut sum = U256::zero();
        for amount in outputs.iter() {
            sum += *amount;
        }

        sum
    };

    let mut builder = MoveVrc20InstructionBuilder::new();

    builder.append_input(input_index, name, input_amount);

    let output_index_start = 0;
    for (index, amount) in outputs.into_iter().enumerate() {
        let output_index = output_index_start + index;
        if output_index >= u8::MAX as usize {
            bail!("the output index not supported >= {}", u8::MAX);
        }

        builder.append_output(output_index as u8, name, amount);
    }

    builder.build()
}

#[derive(Default)]
pub struct MoveVrc20InstructionBuilder {
    inputs: Vec<(u8, Name, U256)>,
    outputs: Vec<(u8, Name, U256)>,
}

impl MoveVrc20InstructionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_input_output_eq(&self) -> bool {
        self.merged_inputs() == self.merged_outputs()
    }

    pub fn append_input(&mut self, input_index: u8, name: Name, amount: U256) {
        self.inputs.push((input_index, name, amount))
    }

    pub fn append_output(&mut self, output_index: u8, name: Name, amount: U256) {
        self.outputs.push((output_index, name, amount))
    }

    pub fn build(self) -> Result<Vec<u8>> {
        if !self.is_input_output_eq() {
            bail!("the input and output not eq!");
        }

        let output_indexs = self
            .outputs
            .iter()
            .map(|(output_index, _, _)| Ok(*output_index))
            .collect::<Result<Vec<_>>>()
            .context("output index")?;

        let mut instructions =
            [Instruction::Output(InstructionOutputAssert { indexs: output_indexs })].to_vec();

        for (input_index, name, amount) in self.inputs.iter() {
            instructions.push(Instruction::Input(InstructionInputAssert {
                index: *input_index,
                resource: Resource::VRC20(VRC20::new(*name, *amount)),
            }));
        }

        for (output_index, name, amount) in self.outputs.iter() {
            let move_instruction =
                Instruction::move_to(*output_index, Resource::VRC20(VRC20::new(*name, *amount)));

            instructions.push(move_instruction);
        }

        let ops_bytes =
            ScriptBuilderFromInstructions::build(instructions).context("build script failed")?;

        Ok(ops_bytes)
    }

    fn merged_inputs(&self) -> Vec<(Name, U256)> {
        let mut inputs_map = BTreeMap::new();

        for (_, name, amount) in self.inputs.iter() {
            match inputs_map.get_mut(name) {
                None => {
                    inputs_map.insert(*name, *amount);
                }
                Some(sum) => {
                    *sum += *amount;
                }
            }
        }

        let mut res = inputs_map.into_iter().collect::<Vec<_>>();
        res.sort_by(|a, b| a.0.cmp(&b.0));

        res
    }

    fn merged_outputs(&self) -> Vec<(Name, U256)> {
        let mut outputs_map = BTreeMap::new();

        for (_, name, amount) in self.outputs.iter() {
            match outputs_map.get_mut(name) {
                None => {
                    outputs_map.insert(*name, *amount);
                }
                Some(sum) => {
                    *sum += *amount;
                }
            }
        }

        let mut res = outputs_map.into_iter().collect::<Vec<_>>();
        res.sort_by(|a, b| a.0.cmp(&b.0));

        res
    }
}
