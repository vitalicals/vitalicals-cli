//! The basic ops.
use anyhow::Result;
use bytes::Bytes;

mod op_dmint;
mod op_input;
mod op_mint;
mod op_output;
mod op_transfer;

pub use op_dmint::*;
pub use op_input::*;
pub use op_mint::*;
pub use op_output::*;
pub use op_transfer::*;

use crate::instruction::Instruction;

pub trait Opcode: Sized {
    const ID: u8;

    fn into_instruction(self) -> Instruction;
}

pub trait BasicOpcodeCodec: Opcode {
    const OPERAND_SIZE: usize;

    fn encode(self) -> Vec<u8>;
    fn decode_operand(datas: &mut Bytes) -> Result<Self>;
}

pub trait BasicOpcode: Opcode + BasicOpcodeCodec {}

impl<T> BasicOpcode for T where T: BasicOpcodeCodec {}
