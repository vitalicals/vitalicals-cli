//! The Resource Deploy instruction

use alloc::vec::Vec;
use anyhow::{Context as AnyhowContext, Result};
use vital_script_primitives::{resources::Resource, traits::*};

use crate::op_extension::{BurnResource, ExtensionOpcode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceBurn {
    pub resource: Resource,
}

impl core::fmt::Display for InstructionResourceBurn {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Burn:({})", self.resource)
    }
}

impl Instruction for InstructionResourceBurn {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        context
            .input_resource_mut()
            .cost(&self.resource)
            .context("cost resource failed")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let res = BurnResource { resource: self.resource }.encode_op();

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use vital_script_primitives::resources::{Name, Resource};
    use vital_script_runner::mock::*;

    use vital_script_ops::instruction::{
        assert_input::InstructionInputAssert, resource_burn::InstructionResourceBurn, Instruction,
    };

    #[test]
    fn test_burn_no_input_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        ctx.mint_name("abcde");
        ctx.mint_name("abe");

        let outpoint01 = ctx.get_name_outpoint("abcde").expect("should exist");
        let outpoint02 = ctx.get_name_outpoint("abe").expect("should exist");

        let name_res1 = Resource::name(Name::must_from("abcde"));
        let name_res2 = Resource::name(Name::must_from("abe"));

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res1.clone(),
                }),
                Instruction::Burn(InstructionResourceBurn { resource: name_res2 }),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not found name abe res in inputs", "merge name diff type");

        ctx.deploy_vrc20("abe", 10000);
        let outpoint = ctx.mint_vrc20("abe");
        let vrc20_res = Resource::vrc20("abe", 10000.into())?;

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res.clone(),
                }),
                Instruction::Burn(InstructionResourceBurn {
                    resource: Resource::vrc20("abe", 15000.into())?,
                }),
            ])
            .with_ops()
            .with_input(outpoint)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "merge name diff type");

        Ok(())
    }

    #[test]
    fn test_burn_work() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        ctx.mint_name("abcde");
        ctx.mint_name("abe");

        let outpoint01 = ctx.get_name_outpoint("abcde").expect("should exist");
        let outpoint02 = ctx.get_name_outpoint("abe").expect("should exist");

        let name_res1 = Resource::name(Name::must_from("abcde"));
        let name_res2 = Resource::name(Name::must_from("abe"));

        TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: name_res2.clone(),
                }),
                Instruction::Burn(InstructionResourceBurn { resource: name_res1 }),
                Instruction::Burn(InstructionResourceBurn { resource: name_res2 }),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .run()
            .expect("should ok");

        ctx.deploy_vrc20("abcce", 10000);
        let outpoint = ctx.mint_vrc20("abcce");
        let vrc20_res = Resource::vrc20("abcce", 10000.into())?;

        TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res.clone(),
                }),
                Instruction::Burn(InstructionResourceBurn {
                    resource: Resource::vrc20("abcce", 10000.into())?,
                }),
            ])
            .with_ops()
            .with_input(outpoint)
            .run()
            .expect("should ok");

        Ok(())
    }
}
