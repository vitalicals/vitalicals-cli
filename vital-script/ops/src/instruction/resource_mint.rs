//! The Resource Mint instruction

use alloc::vec::Vec;
use anyhow::Result;
use vital_script_primitives::{resources::Resource, traits::*};

use crate::instruction::VitalInstruction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMint {
    pub output_index: u8,
    pub resource: Resource,
}

impl VitalInstruction for InstructionResourceMint {
    fn exec(self, _context: &mut impl Context) -> Result<()> {
        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        todo!()
    }
}
