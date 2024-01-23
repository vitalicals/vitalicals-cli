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
    InputVRC20AssertA32,   // = 0x14,
    InputVRC20AssertA64,   // = 0x15,
    InputVRC20AssertA128,  // = 0x16,
    InputVRC20AssertA256,  // = 0x17,

    InputVRC721Assert = 0x18,

    MoveShortName = 0x19,
    MoveName,
    MoveLongName,

    MoveAllVRC20S = 0x1c,
    MoveAllVRC20 = 0x1d,

    MoveVRC20Sa32 = 0x1e, //
    MoveVRC20Sa64,        // = 0x1f,
    MoveVRC20Sa128,       // = 0x20,
    MoveVRC20Sa256,       // = 0x21,
    MoveVRC20A32,         // = 0x22,
    MoveVRC20A64,         // = 0x23,
    MoveVRC20A128,        // = 0x24,
    MoveVRC20A256,        // = 0x25,

    MoveVRC721 = 0x26,

    MintShortName,  // = 0x27,
    MintName,       // = 0x28,
    MintShortVRC20, // = 0x29,
    MintVRC20,      // = 0x2a,
    MintVRC721,     // = 0x2b,

    DMintShortName = 0x2c, // = 0x2c,
    DMintName,             // = 0x2d,
    DMintShortVRC20,       // = 0x2e,
    DMintVRC20,            // = 0x2f,
    DMintVRC721,           // = 0x30,

                           // TODO: Burn
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
            0x19 => Ok(Self::MoveShortName),
            0x1a => Ok(Self::MoveName),
            0x1b => Ok(Self::MoveLongName),
            0x1c => Ok(Self::MoveAllVRC20S),
            0x1d => Ok(Self::MoveAllVRC20),
            0x1e => Ok(Self::MoveVRC20Sa32),
            0x1f => Ok(Self::MoveVRC20Sa64),
            0x20 => Ok(Self::MoveVRC20Sa128),
            0x21 => Ok(Self::MoveVRC20Sa256),
            0x22 => Ok(Self::MoveVRC20A32),
            0x23 => Ok(Self::MoveVRC20A64),
            0x24 => Ok(Self::MoveVRC20A128),
            0x25 => Ok(Self::MoveVRC20A256),
            0x26 => Ok(Self::MoveVRC721),
            0x27 => Ok(Self::MintShortName),
            0x28 => Ok(Self::MintName),
            0x29 => Ok(Self::MintShortVRC20),
            0x2a => Ok(Self::MintVRC20),
            0x2b => Ok(Self::MintVRC721),
            0x2c => Ok(Self::DMintShortName),
            0x2d => Ok(Self::DMintName),
            0x2e => Ok(Self::DMintShortVRC20),
            0x2f => Ok(Self::DMintVRC20),
            0x30 => Ok(Self::DMintVRC721),

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
