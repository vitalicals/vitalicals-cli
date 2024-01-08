//! The basic ops.

mod op_input;
mod op_output;
mod op_transfer;

pub use op_input::*;
pub use op_output::*;
pub use op_transfer::*;

use vital_script_instruction::Instruction;

pub trait BasicOpcode {
    fn id(&self) -> u8;

    fn into_instruction(self) -> Instruction;
}
