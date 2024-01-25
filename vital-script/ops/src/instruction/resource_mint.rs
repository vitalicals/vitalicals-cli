//! The Resource Mint instruction

use alloc::vec::Vec;
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, ResourceClass, ResourceType, VRC20},
    traits::*,
    U256,
};

use crate::op_basic::{
    BasicOpcode, MintName, MintShortName, MintShortVRC20, MintVRC20, MintVRC721,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMint {
    pub output_index: u8,
    pub resource_type: ResourceType,
}

impl core::fmt::Display for InstructionResourceMint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResourceMint:({}, {})", self.output_index, self.resource_type)
    }
}

impl InstructionResourceMint {
    pub fn new(index: u8, resource_type: ResourceType) -> Self {
        Self { output_index: index, resource_type }
    }

    fn make_mint_resource(&self, context: &mut impl Context) -> Result<Resource> {
        match self.resource_type.class {
            ResourceClass::Name => Ok(Resource::name(self.resource_type.name)),
            ResourceClass::VRC20 => {
                let name = self.resource_type.name;
                let status_data = context
                    .env()
                    .get_vrc20_metadata(name)
                    .context("get vrc20 metadata")?
                    .ok_or_else(|| anyhow!("not found vrc20 metadata, may not deployed"))?;

                // get the mint amount.
                let amount = status_data.meta.mint.mint_amount;

                // check if can mint.
                if status_data.mint_count >= status_data.meta.mint.max_mints {
                    bail!("mint count had reached max");
                }

                Ok(Resource::VRC20(VRC20 { name, amount: U256::from(amount) }))
            }
            ResourceClass::VRC721 => {
                todo!()
            }
        }
    }
}

impl Instruction for InstructionResourceMint {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // TODO:: check if can mint

        // println!("InstructionResourceMint");

        let resource = self.make_mint_resource(context)?;

        match &resource {
            Resource::Name(n) => {
                // for name, we need flag it
                context.env().new_name(*n).context("new name failed")?;
            }
            Resource::VRC20(v) => {
                // for vrc20, we need add mint count
                context
                    .env()
                    .increase_vrc20_mint_count(v.name)
                    .context("increase mint count failed")?;
            }
            Resource::VRC721(_v) => {
                todo!();
            }
        }

        context.send_resource_to_output(self.output_index, resource)?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let bytes = {
            let l = self.resource_type.name.len();
            if l <= SHORT_NAME_LEN_MAX {
                let name = self.resource_type.name.try_into().expect("the name should be short");
                let index = self.output_index;

                match self.resource_type.class {
                    ResourceClass::Name => MintShortName { name, index }.encode_op(),
                    ResourceClass::VRC20 => MintShortVRC20 { name, index }.encode_op(),
                    ResourceClass::VRC721 => {
                        // The VRC721 just support name, TODO: add mint vrc721 short
                        MintVRC721 { name: self.resource_type.name, index }.encode_op()
                    }
                }
            } else if l <= NAME_LEN_MAX {
                let name = self.resource_type.name;
                let index = self.output_index;

                match self.resource_type.class {
                    ResourceClass::Name => MintName { name, index }.encode_op(),
                    ResourceClass::VRC20 => MintVRC20 { name, index }.encode_op(),
                    ResourceClass::VRC721 => {
                        // The VRC721 just support name, TODO: add mint vrc721 short
                        MintVRC721 { name, index }.encode_op()
                    }
                }
            } else {
                bail!("not support long name")
            }
        };
        Ok(bytes)
    }
}
