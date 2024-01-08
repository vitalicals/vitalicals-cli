//! The input assert instruction

use anyhow::{bail, Context as AnyhowContext, Result};
use vital_script_primitives::{resources::Resource, traits::*};

use crate::Instruction;

pub struct InstructionInputAssert {
    index: u8,
    resource: Resource,
}

impl Instruction for InstructionInputAssert {
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
}
