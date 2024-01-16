//! The id def for opcode

use anyhow::{bail, Context, Result};
use bytes::Bytes;

use crate::instruction::Instruction;

pub use crate::op_basic::BasicOpcode;

#[repr(u8)]
pub enum BasicOp {
    OutputIndexAssert = 0x0a,
    OutputIndexFlag16Assert = 0x0b,
    OutputIndexFlag32Assert = 0x0c,

    InputVRC20AssertSa32 = 0x0d,
    InputVRC20AssertSa64,  // = 0x0e,
    InputVRC20AssertSa128, // = 0x0f,
    InputVRC20AssertSa256, // = 0x10,

    InputVRC20AssertA32 = 0x11,
    InputVRC20AssertA64,  // = 0x12,
    InputVRC20AssertA128, // = 0x13,
    InputVRC20AssertA256, // = 0x14,

    InputVRC721Assert = 0x15,

    TransferAllVRC20S = 0x16, // 22
    TransferAllVRC20 = 0x17,

    TransferVRC20Sa32 = 0x18, // 24
    TransferVRC20Sa64,        // = 0x19,
    TransferVRC20Sa128,       // = 0x1a,
    TransferVRC20Sa256,       // = 0x1b,

    TransferVRC20A32,  // = 0x1c,
    TransferVRC20A64,  // = 0x1d,
    TransferVRC20A128, // = 0x1e,
    TransferVRC20A256, // = 0x1f,

    TransferVRC721 = 0x20,

    MintShortName, // = 0x21,
    MintName,      // = 0x22,

    MintShortVRC20, // = 0x23,
    MintVRC20,      // = 0x24,
    MintVRC721,     // = 0x25,

    DMintShortVRC20, // = 0x26,
    DMintVRC20,      // = 0x27,
    DMintVRC721,     // = 0x28,

                     // Mint
                     // Burn
}

impl BasicOp {
    pub fn new(v: u8) -> Result<Self> {
        match v {
            0x0a => Ok(Self::OutputIndexAssert),
            0x0b => Ok(Self::OutputIndexFlag16Assert),
            0x0c => Ok(Self::OutputIndexFlag32Assert),
            0x0d => Ok(Self::InputVRC20AssertSa32),
            0x0e => Ok(Self::InputVRC20AssertSa64),
            0x0f => Ok(Self::InputVRC20AssertSa128),
            0x10 => Ok(Self::InputVRC20AssertSa256),
            0x11 => Ok(Self::InputVRC20AssertA32),
            0x12 => Ok(Self::InputVRC20AssertA64),
            0x13 => Ok(Self::InputVRC20AssertA128),
            0x14 => Ok(Self::InputVRC20AssertA256),
            0x15 => Ok(Self::InputVRC721Assert),
            0x16 => Ok(Self::TransferAllVRC20S),
            0x17 => Ok(Self::TransferAllVRC20),
            0x18 => Ok(Self::TransferVRC20Sa32),

            0x21 => Ok(Self::MintShortName),
            0x22 => Ok(Self::MintName),

            _ => bail!("not supported op {}", v),
        }
    }

    pub fn decode_operand<Op>(datas: &mut Bytes) -> Result<Instruction>
    where
        Op: BasicOpcode,
    {
        Ok(Op::decode_operand(datas).context("decode operand")?.into())
    }
}
