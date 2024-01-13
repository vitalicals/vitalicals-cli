use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, VRC20, VRC721},
    H256, U256,
};

use crate::instruction::{Instruction, InstructionResourceMint};

/// Mint short name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintShortName {
    pub name: ShortName,
    pub index: u8,
}

impl From<MintShortName> for Instruction {
    fn from(value: MintShortName) -> Self {
        Instruction::Mint(InstructionResourceMint {
            output_index: value.index,
            resource: Resource::name(value.name),
        })
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
        Instruction::Mint(InstructionResourceMint {
            output_index: value.index,
            resource: Resource::name(value.name),
        })
    }
}
