//! The Resource Deploy instruction

use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, Tag},
    traits::*,
    types::{vrc20::VRC20MintMeta, MetaData},
    U256,
};

use crate::{
    instruction::VitalInstruction,
    op_basic::{BasicOpcode, MintName},
    op_extension::{DeployVRC20, DeployVRC20S, ExtensionOpcode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionVRC20Deploy {
    pub name_input: u8,
    pub decimals: u8,
    pub name: Tag,
    pub max: U256,
    pub nonce: u64,
    pub bworkc: u64,
    pub mint: VRC20MintMeta,
    pub meta: Option<MetaData>,
}

impl VitalInstruction for InstructionVRC20Deploy {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // cost the name, check if the vrc20 had deployed.

        todo!();
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let name_len = self.name.len();
        let res = if name_len <= SHORT_NAME_LEN_MAX {
            DeployVRC20S {
                name_input: self.name_input,
                decimals: self.decimals,
                name: self.name.try_into().expect("the name should be short"),
                max: self.max,
                nonce: self.nonce,
                bworkc: self.bworkc,
                mint: self.mint,
                meta: self.meta,
            }
            .encode_op()
        } else if name_len <= NAME_LEN_MAX {
            DeployVRC20 {
                name_input: self.name_input,
                decimals: self.decimals,
                name: self.name,
                max: self.max,
                nonce: self.nonce,
                bworkc: self.bworkc,
                mint: self.mint,
                meta: self.meta,
            }
            .encode_op()
        } else {
            bail!("not support long name")
        };

        Ok(res)
    }
}
