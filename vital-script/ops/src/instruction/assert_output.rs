//! The output assert instruction

use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::traits::*;

use crate::{
    basic::{BasicOpcode, OutputIndexAssert, OutputIndexFlag16Assert, OutputIndexFlag32Assert},
    instruction::VitalInstruction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionOutputAssert {
    pub indexs: Vec<u8>,
}

impl VitalInstruction for InstructionOutputAssert {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        for index in self.indexs.iter() {
            // 1. ensure if current output index is not asserted.
            context.runner().try_assert_output(*index)?;
        }

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        if self.indexs.is_empty() {
            bail!("no index for output")
        }

        if self.indexs.len() == 1 {
            let op = OutputIndexAssert { index: self.indexs[0] };

            return Ok(op.encode_op());
        }

        // check if can use flags16
        let is_all_less_than_16 = self.indexs.iter().all(|a| *a < 16_u8);
        if is_all_less_than_16 {
            let mut mask = 0_u16;
            for i in self.indexs {
                let m = 1_u16 << i;
                mask |= m;
            }

            let op = OutputIndexFlag16Assert { index_flag: mask.to_le_bytes() };

            return Ok(op.encode_op());
        }

        let is_all_less_than_32 = self.indexs.iter().all(|a| *a < 32_u8);
        if is_all_less_than_32 {
            let mut mask = 0_u32;
            for i in self.indexs {
                let m = 1_u32 << i;
                mask |= m;
            }

            let op = OutputIndexFlag32Assert { index_flag: mask.to_le_bytes() };

            return Ok(op.encode_op());
        }

        todo!("not support output index >= 32")
    }
}
