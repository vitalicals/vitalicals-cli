use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::ResourceType,
};

use crate::instruction::Instruction;

/// Mint short name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintShortName {
    pub name: ShortName,
    pub index: u8,
}

impl From<MintShortName> for Instruction {
    fn from(value: MintShortName) -> Self {
        Instruction::mint(value.index, ResourceType::name(value.name))
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
        Instruction::mint(value.index, ResourceType::name(value.name))
    }
}

/// Mint short name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintShortVRC20 {
    pub name: ShortName,
    pub index: u8,
}

impl From<MintShortVRC20> for Instruction {
    fn from(value: MintShortVRC20) -> Self {
        Instruction::mint(value.index, ResourceType::vrc20(value.name))
    }
}

/// Mint name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintVRC20 {
    pub name: Name,
    pub index: u8,
}

impl From<MintVRC20> for Instruction {
    fn from(value: MintVRC20) -> Self {
        Instruction::mint(value.index, ResourceType::vrc20(value.name))
    }
}

/// Mint name
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MintVRC721 {
    pub name: Name,
    pub index: u8,
}

impl From<MintVRC721> for Instruction {
    fn from(value: MintVRC721) -> Self {
        Instruction::mint(value.index, ResourceType::vrc721(value.name))
    }
}
