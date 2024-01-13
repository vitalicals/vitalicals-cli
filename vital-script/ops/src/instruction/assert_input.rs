//! The input assert instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use parity_scale_codec::Encode;
use vital_script_primitives::{
    resources::{Resource, VRC20, VRC721},
    traits::*,
};

use crate::{
    basic::InputVRC721Assert,
    instruction::{utils::*, VitalInstruction},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionInputAssert {
    pub index: u8,
    pub resource: Resource,
}

impl VitalInstruction for InstructionInputAssert {
    fn exec(self, context: &mut impl Context) -> Result<()> {
        // 1. ensure if current input index is not asserted.
        context.runner().try_assert_input(self.index)?;

        // 2. ensure the resource is expected by index.
        let resource_from_env =
            context.env().get_input_resource(self.index).context("get input resource")?;
        if resource_from_env != self.resource {
            bail!("the resource not expected")
        }

        // 3. push the resource into resources.
        context
            .input_resource()
            .push(self.index, self.resource)
            .context("push input resource")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self.resource {
            Resource::Name(_name) => {
                todo!("add input name assert")
            }
            Resource::VRC20(vrc20) => Self::into_input_vrc20(vrc20, self.index),
            Resource::VRC721(vrc721) => Self::into_input_vrc721(vrc721, self.index),
        }
    }
}

impl InstructionInputAssert {
    fn into_input_vrc20(v: VRC20, index: u8) -> Result<Vec<u8>> {
        Vrc20ResourceOperand::new(v).to_input_vrc20_opcode_bytes(index)
    }

    fn into_input_vrc721(v: VRC721, index: u8) -> Result<Vec<u8>> {
        let op = InputVRC721Assert { hash: v.hash, name: v.name, index };

        Ok(op.encode())
    }
}
