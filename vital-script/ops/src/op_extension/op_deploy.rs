//! The Deploy opcode.

use parity_scale_codec::{Decode, Encode};

use vital_script_derive::ExtensionOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    types::vrc20::VRC20MetaData,
};

use crate::instruction::{resource_deploy::InstructionVRC20Deploy, Instruction};

/// Deploy VRC20 with ShortName
#[derive(Debug, ExtensionOpcode, Encode, Decode)]
pub struct DeployVRC20S {
    pub name_input: u8,
    pub name: ShortName,
    pub meta: VRC20MetaData,
}

impl From<DeployVRC20S> for Instruction {
    fn from(value: DeployVRC20S) -> Self {
        Instruction::Deploy(InstructionVRC20Deploy {
            name_input: value.name_input,
            name: value.name.into(),
            meta: value.meta,
        })
    }
}

/// Deploy VRC20 with Name
#[derive(Debug, ExtensionOpcode, Encode, Decode)]
pub struct DeployVRC20 {
    pub name_input: u8,
    pub name: Name,
    pub meta: VRC20MetaData,
}

impl From<DeployVRC20> for Instruction {
    fn from(value: DeployVRC20) -> Self {
        Instruction::Deploy(InstructionVRC20Deploy {
            name_input: value.name_input,
            name: value.name,
            meta: value.meta,
        })
    }
}
