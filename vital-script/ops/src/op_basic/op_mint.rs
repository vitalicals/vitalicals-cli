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

#[cfg(test)]
mod tests {
    use vital_script_primitives::names::Name;

    use super::*;
    use crate::op_basic::tests::check_ops_encode_and_decode;

    #[test]
    fn test_mint_ops_encode_and_decode() {
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(MintShortName { name: short_name, index: 128 });

        check_ops_encode_and_decode(MintName { name, index: 128 });

        check_ops_encode_and_decode(MintShortVRC20 { name: short_name, index: 128 });

        check_ops_encode_and_decode(MintVRC20 { name, index: 128 });

        check_ops_encode_and_decode(MintVRC721 { name, index: 128 });
    }
}
