use anyhow::{bail, Context, Result};
use bytes::{Buf, Bytes};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, VRC20, VRC721},
    H256, U256,
};

use crate::{
    instruction::{assert_input::InstructionInputAssert, Instruction},
    opcodes::BasicOp,
};

use super::*;

/// Input VRC20 Res Assert for (ShortName, u32 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertSa32 {
    pub amount: u32,
    pub name: ShortName,
    pub index: u8,
}

impl From<InputVRC20AssertSa32> for Instruction {
    fn from(value: InputVRC20AssertSa32) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name.into(), value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertSa32 {
    const ID: u8 = BasicOp::InputVRC20AssertSa32 as u8;
}

/// Input VRC20 Res Assert for (ShortName, u64 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertSa64 {
    pub amount: u64,
    pub name: ShortName,
    pub index: u8,
}

impl From<InputVRC20AssertSa64> for Instruction {
    fn from(value: InputVRC20AssertSa64) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name.into(), value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertSa64 {
    const ID: u8 = BasicOp::InputVRC20AssertSa64 as u8;
}

/// Input VRC20 Res Assert for (ShortName, u128 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertSa128 {
    pub amount: u128,
    pub name: ShortName,
    pub index: u8,
}

impl From<InputVRC20AssertSa128> for Instruction {
    fn from(value: InputVRC20AssertSa128) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name.into(), value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertSa128 {
    const ID: u8 = BasicOp::InputVRC20AssertSa128 as u8;
}

/// Input VRC20 Res Assert for (ShortName, u256 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertSa256 {
    pub amount: U256,
    pub name: ShortName,
    pub index: u8,
}

impl From<InputVRC20AssertSa256> for Instruction {
    fn from(value: InputVRC20AssertSa256) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name.into(), value.amount)),
        })
    }
}

impl Opcode for InputVRC20AssertSa256 {
    const ID: u8 = BasicOp::InputVRC20AssertSa256 as u8;
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertA32 {
    pub amount: u32,
    pub name: Name,
    pub index: u8,
}

impl From<InputVRC20AssertA32> for Instruction {
    fn from(value: InputVRC20AssertA32) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name, value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertA32 {
    const ID: u8 = BasicOp::InputVRC20AssertA32 as u8;
}

/// Input VRC20 Res Assert for (Name, u64 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertA64 {
    pub amount: u64,
    pub name: Name,
    pub index: u8,
}

impl From<InputVRC20AssertA64> for Instruction {
    fn from(value: InputVRC20AssertA64) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name, value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertA64 {
    const ID: u8 = BasicOp::InputVRC20AssertA64 as u8;
}

/// Input VRC20 Res Assert for (Name, u128 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertA128 {
    pub amount: u128,
    pub name: Name,
    pub index: u8,
}

impl From<InputVRC20AssertA128> for Instruction {
    fn from(value: InputVRC20AssertA128) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name, value.amount.into())),
        })
    }
}

impl Opcode for InputVRC20AssertA128 {
    const ID: u8 = BasicOp::InputVRC20AssertA128 as u8;
}

/// Input VRC20 Res Assert for (Name, u256 amount)
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC20AssertA256 {
    pub amount: U256,
    pub name: Name,
    pub index: u8,
}

impl From<InputVRC20AssertA256> for Instruction {
    fn from(value: InputVRC20AssertA256) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC20(VRC20::new(value.name, value.amount)),
        })
    }
}

impl Opcode for InputVRC20AssertA256 {
    const ID: u8 = BasicOp::InputVRC20AssertA256 as u8;
}

/// Input VRC721 Res Assert for (Name, hash256 )
#[derive(Debug, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct InputVRC721Assert {
    pub hash: H256,
    pub name: Name,
    pub index: u8,
}

impl From<InputVRC721Assert> for Instruction {
    fn from(value: InputVRC721Assert) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC721(VRC721::new(value.name, value.hash)),
        })
    }
}

impl Opcode for InputVRC721Assert {
    const ID: u8 = BasicOp::InputVRC721Assert as u8;
}
