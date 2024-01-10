use anyhow::{bail, Result};
use bytes::{Buf, Bytes};

use crate::{
    instruction::{assert_output::InstructionOutputAssert, Instruction},
    opcodes::BasicOp,
};

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
    const ID: u8 = BasicOp::OutputIndexAssert as u8;
    const OPERAND_SIZE: usize = 1;

    fn into_instruction(self) -> Instruction {
        Instruction::Output(InstructionOutputAssert { indexs: vec![self.index] })
    }

    fn encode(self) -> Vec<u8> {
        vec![Self::ID, self.index]
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let index = datas.get_u8();

        Ok(Self { index })
    }
}

/// Output Index Assert By A u16 as FlagMask
pub struct OutputIndexFlag16Assert {
    pub index_flag: [u8; 2],
}

impl BasicOpcode for OutputIndexFlag16Assert {
    const ID: u8 = BasicOp::OutputIndexFlag16Assert as u8;
    const OPERAND_SIZE: usize = 2;

    fn into_instruction(self) -> Instruction {
        let indexs = [u8_to_pos(self.index_flag[0], 0), u8_to_pos(self.index_flag[1], 1)].concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }

    fn encode(self) -> Vec<u8> {
        vec![Self::ID, self.index_flag[0], self.index_flag[1]]
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let mut index_flag = [0_u8; 2];
        datas.copy_to_slice(&mut index_flag);

        Ok(Self { index_flag })
    }
}

/// Output Index Assert By A u32 as FlagMask
pub struct OutputIndexFlag32Assert {
    pub index_flag: [u8; 4],
}

impl BasicOpcode for OutputIndexFlag32Assert {
    const ID: u8 = BasicOp::OutputIndexFlag32Assert as u8;
    const OPERAND_SIZE: usize = 4;

    fn into_instruction(self) -> Instruction {
        let indexs = [0_u8, 1, 2, 3].map(|c| u8_to_pos(self.index_flag[c as usize], c)).concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }

    fn encode(self) -> Vec<u8> {
        vec![
            Self::ID,
            self.index_flag[0],
            self.index_flag[1],
            self.index_flag[2],
            self.index_flag[3],
        ]
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let mut index_flag = [0_u8; 4];
        datas.copy_to_slice(&mut index_flag);

        Ok(Self { index_flag })
    }
}
