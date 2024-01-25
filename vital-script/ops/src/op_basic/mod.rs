//! The basic ops.

use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use parity_scale_codec::Encode;

mod op_dmint;
mod op_input;
mod op_mint;
mod op_move;
mod op_output;

pub use op_dmint::*;
pub use op_input::*;
pub use op_mint::*;
pub use op_move::*;
pub use op_output::*;

use crate::{instruction::Instruction, utils::Reader};

pub trait BasicOpcodeBase: Sized + Into<Instruction> {
    const ID: u8;
}

pub trait BasicOpcode: BasicOpcodeBase + parity_scale_codec::Codec {
    fn encode_op(&self) -> Vec<u8> {
        (Self::ID, self).encode()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        Self::decode(&mut Reader::new(datas)).map_err(|err| anyhow!("decode_operand {}", err))
    }
}

impl<T> BasicOpcode for T where T: BasicOpcodeBase + parity_scale_codec::Codec {}
