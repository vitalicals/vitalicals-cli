//! The Resource Deploy instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, Tag},
    traits::*,
    types::vrc20::VRC20MetaData,
};

use crate::{
    instruction::VitalInstruction,
    op_extension::{DeployVRC20, DeployVRC20S, ExtensionOpcode},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionVRC20Deploy {
    pub name_input: u8,
    pub name: Tag,
    pub meta: VRC20MetaData,
}

impl VitalInstruction for InstructionVRC20Deploy {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // cost the name, check if the vrc20 had deployed.
        let metadata = context.env().get_vrc20_metadata(self.name).context("get vrc20 metadata")?;
        if metadata.is_some() {
            bail!("the vrc20 had deployed");
        }

        // check name resource
        let name_resource =
            context.env().get_input_resource(self.name_input).context("get resource")?;
        if !matches!(name_resource, Resource::Name(n) if n == self.name) {
            bail!("the name input is invalid");
        }
        context
            .input_resource()
            .cost(&name_resource)
            .context("cost name resource input")?;

        // cost the name for deploy the vrc20
        context.env().cost_name(self.name).context("cost name")?;

        // set vrc metadata
        context.env().deploy_vrc20(self.name, self.meta.clone()).context("deploy")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let name_len = self.name.len();
        let res = if name_len <= SHORT_NAME_LEN_MAX {
            DeployVRC20S {
                name_input: self.name_input,
                name: self.name.try_into().expect("the name should be short"),
                meta: self.meta,
            }
            .encode_op()
        } else if name_len <= NAME_LEN_MAX {
            DeployVRC20 { name_input: self.name_input, name: self.name, meta: self.meta }
                .encode_op()
        } else {
            bail!("not support long name")
        };

        Ok(res)
    }
}
