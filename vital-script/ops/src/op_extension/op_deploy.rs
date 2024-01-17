//! The Deploy opcode.

use parity_scale_codec::{Decode, Encode};

use vital_script_derive::ExtensionOpcode;
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, VRC20, VRC721},
    types::{vrc20::VRC20MintMeta, MetaData},
    H256, U256,
};

use crate::instruction::{resource_deploy::InstructionVRC20Deploy, Instruction};

/// Deploy VRC20 with ShortName
#[derive(Debug, ExtensionOpcode, Encode, Decode)]
pub struct DeployVRC20S {
    pub name_input: u8,
    pub decimals: u8,
    pub name: ShortName,
    pub max: U256,
    pub nonce: u64,
    pub bworkc: u64,
    pub mint: VRC20MintMeta,
    pub meta: Option<MetaData>,
}

impl From<DeployVRC20S> for Instruction {
    fn from(value: DeployVRC20S) -> Self {
        Instruction::Deploy(InstructionVRC20Deploy {
            name_input: value.name_input,
            decimals: value.decimals,
            name: value.name.into(),
            max: value.max,
            nonce: value.nonce,
            bworkc: value.bworkc,
            mint: value.mint,
            meta: value.meta,
        })
    }
}

/// Deploy VRC20 with Name
#[derive(Debug, ExtensionOpcode, Encode, Decode)]
pub struct DeployVRC20 {
    pub name_input: u8,
    pub decimals: u8,
    pub name: Name,
    pub max: U256,
    pub nonce: u64,
    pub bworkc: u64,
    pub mint: VRC20MintMeta,
    pub meta: Option<MetaData>,
}

impl From<DeployVRC20> for Instruction {
    fn from(value: DeployVRC20) -> Self {
        Instruction::Deploy(InstructionVRC20Deploy {
            name_input: value.name_input,
            decimals: value.decimals,
            name: value.name,
            max: value.max,
            nonce: value.nonce,
            bworkc: value.bworkc,
            mint: value.mint,
            meta: value.meta,
        })
    }
}
