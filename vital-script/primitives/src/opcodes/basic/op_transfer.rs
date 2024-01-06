use super::BasicOpcode;
use crate::names::{Name, ShortName};

/// Transfer all VRC20 Res to a output for (ShortName)
pub struct TransferAllShortVRC20 {
    pub name: ShortName,
    pub output_index: u8,
}

impl BasicOpcode for TransferAllShortVRC20 {
    fn id(&self) -> u8 {
        0x16
    }
}

/// Transfer all VRC20 Res to a output for (Name)
pub struct TransferAllVRC20 {
    pub name: Name,
    pub output_index: u8,
}

impl BasicOpcode for TransferAllVRC20 {
    fn id(&self) -> u8 {
        0x17
    }
}

/// Transfer VRC20 Res with a amount to a output for (ShortName, u32)
pub struct Transfer32ShortVRC20 {
    pub name: ShortName,
    pub amount: u32,
    pub output_index: u8,
}

impl BasicOpcode for Transfer32ShortVRC20 {
    fn id(&self) -> u8 {
        0x18
    }
}

/// Transfer VRC20 Res with a amount to a output for (Name, u32)
pub struct Transfer32VRC20 {
    pub name: Name,
    pub amount: u32,
    pub output_index: u8,
}

impl BasicOpcode for Transfer32VRC20 {
    fn id(&self) -> u8 {
        0x19
    }
}

// TODO: more
