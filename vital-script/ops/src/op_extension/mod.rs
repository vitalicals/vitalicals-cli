//! The extension opcode.

use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use bytes::Bytes;
use parity_scale_codec::Encode;

mod op_deploy;

pub use op_deploy::*;

use crate::{instruction::Instruction, utils::Reader};

pub trait ExtensionOpcodeBase: Sized + Into<Instruction> {
    const ID: u16;
}

pub trait ExtensionOpcode: ExtensionOpcodeBase + parity_scale_codec::Codec {
    fn encode_op(&self) -> Vec<u8> {
        (Self::ID, self).encode()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        Self::decode(&mut Reader::new(datas))
            .map_err(|err| anyhow!("decode_operand {}", err.to_string()))
    }
}

impl<T> ExtensionOpcode for T where T: ExtensionOpcodeBase + parity_scale_codec::Codec {}
