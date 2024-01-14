use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::names::{Name, ShortName};

use crate::instruction::Instruction;

/// Mint short name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintShortName {
    pub name: ShortName,
    pub index: u8,
}

impl From<MintShortName> for Instruction {
    fn from(value: MintShortName) -> Self {
        Instruction::mint(value.index, value.name)
    }
}

/// Mint name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintName {
    pub name: Name,
    pub index: u8,
}

impl From<MintName> for Instruction {
    fn from(value: MintName) -> Self {
        Instruction::mint(value.index, value.name)
    }
}
