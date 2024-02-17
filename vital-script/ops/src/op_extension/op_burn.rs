//! The Burn opcode.

use parity_scale_codec::{Decode, Encode};

use vital_script_derive::ExtensionOpcode;
use vital_script_primitives::resources::Resource;

use crate::instruction::{resource_burn::InstructionResourceBurn, Instruction};

/// Burn resource which inputs
#[derive(Debug, ExtensionOpcode, Encode, Decode)]
pub struct BurnResource {
    pub resource: Resource,
}

impl From<BurnResource> for Instruction {
    fn from(value: BurnResource) -> Self {
        Instruction::Burn(InstructionResourceBurn { resource: value.resource })
    }
}
