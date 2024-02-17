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

#[allow(unused_imports)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use vital_script_primitives::traits::Instruction as InstructionT;

    pub fn check_ops_encode_and_decode<T: BasicOpcode>(op: T)
    where
        Instruction: From<T>,
    {
        let op_encoded = op.encode_op();

        let i = Instruction::from(op);
        let bytes = i.into_ops_bytes().expect("the into should ok");

        assert_eq!(op_encoded, bytes, "the encode by ins and ops need eq");
        assert_eq!(bytes[0], T::ID, "the id should be eq");

        let mut bytes = Bytes::from(bytes[1..].to_vec());
        let ops_decode = T::decode_operand(&mut bytes).expect("should ok");

        assert_eq!(op_encoded[1..].to_vec(), ops_decode.encode(), "op should be eq");
    }
}
