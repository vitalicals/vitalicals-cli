use alloc::vec::Vec;
use anyhow::{bail, Context, Result};
use bytes::{Buf, Bytes};

use crate::{
    instruction::Instruction,
    op_basic, op_extension,
    opcodes::{BasicOp, ExtensionOp},
};

pub struct Parser {
    datas: Bytes,
}

impl Parser {
    pub fn new(datas: &[u8]) -> Self {
        Self { datas: Bytes::copy_from_slice(datas) }
    }

    pub fn parse(&mut self) -> Result<Vec<Instruction>> {
        let mut res = Vec::with_capacity(16);

        while !self.datas.is_empty() {
            let opcodes_0 = self.datas.get_u8();
            let remaining = self.datas.remaining();

            let instruction = if opcodes_0 < 0x80 {
                // a basic opcodes
                self.parse_basic_instruction(remaining, opcodes_0)
                    .context("parse_basic_instruction")?
            } else {
                if remaining < 1 {
                    bail!("Invalid opcodes for extend opcode")
                }
                let opcodes_1 = self.datas.get_u8();
                let opcodes = u16::from_be_bytes([opcodes_0, opcodes_1]);

                // a extend opcodes
                self.parse_extend_instruction(remaining - 1, opcodes)
                    .context("parse_extend_instruction")?
            };

            res.push(instruction);
        }

        Ok(res)
    }

    fn parse_basic_instruction(&mut self, _remaining: usize, opcode: u8) -> Result<Instruction> {
        let opcode = BasicOp::new(opcode).context("basic op")?;

        macro_rules! decode_operand {
            ( $x:ident ) => {
                BasicOp::decode_operand::<op_basic::$x>(&mut self.datas)?
            };
        }

        macro_rules! decode_operands {
            ( $($x:ident),* ) => {
                match opcode {
                    $(
                        BasicOp::$x => decode_operand!($x),
                    )*
                    _ => panic!("Not supported opcode")
                }
            }
        }

        let res = decode_operands!(
            OutputIndexAssert,
            OutputIndexFlag16Assert,
            OutputIndexFlag32Assert,
            InputAssertShortName,
            InputAssertName,
            InputVRC20AssertSa32,
            InputVRC20AssertSa64,
            InputVRC20AssertSa128,
            InputVRC20AssertSa256,
            InputVRC20AssertA32,
            InputVRC20AssertA64,
            InputVRC20AssertA128,
            InputVRC20AssertA256,
            InputVRC721Assert,
            TransferAllVRC20S,
            TransferAllVRC20,
            TransferVRC20Sa32,
            TransferVRC20A32,
            MintShortName,
            MintName
        );

        Ok(res)
    }

    fn parse_extend_instruction(&mut self, _remaining: usize, opcode: u16) -> Result<Instruction> {
        println!("opcode {}", opcode);
        let opcode = ExtensionOp::new(opcode).context("extension op")?;

        macro_rules! decode_operand {
            ( $x:ident ) => {
                ExtensionOp::decode_operand::<op_extension::$x>(&mut self.datas)?
            };
        }

        macro_rules! decode_operands {
            ( $($x:ident),* ) => {
                match opcode {
                    $(
                        ExtensionOp::$x => decode_operand!($x),
                    )*
                    _ => panic!("Not supported opcode")
                }
            }
        }

        let res = decode_operands!(
            // OutputIndexFlag64Assert,
            DeployVRC20S,
            DeployVRC20
        );

        Ok(res)
    }
}
