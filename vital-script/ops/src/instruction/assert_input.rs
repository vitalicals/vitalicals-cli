//! The input assert instruction

use anyhow::{bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, VRC20, VRC721},
    traits::*,
    U256,
};

use crate::{
    instruction::{utils::*, VitalInstruction},
    opcodes::BasicOp,
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
            Resource::Name(name) => {
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
        let mut res = Vec::with_capacity(512);

        res.push(BasicOp::InputVRC721Assert as u8);
        res.append(&mut v.hash.0.to_vec());
        res.append(&mut v.name.0.to_vec());
        res.push(index);

        Ok(res)
    }
}
