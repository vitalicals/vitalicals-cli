//! A Runner for vital scripts
//!
//! It will run a vital script then call the impl callback.
//!
//! A Runner need depend the env trait which mainly contains the resource interface.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use anyhow::{Context as AnyhowContext, Result};

pub mod traits;

mod context;
mod outputs;
mod resource_cache;

#[cfg(test)]
mod mock;

use context::Context;
use traits::EnvFunctions;
use vital_script_ops::instruction::{Instruction, VitalInstruction};

pub struct Runner<Functions: EnvFunctions> {
    instructions: Vec<Instruction>,
    context: Context<Functions>,
}

impl<Functions: EnvFunctions> Runner<Functions> {
    pub fn new(context: Context<Functions>) -> Result<Self> {
        let instructions = context.get_instructions().context("get instructions")?;

        Ok(Self { instructions, context })
    }

    pub fn run(&mut self) -> Result<()> {
        // 1. run pre check
        self.pre_check().context("pre check")?;

        // 2. run opcodes, cost input resources, call env traits.
        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction
                .exec(&mut self.context)
                .with_context(|| format!("execute {}", index))?;
        }

        // 3. post check
        self.post_check().context("post check")?;

        // 4. apply the resources
        self.context.apply_resources().context("apply")?;

        Ok(())
    }
}

impl<Functions: EnvFunctions> Runner<Functions> {
    // The pre checks for context and instructions.
    fn pre_check(&self) -> Result<()> {
        self.context.pre_check().context("context")?;

        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction.pre_check().with_context(|| format!("instruction {}", index))?;
        }

        Ok(())
    }

    // The post check
    fn post_check(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vital_script_ops::{
        builder::instruction::ScriptBuilderFromInstructions,
        instruction::assert_output::InstructionOutputAssert,
    };
    use vital_script_primitives::{names::Name, resources::Resource};

    use super::*;
    use crate::mock::*;

    #[test]
    fn test_simple_runner() {
        let mint_name = Name::try_from("abcdefg".to_string()).unwrap();
        let instructions = vec![
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::mint(0, Resource::name(mint_name)),
        ];
        let ops_bytes =
            ScriptBuilderFromInstructions::build(instructions).expect("build should ok");

        let mut tx_mock = TxMock::new();
        tx_mock.push_ops(ops_bytes);
        tx_mock.push_output(1000);

        let env_interface = EnvMock::new(tx_mock);
        let context = Context::new(env_interface.clone());

        let mut runner = Runner::new(context).expect("new runner");

        runner.run().expect("run failed");

        println!("res storage {:?}", env_interface.resource_storage);
    }
}
