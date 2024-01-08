use vital_script_instruction::{assert_input::InstructionInputAssert, Instruction};
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, VRC20, VRC721},
    H256, U256,
};

use crate::opcodes::BasicOp;

use super::BasicOpcode;

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20AssertSa32 {
    pub amount: u32,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa32 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertSa32 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (ShortName, u64 amount)
pub struct InputVRC20AssertSa64 {
    pub amount: u64,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa64 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertSa64 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (ShortName, u128 amount)
pub struct InputVRC20AssertSa128 {
    pub amount: u128,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa128 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertSa128 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (ShortName, u256 amount)
pub struct InputVRC20AssertSa256 {
    pub amount: U256,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa256 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertSa256 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount)),
        })
    }
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20AssertA32 {
    pub amount: u32,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA32 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertA32 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (Name, u64 amount)
pub struct InputVRC20AssertA64 {
    pub amount: u64,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA64 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertA64 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (Name, u128 amount)
pub struct InputVRC20AssertA128 {
    pub amount: u128,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA128 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertA128 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }
}

/// Input VRC20 Res Assert for (Name, u256 amount)
pub struct InputVRC20AssertA256 {
    pub amount: U256,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA256 {
    fn id(&self) -> u8 {
        BasicOp::InputVRC20AssertA256 as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }
}

/// Input VRC721 Res Assert for (Name, hash256 )
pub struct InputVRC721Assert {
    pub hash: H256,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC721Assert {
    fn id(&self) -> u8 {
        BasicOp::InputVRC721Assert as u8
    }

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC721(VRC721::new(self.name, self.hash)),
        })
    }
}
