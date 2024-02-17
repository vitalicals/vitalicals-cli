use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, Tag, VRC20, VRC721},
    H256, U256,
};

use crate::instruction::{assert_input::InstructionInputAssert, Instruction};

/// Input ShortName Res Assert
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct InputAssertShortName {
    pub name: ShortName,
    pub index: u8,
}

impl From<InputAssertShortName> for Instruction {
    fn from(value: InputAssertShortName) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::Name(value.name.into()),
        })
    }
}

/// Input Name Res Assert
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct InputAssertName {
    pub name: Name,
    pub index: u8,
}

impl From<InputAssertName> for Instruction {
    fn from(value: InputAssertName) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::Name(value.name),
        })
    }
}

/// Input Long Name Res Assert
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct InputAssertLongName {
    pub name: Tag, // FIXME: add long name
    pub index: u8,
}

impl From<InputAssertLongName> for Instruction {
    fn from(value: InputAssertLongName) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::Name(value.name),
        })
    }
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (ShortName, u64 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (ShortName, u128 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (ShortName, u256 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (ShortName, u32 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (Name, u64 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (Name, u128 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC20 Res Assert for (Name, u256 amount)
#[derive(Debug, BasicOpcode, Encode, Decode)]
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

/// Input VRC721 Res Assert for (Name, hash256 )
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct InputVRC721Assert {
    pub hash: H256,
    pub index: u8,
}

impl From<InputVRC721Assert> for Instruction {
    fn from(value: InputVRC721Assert) -> Self {
        Instruction::Input(InstructionInputAssert {
            index: value.index,
            resource: Resource::VRC721(VRC721::new(value.hash)),
        })
    }
}
