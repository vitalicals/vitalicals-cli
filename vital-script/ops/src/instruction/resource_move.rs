//! The Resource Move instruction

use alloc::vec::Vec;
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, ResourceType},
    traits::*,
};

use crate::{
    instruction::utils::Vrc20ResourceOperand,
    op_basic::{BasicOpcode, MoveAllVRC20S, MoveName, MoveShortName, MoveVRC721},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMove {
    pub output_index: u8,
    pub resource: Resource,
}

impl InstructionResourceMove {
    pub fn new(index: u8, resource: impl Into<Resource>) -> Self {
        Self { output_index: index, resource: resource.into() }
    }
}

impl core::fmt::Display for InstructionResourceMove {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResourceMove:({}, {})", self.output_index, self.resource)
    }
}

impl Instruction for InstructionResourceMove {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        context.input_resource().cost(&self.resource).context("cost resource failed")?;

        context
            .send_resource_to_output(self.output_index, self.resource.clone())
            .context("send to output failed")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let raw = match self.resource {
            Resource::Name(name) => match name.len() {
                n if n <= SHORT_NAME_LEN_MAX => MoveShortName {
                    name: name.try_into().context("the name is not short")?,
                    output_index: self.output_index,
                }
                .encode_op(),
                n if n <= NAME_LEN_MAX => {
                    MoveName { name, output_index: self.output_index }.encode_op()
                }
                _ => {
                    bail!("not support long name")
                }
            },
            Resource::VRC20(vrc20) => Vrc20ResourceOperand::new(vrc20)
                .into_move_vrc20_opcode_bytes(self.output_index)
                .context("use Vrc20ResourceOperand into opcode bytes")?,
            Resource::VRC721(vrc721) => {
                MoveVRC721 { name: vrc721.name, hash: vrc721.hash, output_index: self.output_index }
                    .encode_op()
            }
        };

        Ok(raw)
    }
}

impl InstructionResourceMove {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMoveAll {
    pub output_index: u8,
    pub resource_type: ResourceType,
}

impl InstructionResourceMoveAll {
    pub fn new(index: u8, resource_type: ResourceType) -> Self {
        Self { output_index: index, resource_type }
    }
}

impl core::fmt::Display for InstructionResourceMoveAll {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResourceMoveAll:({}, {})", self.output_index, self.resource_type)
    }
}

impl Instruction for InstructionResourceMoveAll {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        let resource = context
            .input_resource()
            .get_vrc20(self.resource_type.name)
            .ok_or_else(|| anyhow!("not found vrc20 resource by name"))?;

        context.input_resource().cost(&resource).context("cost resource failed")?;

        context
            .send_resource_to_output(self.output_index, resource)
            .context("send to output failed")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        if !self.resource_type.is_vrc20() {
            bail!("only vrc20 resource type support move all");
        }

        let raw = match self.resource_type.name.len() {
            n if n <= SHORT_NAME_LEN_MAX => MoveAllVRC20S {
                name: self.resource_type.name.try_into().context("the name is not short")?,
                output_index: self.output_index,
            }
            .encode_op(),
            n if n <= NAME_LEN_MAX => {
                MoveName { name: self.resource_type.name, output_index: self.output_index }
                    .encode_op()
            }
            _ => {
                bail!("not support long name")
            }
        };

        Ok(raw)
    }
}

impl InstructionResourceMoveAll {}
