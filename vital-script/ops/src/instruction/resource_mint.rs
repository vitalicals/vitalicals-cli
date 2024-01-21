//! The Resource Mint instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, ResourceClass, ResourceType, Tag},
    traits::*,
};

use crate::{
    instruction::VitalInstruction,
    op_basic::{BasicOpcode, MintName, MintShortName},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMint {
    pub output_index: u8,
    pub resource_type: ResourceType,
}

impl InstructionResourceMint {
    pub fn new(index: u8, resource_type: ResourceType) -> Self {
        Self { output_index: index, resource_type }
    }

    fn make_mint_resource(&self, context: &mut impl Context) -> Result<Resource> {
        match self.resource_type.class {
            ResourceClass::Name => Ok(Resource::name(self.resource_type.name)),
            ResourceClass::VRC20 => {
                todo!()
            }
            ResourceClass::VRC721 => {
                todo!()
            }
        }
    }
}

impl VitalInstruction for InstructionResourceMint {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // TODO:: check if can mint

        let resource = self.make_mint_resource(context)?;

        // for name, we need flag it
        if let Resource::Name(n) = resource {
            context.env().new_name(n).context("new name failed")?;
        }

        context.send_resource_to_output(self.output_index, resource.clone())?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let bytes = {
            let l = self.resource_type.name.len();
            if l <= SHORT_NAME_LEN_MAX {
                match self.resource_type.class {
                    ResourceClass::Name => MintShortName {
                        name: self.resource_type.name.try_into().expect("the name should be short"),
                        index: self.output_index,
                    }
                    .encode_op(),
                    ResourceClass::VRC20 => {
                        todo!()
                    }
                    ResourceClass::VRC721 => {
                        todo!()
                    }
                }
            } else if l <= NAME_LEN_MAX {
                match self.resource_type.class {
                    ResourceClass::Name => {
                        MintName { name: self.resource_type.name, index: self.output_index }
                            .encode_op()
                    }
                    ResourceClass::VRC20 => {
                        todo!()
                    }
                    ResourceClass::VRC721 => {
                        todo!()
                    }
                }
            } else {
                bail!("not support long name")
            }
        };
        Ok(bytes)
    }
}
