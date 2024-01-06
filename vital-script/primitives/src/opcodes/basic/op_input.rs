use primitive_types::{H256, U256};

use super::BasicOpcode;
use crate::names::{Name, ShortName};

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputShortVRC20Assert32 {
    pub amount: u32,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputShortVRC20Assert32 {
    fn id(&self) -> u8 {
        0x0d
    }
}

/// Input VRC20 Res Assert for (ShortName, u64 amount)
pub struct InputShortVRC20Assert64 {
    pub amount: u64,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputShortVRC20Assert64 {
    fn id(&self) -> u8 {
        0x0e
    }
}

/// Input VRC20 Res Assert for (ShortName, u128 amount)
pub struct InputShortVRC20Assert128 {
    pub amount: u128,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputShortVRC20Assert128 {
    fn id(&self) -> u8 {
        0x0f
    }
}

/// Input VRC20 Res Assert for (ShortName, u256 amount)
pub struct InputShortVRC20Assert256 {
    pub amount: U256,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputShortVRC20Assert256 {
    fn id(&self) -> u8 {
        0x10
    }
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20Assert32 {
    pub amount: u32,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20Assert32 {
    fn id(&self) -> u8 {
        0x11
    }
}

/// Input VRC20 Res Assert for (Name, u64 amount)
pub struct InputVRC20Assert64 {
    pub amount: u64,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20Assert64 {
    fn id(&self) -> u8 {
        0x12
    }
}

/// Input VRC20 Res Assert for (Name, u128 amount)
pub struct InputVRC20Assert128 {
    pub amount: u128,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20Assert128 {
    fn id(&self) -> u8 {
        0x13
    }
}

/// Input VRC20 Res Assert for (Name, u256 amount)
pub struct InputVRC20Assert256 {
    pub amount: U256,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20Assert256 {
    fn id(&self) -> u8 {
        0x14
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
        0x15
    }
}
