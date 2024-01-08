use vital_script_instruction::Instruction;
use vital_script_primitives::names::{Name, ShortName};

use crate::opcodes::BasicOp;

use super::BasicOpcode;

/// Transfer all VRC20 Res to a output for (ShortName)
pub struct TransferAllVRC20S {
    pub name: ShortName,
    pub output_index: u8,
}

impl BasicOpcode for TransferAllVRC20S {
    fn id(&self) -> u8 {
        BasicOp::TransferAllVRC20S as u8
    }

    fn into_instruction(self) -> Instruction {
        todo!()
    }
}

/// Transfer all VRC20 Res to a output for (Name)
pub struct TransferAllVRC20 {
    pub name: Name,
    pub output_index: u8,
}

impl BasicOpcode for TransferAllVRC20 {
    fn id(&self) -> u8 {
        BasicOp::TransferAllVRC20 as u8
    }

    fn into_instruction(self) -> Instruction {
        todo!()
    }
}

/// Transfer VRC20 Res with a amount to a output for (ShortName, u32)
pub struct TransferVRC20Sa32 {
    pub name: ShortName,
    pub amount: u32,
    pub output_index: u8,
}

impl BasicOpcode for TransferVRC20Sa32 {
    fn id(&self) -> u8 {
        BasicOp::TransferVRC20Sa32 as u8
    }

    fn into_instruction(self) -> Instruction {
        todo!()
    }
}

/// Transfer VRC20 Res with a amount to a output for (Name, u32)
pub struct TransferVRC20A32 {
    pub name: Name,
    pub amount: u32,
    pub output_index: u8,
}

impl BasicOpcode for TransferVRC20A32 {
    fn id(&self) -> u8 {
        BasicOp::TransferVRC20A32 as u8
    }

    fn into_instruction(self) -> Instruction {
        todo!()
    }
}

// TODO: more
