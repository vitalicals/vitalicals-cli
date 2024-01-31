//! The input assert instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use parity_scale_codec::Encode;
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, Tag, VRC20, VRC721},
    traits::*,
};

use crate::{
    instruction::utils::*,
    op_basic::{BasicOpcodeBase, InputAssertName, InputAssertShortName, InputVRC721Assert},
    TARGET,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionInputAssert {
    pub index: u8,
    pub resource: Resource,
}

impl core::fmt::Display for InstructionInputAssert {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InputAssert:({}, {})", self.index, self.resource)
    }
}

impl Instruction for InstructionInputAssert {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // 1. ensure if current input index is not asserted.
        context.runner().try_assert_input(self.index)?;

        // 2. ensure the resource is expected by index.
        if context.run_mod().is_skip_check() {
            log::debug!(target: TARGET, "skip input resource check by in sim mode" )
        } else {
            let resource_from_env =
                context.env().get_input_resource(self.index).context("get input resource")?;

            if resource_from_env != self.resource {
                log::debug!(target: TARGET, "resource from {:?} expect {:?}", resource_from_env, self.resource);
                bail!("the resource not expected")
            }
        }

        // 3. push the resource into resources.
        context
            .input_resource()
            .push(self.index, self.resource.clone())
            .context("push input resource")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self.resource {
            Resource::Name(name) => Self::into_input_name(name, self.index),
            Resource::VRC20(vrc20) => Self::into_input_vrc20(vrc20, self.index),
            Resource::VRC721(vrc721) => Self::into_input_vrc721(vrc721, self.index),
        }
    }
}

impl InstructionInputAssert {
    fn into_input_name(n: Tag, index: u8) -> Result<Vec<u8>> {
        // TODO: use a common way to process different name
        let name_len = n.len();
        let (opcode, mut res) = if name_len <= SHORT_NAME_LEN_MAX {
            (
                <InputAssertShortName as BasicOpcodeBase>::ID,
                InputAssertShortName { name: n.try_into().expect("should ok"), index }.encode(),
            )
        } else if name_len <= NAME_LEN_MAX {
            (<InputAssertName as BasicOpcodeBase>::ID, InputAssertName { name: n, index }.encode())
        } else {
            bail!("not support long name")
        };

        let mut bytes = Vec::with_capacity(4 + res.len());
        bytes.push(opcode);
        bytes.append(&mut res);

        Ok(bytes)
    }

    fn into_input_vrc20(v: VRC20, index: u8) -> Result<Vec<u8>> {
        Vrc20ResourceOperand::new(v).into_input_vrc20_opcode_bytes(index)
    }

    fn into_input_vrc721(v: VRC721, index: u8) -> Result<Vec<u8>> {
        let op = InputVRC721Assert { hash: v.hash, name: v.name, index };

        let mut bytes = Vec::with_capacity(512);
        bytes.push(<InputVRC721Assert as BasicOpcodeBase>::ID);
        bytes.append(&mut op.encode());

        Ok(bytes)
    }
}
