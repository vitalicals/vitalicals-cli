use vital_script_instruction::{assert_output::InstructionOutputAssert, Instruction};

use crate::opcodes::BasicOp;

use super::BasicOpcode;

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
pub struct OutputIndexAssert {
    pub index: u8,
}

impl BasicOpcode for OutputIndexAssert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexAssert as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Output(InstructionOutputAssert { indexs: vec![self.index] })
    }
}

/// Output Index Assert By A u16 as FlagMask
pub struct OutputIndexFlag16Assert {
    pub index_flag: [u8; 2],
}

impl BasicOpcode for OutputIndexFlag16Assert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexFlag16Assert as u8
    }

    fn into_instruction(self) -> Instruction {
        let indexs = [u8_to_pos(self.index_flag[0], 0), u8_to_pos(self.index_flag[1], 1)].concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}

/// Output Index Assert By A u32 as FlagMask
pub struct OutputIndexFlag32Assert {
    pub index_flag: [u8; 4],
}

impl BasicOpcode for OutputIndexFlag32Assert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexFlag32Assert as u8
    }

    fn into_instruction(self) -> Instruction {
        let indexs = [0_u8, 1, 2, 3].map(|c| u8_to_pos(self.index_flag[c as usize], c)).concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}
