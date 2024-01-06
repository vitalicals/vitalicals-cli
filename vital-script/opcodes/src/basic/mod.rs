//! The basic opcodes.

mod op_input;
mod op_output;
mod op_transfer;

pub use op_input::*;
pub use op_output::*;
pub use op_transfer::*;

pub trait BasicOpcode {
    fn id(&self) -> u8;
}
