//! Some utils for into opcode.
//!

use alloc::vec::Vec;
use anyhow::Result;
use vital_script_primitives::{
    names::{ShortName, NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::VRC20,
    U256,
};

use crate::opcodes::BasicOp;

pub struct Vrc20ResourceOperand {
    name_bytes: Vec<u8>,
    name_typ_idx: u8,
    amount_bytes: Vec<u8>,
    amount_typ_idx: u8,
}

impl Vrc20ResourceOperand {
    pub fn new(res: VRC20) -> Self {
        let name_len = res.name.len();
        let (name_bytes, name_typ_idx) = if name_len <= SHORT_NAME_LEN_MAX {
            (ShortName::try_from(res.name).expect("should ok").0.to_vec(), 0)
        } else if name_len <= NAME_LEN_MAX {
            (res.name.0.to_vec(), 1)
        } else {
            todo!("not support long name")
        };

        let (amount_bytes, amount_typ_idx) = if res.amount <= U256::from(u32::MAX) {
            (res.amount.as_u32().to_le_bytes().to_vec(), 0)
        } else if res.amount <= U256::from(u64::MAX) {
            (res.amount.as_u64().to_le_bytes().to_vec(), 1)
        } else if res.amount <= U256::from(u128::MAX) {
            (res.amount.as_u128().to_le_bytes().to_vec(), 2)
        } else {
            let mut raw = [0_u8; 32];
            res.amount.to_little_endian(&mut raw);

            (raw.to_vec(), 3)
        };

        Self { name_bytes, name_typ_idx: name_typ_idx * 4, amount_bytes, amount_typ_idx }
    }

    pub fn into_input_vrc20_opcode_bytes(mut self, index: u8) -> Result<Vec<u8>> {
        let opcode_header = BasicOp::InputVRC20AssertSa32 as u8;
        let opcode = opcode_header + self.name_typ_idx + self.amount_typ_idx;

        let opcode = BasicOp::new(opcode)?;

        let mut bytes = Vec::with_capacity(1 + 1 + self.name_bytes.len() + self.amount_bytes.len());
        bytes.push(opcode as u8);
        bytes.append(&mut self.amount_bytes);
        bytes.append(&mut self.name_bytes);
        bytes.push(index);

        Ok(bytes)
    }

    pub fn into_move_vrc20_opcode_bytes(mut self, index: u8) -> Result<Vec<u8>> {
        let opcode_header = BasicOp::MoveVRC20Sa32 as u8;
        let opcode = opcode_header + self.name_typ_idx + self.amount_typ_idx;

        let opcode = BasicOp::new(opcode)?;

        let mut bytes = Vec::with_capacity(1 + 1 + self.name_bytes.len() + self.amount_bytes.len());
        bytes.push(opcode as u8);
        bytes.append(&mut self.name_bytes);
        bytes.append(&mut self.amount_bytes);
        bytes.push(index);

        Ok(bytes)
    }
}
