//! The basic ops.
use anyhow::Result;
use bytes::Bytes;

mod op_input;
mod op_output;
mod op_transfer;

pub use op_input::*;
pub use op_output::*;
pub use op_transfer::*;

use vital_script_instruction::Instruction;

pub trait BasicOpcode: Sized {
    const OPERAND_SIZE: usize;
    const ID: u8;

    fn into_instruction(self) -> Instruction;

    fn decode_operand(datas: &mut Bytes) -> Result<Self>;
}
