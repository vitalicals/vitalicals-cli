use crate::opcode::BasicOp;

use super::BasicOpcode;

/// Output Index Assert by 1 indexs
pub struct OutputIndexAssert {
    pub index: u8,
}

impl BasicOpcode for OutputIndexAssert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexAssert as u8
    }
}

/// Output Index Assert By A u16 as FlagMask
pub struct OutputIndexFlag16Assert {
    pub index_flag: [u8; 2],
}

impl BasicOpcode for OutputIndexFlag16Assert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexFlag16Assert as u8
    }
}

/// Output Index Assert By A u32 as FlagMask
pub struct OutputIndexFlag32Assert {
    pub index_flag: [u8; 4],
}

impl BasicOpcode for OutputIndexFlag32Assert {
    fn id(&self) -> u8 {
        BasicOp::OutputIndexFlag32Assert as u8
    }
}
