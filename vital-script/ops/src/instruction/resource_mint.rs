//! The Resource Mint instruction

use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::Resource,
    traits::*,
};

use crate::{
    basic::{BasicOpcode, MintName, MintShortName},
    instruction::VitalInstruction,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMint {
    pub output_index: u8,
    pub resource: Resource,
}

impl InstructionResourceMint {
    pub fn new(index: u8, resource: impl Into<Resource>) -> Self {
        Self { output_index: index, resource: resource.into() }
    }
}

impl VitalInstruction for InstructionResourceMint {
    fn exec(self, context: &mut impl Context) -> Result<()> {
        // TODO:: check if can mint
        context.send_resource_to_output(self.output_index, self.resource)?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        Ok(match self.resource {
            Resource::Name(n) => {
                let l = n.len();
                if l <= SHORT_NAME_LEN_MAX {
                    MintShortName {
                        name: n.try_into().expect("the name should be short"),
                        index: self.output_index,
                    }
                    .encode_op()
                } else if l <= NAME_LEN_MAX {
                    MintName { name: n, index: self.output_index }.encode_op()
                } else {
                    bail!("not support long name")
                }
            }
            _ => bail!("not supported resource"),
        })
    }
}
