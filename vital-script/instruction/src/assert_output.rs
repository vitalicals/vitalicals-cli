//! The output assert instruction

use alloc::vec::Vec;
use anyhow::Result;
use vital_script_primitives::traits::*;

use crate::VitalInstruction;

pub struct InstructionOutputAssert {
    pub indexs: Vec<u8>,
}

impl VitalInstruction for InstructionOutputAssert {
    fn exec(self, context: &mut impl Context) -> Result<()> {
        for index in self.indexs.into_iter() {
            // 1. ensure if current output index is not asserted.
            context.runner().try_assert_output(index)?;
        }

        Ok(())
    }
}
