use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, ResourceType, VRC721},
    H256, U256,
};

use crate::instruction::{
    resource_move::{InstructionResourceMove, InstructionResourceMoveAll},
    Instruction,
};

/// Move short name to a output for (ShortName)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveShortName {
    pub name: ShortName,
    pub output_index: u8,
}

impl From<MoveShortName> for Instruction {
    fn from(value: MoveShortName) -> Self {
        Instruction::move_to(value.output_index, value.name)
    }
}

/// Move name to a output for (ShortName)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveName {
    pub name: Name,
    pub output_index: u8,
}

impl From<MoveName> for Instruction {
    fn from(value: MoveName) -> Self {
        Instruction::move_to(value.output_index, value.name)
    }
}

// TODO: Move LongName.

/// Move all VRC20 Res to a output for (ShortName)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveAllVRC20S {
    pub name: ShortName,
    pub output_index: u8,
}

impl From<MoveAllVRC20S> for Instruction {
    fn from(value: MoveAllVRC20S) -> Self {
        Instruction::MoveAll(InstructionResourceMoveAll::new(
            value.output_index,
            ResourceType::vrc20(value.name),
        ))
    }
}

/// Move all VRC20 Res to a output for (Name)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveAllVRC20 {
    pub name: Name,
    pub output_index: u8,
}

impl From<MoveAllVRC20> for Instruction {
    fn from(value: MoveAllVRC20) -> Self {
        Instruction::MoveAll(InstructionResourceMoveAll::new(
            value.output_index,
            ResourceType::vrc20(value.name),
        ))
    }
}

/// Move VRC20 Res with a amount to a output for (ShortName, u32)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20Sa32 {
    pub name: ShortName,
    pub amount: u32,
    pub output_index: u8,
}

impl From<MoveVRC20Sa32> for Instruction {
    fn from(value: MoveVRC20Sa32) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (ShortName, u64)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20Sa64 {
    pub name: ShortName,
    pub amount: u64,
    pub output_index: u8,
}

impl From<MoveVRC20Sa64> for Instruction {
    fn from(value: MoveVRC20Sa64) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (ShortName, u128)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20Sa128 {
    pub name: ShortName,
    pub amount: u128,
    pub output_index: u8,
}

impl From<MoveVRC20Sa128> for Instruction {
    fn from(value: MoveVRC20Sa128) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (ShortName, u256)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20Sa256 {
    pub name: ShortName,
    pub amount: U256,
    pub output_index: u8,
}

impl From<MoveVRC20Sa256> for Instruction {
    fn from(value: MoveVRC20Sa256) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (Name, u32)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20A32 {
    pub name: Name,
    pub amount: u32,
    pub output_index: u8,
}

impl From<MoveVRC20A32> for Instruction {
    fn from(value: MoveVRC20A32) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (Name, u64)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20A64 {
    pub name: Name,
    pub amount: u64,
    pub output_index: u8,
}

impl From<MoveVRC20A64> for Instruction {
    fn from(value: MoveVRC20A64) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (Name, u128)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20A128 {
    pub name: Name,
    pub amount: u128,
    pub output_index: u8,
}

impl From<MoveVRC20A128> for Instruction {
    fn from(value: MoveVRC20A128) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC20 Res with a amount to a output for (Name, U256)
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC20A256 {
    pub name: Name,
    pub amount: U256,
    pub output_index: u8,
}

impl From<MoveVRC20A256> for Instruction {
    fn from(value: MoveVRC20A256) -> Self {
        Instruction::move_vrc20_to(value.output_index, value.name, value.amount)
    }
}

/// Move VRC721
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct MoveVRC721 {
    pub name: Name,
    pub hash: H256,
    pub output_index: u8,
}

impl From<MoveVRC721> for Instruction {
    fn from(value: MoveVRC721) -> Self {
        Instruction::Move(InstructionResourceMove::new(
            value.output_index,
            Resource::VRC721(VRC721::new(value.name, value.hash)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use vital_script_primitives::names::Name;

    use super::*;
    use crate::op_basic::tests::check_ops_encode_and_decode;

    #[test]
    fn test_move_name_ops_encode_and_decode() {
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(MoveShortName { name: short_name, output_index: 128 });

        check_ops_encode_and_decode(MoveName { name, output_index: 128 });
    }

    #[test]
    fn test_move_all_vrc20_ops_encode_and_decode() {
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(MoveAllVRC20S { name: short_name, output_index: 128 });

        check_ops_encode_and_decode(MoveAllVRC20 { name, output_index: 128 });
    }

    #[test]
    fn test_move_vrc20_ops_encode_and_decode() {
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(MoveVRC20Sa32 {
            amount: u32::MAX / 2 + 999,
            name: short_name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20Sa64 {
            amount: u64::MAX / 2 + 999,
            name: short_name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20Sa128 {
            amount: u128::MAX / 2 + 999,
            name: short_name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20Sa256 {
            amount: U256::from(u128::MAX) + U256::from(999),
            name: short_name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20A32 {
            amount: u32::MAX / 2 + 999,
            name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20A64 {
            amount: u64::MAX / 2 + 999,
            name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20A128 {
            amount: u128::MAX / 2 + 999,
            name,
            output_index: 3,
        });

        check_ops_encode_and_decode(MoveVRC20A256 {
            amount: U256::from(u128::MAX) + U256::from(999),
            name,
            output_index: 3,
        });
    }

    #[test]
    fn test_move_vrc721_ops_encode_and_decode() {
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(MoveVRC721 { hash: H256::random(), name, output_index: 3 });
    }
}
