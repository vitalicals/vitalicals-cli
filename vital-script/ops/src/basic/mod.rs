//! The basic ops.
use std::io::{Read, Write};

use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use bytes::{buf::Writer, Buf, BufMut, Bytes};

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
use parity_scale_codec::Encode;

use crate::instruction::Instruction;

const CAP_SIZE: usize = 1024;

pub trait Opcode: Sized + Into<Instruction> {
    const ID: u8;
}

pub trait BasicOpcode:
    Opcode + serde::de::DeserializeOwned + serde::Serialize + parity_scale_codec::Codec
{
    fn encode_op(&self) -> Vec<u8> {
        (Self::ID, self).encode()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        Self::decode(&mut Reader::new(datas))
            .map_err(|err| anyhow!("decode_operand {}", err.to_string()))
    }
}

impl<T> BasicOpcode for T where
    T: Opcode + serde::de::DeserializeOwned + serde::Serialize + parity_scale_codec::Codec
{
}

struct Reader<'a> {
    datas: &'a mut Bytes,
}

impl<'a> Reader<'a> {
    fn new(datas: &'a mut Bytes) -> Self {
        Self { datas }
    }
}

impl<'a> parity_scale_codec::Input for Reader<'a> {
    fn remaining_len(
        &mut self,
    ) -> core::prelude::v1::Result<Option<usize>, parity_scale_codec::Error> {
        Ok(Some(self.datas.remaining()))
    }

    fn read(
        &mut self,
        into: &mut [u8],
    ) -> core::prelude::v1::Result<(), parity_scale_codec::Error> {
        self.datas
            .reader()
            .read(into)
            .map_err(|err| parity_scale_codec::Error::from("io"))?;
        Ok(())
    }
}
