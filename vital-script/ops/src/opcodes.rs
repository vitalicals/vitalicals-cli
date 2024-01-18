//! The id def for opcode

use anyhow::{bail, Context, Result};
use bytes::Bytes;

use crate::{instruction::Instruction, op_extension::ExtensionOpcode};

pub use crate::op_basic::BasicOpcode;

#[repr(u8)]
pub enum BasicOp {
    OutputIndexAssert = 0x0a,
    OutputIndexFlag16Assert = 0x0b,
    OutputIndexFlag32Assert = 0x0c,

    InputAssertShortName = 0x0d,
    InputAssertName,     // = 0x0e,
    InputAssertLongName, // = 0x0f,

    InputVRC20AssertSa32 = 0x10,
    InputVRC20AssertSa64,  // = 0x11,
    InputVRC20AssertSa128, // = 0x12,
    InputVRC20AssertSa256, // = 0x13,

    InputVRC20AssertA32 = 0x14,
    InputVRC20AssertA64,  // = 0x15,
    InputVRC20AssertA128, // = 0x16,
    InputVRC20AssertA256, // = 0x17,

    InputVRC721Assert = 0x18,

    TransferAllVRC20S = 0x19,
    TransferAllVRC20 = 0x1a,

    TransferVRC20Sa32 = 0x1b, //
    TransferVRC20Sa64,        // = 0x1c,
    TransferVRC20Sa128,       // = 0x1d,
    TransferVRC20Sa256,       // = 0x1e,

    TransferVRC20A32,  // = 0x1f,
    TransferVRC20A64,  // = 0x20,
    TransferVRC20A128, // = 0x21,
    TransferVRC20A256, // = 0x22,

    TransferVRC721 = 0x23,

    MintShortName, // = 0x24,
    MintName,      // = 0x25,

    MintShortVRC20, // = 0x26,
    MintVRC20,      // = 0x27,
    MintVRC721,     // = 0x28,

    DMintShortVRC20, // = 0x29,
    DMintVRC20,      // = 0x2a,
    DMintVRC721,     // = 0x2b,

                     // Mint
                     // Burn
}

impl BasicOp {
    pub fn new(v: u8) -> Result<Self> {
        match v {
            0x0a => Ok(Self::OutputIndexAssert),
            0x0b => Ok(Self::OutputIndexFlag16Assert),
            0x0c => Ok(Self::OutputIndexFlag32Assert),
            0x0d => Ok(Self::InputAssertShortName),
            0x0e => Ok(Self::InputAssertName),
            0x0f => Ok(Self::InputAssertLongName),
            0x10 => Ok(Self::InputVRC20AssertSa32),
            0x11 => Ok(Self::InputVRC20AssertSa64),
            0x12 => Ok(Self::InputVRC20AssertSa128),
            0x13 => Ok(Self::InputVRC20AssertSa256),
            0x14 => Ok(Self::InputVRC20AssertA32),
            0x15 => Ok(Self::InputVRC20AssertA64),
            0x16 => Ok(Self::InputVRC20AssertA128),
            0x17 => Ok(Self::InputVRC20AssertA256),
            0x18 => Ok(Self::InputVRC721Assert),
            0x19 => Ok(Self::TransferAllVRC20S),
            0x1a => Ok(Self::TransferAllVRC20),
            0x1b => Ok(Self::TransferVRC20Sa32),
            0x1c => Ok(Self::TransferVRC20Sa64),
            0x1d => Ok(Self::TransferVRC20Sa128),
            0x1e => Ok(Self::TransferVRC20Sa256),
            0x1f => Ok(Self::TransferVRC20A32),
            0x20 => Ok(Self::TransferVRC20A64),
            0x21 => Ok(Self::TransferVRC20A128),
            0x22 => Ok(Self::TransferVRC20A256),
            0x23 => Ok(Self::TransferVRC721),
            0x24 => Ok(Self::MintShortName),
            0x25 => Ok(Self::MintName),
            0x26 => Ok(Self::MintShortVRC20),
            0x27 => Ok(Self::MintVRC20),
            0x28 => Ok(Self::MintVRC721),

            0x29 => Ok(Self::DMintShortVRC20),
            0x2a => Ok(Self::DMintVRC20),
            0x2b => Ok(Self::DMintVRC721),

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

#[repr(u16)]
pub enum ExtensionOp {
    OutputIndexFlag64Assert = 0x8001,
    DeployVRC20S = 0x8002,
    DeployVRC20 = 0x8003,
}

impl ExtensionOp {
    pub fn new(v: u16) -> Result<Self> {
        match v {
            0x8001 => Ok(Self::OutputIndexFlag64Assert),
            0x8002 => Ok(Self::DeployVRC20S),
            0x8003 => Ok(Self::DeployVRC20),

            _ => bail!("not supported op {}", v),
        }
    }

    pub fn decode_operand<Op>(datas: &mut Bytes) -> Result<Instruction>
    where
        Op: ExtensionOpcode,
    {
        Ok(Op::decode_operand(datas).context("decode operand")?.into())
    }
}
