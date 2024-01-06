//! The basic opcodes.

pub trait BasicOpcode {
    fn id(&self) -> u8;
}

/// Output Index Assert by 1 indexs
pub struct OutputIndexAssert {
    index: u8,
}

impl BasicOpcode for OutputIndexAssert {
    fn id(&self) -> u8 {
        0x0a
    }
}

/// Output Index Assert By A u16 as FlagMask
pub struct OutputIndexFlag16Assert {
    index_flag: u16,
}

impl BasicOpcode for OutputIndexFlag16Assert {
    fn id(&self) -> u8 {
        0x0b
    }
}

/// Output Index Assert By A u32 as FlagMask
pub struct OutputIndexFlag32Assert {
    index_flag: u32,
}

impl BasicOpcode for OutputIndexFlag32Assert {
    fn id(&self) -> u8 {
        0x0c
    }
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20Assert {
    amount: u32,
}

impl BasicOpcode for InputVRC20Assert {
    fn id(&self) -> u8 {
        0x0d
    }
}
