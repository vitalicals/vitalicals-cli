//! A Script builder from the instructions
//!

use alloc::vec::Vec;
use anyhow::Result;

use vital_script_primitives::traits::Instruction as InstructionT;

use crate::instruction::Instruction;

pub struct ScriptBuilderFromInstructions {}

impl ScriptBuilderFromInstructions {
    pub fn build(instructions: Vec<Instruction>) -> Result<Vec<u8>> {
        Ok(instructions
            .into_iter()
            .map(|ins| ins.into_ops_bytes())
            .collect::<Result<Vec<_>>>()?
            .concat())
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Add;

    use vital_script_primitives::{resources::Resource, U256};

    use crate::{
        instruction::{assert_input::*, assert_output::*, *},
        parser::Parser,
    };

    use super::ScriptBuilderFromInstructions;

    #[test]
    fn test_build_input() {
        let instructions = vec![
            Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: Resource::vrc20("btc", U256::from(5000)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 2,
                resource: Resource::vrc20("optimism", U256::from(5000)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 3,
                resource: Resource::vrc20("btc", U256::from(u32::MAX)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 4,
                resource: Resource::vrc20("optimism", U256::from(u64::MAX)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 5,
                resource: Resource::vrc20("btc", U256::from(u128::MAX)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 6,
                resource: Resource::vrc20("optimism", U256::from(u128::MAX).add(U256::from(100)))
                    .unwrap(),
            }),
        ];

        let ops_bytes =
            ScriptBuilderFromInstructions::build(instructions).expect("build should ok");
        println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

        let mut parser = Parser::new(&ops_bytes);
        let instructions = parser.parse().expect("parser should ok");

        println!("instructions: {:?}", instructions);
    }

    #[ignore = "reason"]
    #[test]
    fn test_build_output() {
        let instructions = vec![
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![1] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![2] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![16] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2, 3, 8, 9, 10] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![1, 2, 3, 8, 9, 15] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![1, 15] }),
            Instruction::Output(InstructionOutputAssert { indexs: (0..=15).collect() }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![0, 16] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2, 3, 8, 9, 16] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![16, 17, 18, 19] }),
            Instruction::Output(InstructionOutputAssert { indexs: vec![1, 31] }),
            Instruction::Output(InstructionOutputAssert { indexs: (0..=31).collect() }),
        ];

        let ops_bytes =
            ScriptBuilderFromInstructions::build(instructions.clone()).expect("build should ok");
        println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

        let mut parser = Parser::new(&ops_bytes);
        let instructions_from = parser.parse().expect("parser should ok");

        println!("instructions: {:?}", instructions_from);

        assert_eq!(instructions, instructions_from);
    }
}
