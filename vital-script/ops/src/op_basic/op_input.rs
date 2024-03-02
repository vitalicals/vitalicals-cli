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
    pub name: Tag, // TODO: add long name
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

#[cfg(test)]
mod tests {
    use vital_script_primitives::names::Name;

    use super::*;
    use crate::{
        builder::instruction::ScriptBuilderFromInstructions,
        op_basic::tests::check_ops_encode_and_decode, parser::Parser,
    };

    #[test]
    fn test_input_ops_encode_and_decode() {
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        check_ops_encode_and_decode(InputAssertShortName { name: short_name, index: 1 });

        check_ops_encode_and_decode(InputAssertName { name, index: 2 });

        // TODO: support long name
        // check_ops_encode_and_decode(InputAssertLongName{
        //     name: name.into(),
        //    index: 3,
        // });

        check_ops_encode_and_decode(InputVRC20AssertSa32 {
            amount: u32::MAX / 2 + 999,
            name: short_name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertSa64 {
            amount: u64::MAX / 2 + 999,
            name: short_name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertSa128 {
            amount: u128::MAX / 2 + 999,
            name: short_name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertSa256 {
            amount: U256::from(u128::MAX) + U256::from(999),
            name: short_name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertA32 {
            amount: u32::MAX / 2 + 999,
            name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertA64 {
            amount: u64::MAX / 2 + 999,
            name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertA128 {
            amount: u128::MAX / 2 + 999,
            name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC20AssertA256 {
            amount: U256::from(u128::MAX) + U256::from(999),
            name,
            index: 3,
        });

        check_ops_encode_and_decode(InputVRC721Assert { hash: H256::random(), index: 128 });
    }

    #[test]
    fn test_ops_to_instruction() {
        let hash = H256::random();
        let short_name = ShortName::try_from("abc".to_string()).unwrap();
        let name = Name::try_from("abcdef".to_string()).unwrap();

        assert_eq!(
            Instruction::from(InputAssertShortName { name: short_name, index: 1 }),
            Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: Resource::Name(Name::from(short_name))
            })
        );

        assert_eq!(
            Instruction::from(InputAssertName { name, index: 1 }),
            Instruction::Input(InstructionInputAssert { index: 1, resource: Resource::Name(name) })
        );

        assert_eq!(
            Instruction::from(InputVRC721Assert { hash, index: 1 }),
            Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: Resource::VRC721(VRC721 { hash })
            })
        )
    }

    #[test]
    fn test_to_instruction() {
        let instructions = vec![
            Instruction::Input(InstructionInputAssert {
                index: 0,
                resource: Resource::name(Name::try_from("dd".to_string()).unwrap()),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: Resource::name(Name::try_from("dddddd".to_string()).unwrap()),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 2,
                resource: Resource::vrc20("abc", 1000.into()).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 3,
                resource: Resource::vrc20("abc", U256::from(u32::MAX) + 1).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 4,
                resource: Resource::vrc20("abc", U256::from(u64::MAX) + 1).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 5,
                resource: Resource::vrc20("abc", U256::from(u128::MAX) + U256::from(1)).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 6,
                resource: Resource::vrc20("abcdefg", 1000.into()).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 7,
                resource: Resource::vrc20("abcdefg", U256::from(u32::MAX) + 1).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 8,
                resource: Resource::vrc20("abcdefg", U256::from(u64::MAX) + 1).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 9,
                resource: Resource::vrc20("abcdefg", U256::from(u128::MAX) + 1).unwrap(),
            }),
            Instruction::Input(InstructionInputAssert {
                index: 10,
                resource: Resource::VRC721(VRC721 { hash: H256::random() }),
            }),
        ];

        let ops_bytes = ScriptBuilderFromInstructions::build(instructions.clone()).unwrap();
        let instructions_into = Parser::new(&ops_bytes).parse().expect("parse");

        assert_eq!(instructions, instructions_into);
    }
}
