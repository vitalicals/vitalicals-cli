use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;

use crate::instruction::{assert_output::InstructionOutputAssert, Instruction};

#[inline]
fn u8_to_pos(i: u8, c: u8) -> Vec<u8> {
    let mut res = Vec::new();

    for pos in 0..8 {
        let mask = 1 << pos;
        if (mask & i) != 0 {
            res.push(8 * c + pos);
        }
    }

    res
}

/// Output Index Assert by 1 indexs
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexAssert {
    pub index: u8,
}

impl From<OutputIndexAssert> for Instruction {
    fn from(value: OutputIndexAssert) -> Self {
        Instruction::Output(InstructionOutputAssert { indexs: vec![value.index] })
    }
}

/// Output Index Assert By A u16 as FlagMask
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexFlag16Assert {
    pub index_flag: [u8; 2],
}

impl From<OutputIndexFlag16Assert> for Instruction {
    fn from(value: OutputIndexFlag16Assert) -> Self {
        let indexs =
            [u8_to_pos(value.index_flag[0], 0), u8_to_pos(value.index_flag[1], 1)].concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}

/// Output Index Assert By A u32 as FlagMask
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexFlag32Assert {
    pub index_flag: [u8; 4],
}

impl From<OutputIndexFlag32Assert> for Instruction {
    fn from(value: OutputIndexFlag32Assert) -> Self {
        let indexs = [0_u8, 1, 2, 3].map(|c| u8_to_pos(value.index_flag[c as usize], c)).concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}
