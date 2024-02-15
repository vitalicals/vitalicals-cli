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
        // println!("InstructionResourceMint");

        context.runner_mut().try_mint()?;

        let resource = self.make_mint_resource(context)?;
        match &resource {
            Resource::Name(n) => {
                // for name, we need flag it
                context.env_mut().new_name(*n).context("new name failed")?;
            }
            Resource::VRC20(v) => {
                // for vrc20, we need add mint count
                context
                    .env_mut()
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

#[cfg(test)]
mod tests {
    use anyhow::{anyhow, Result};

    use vital_script_primitives::{
        resources::{Name, Resource},
        traits::{Context, EnvContext},
    };
    use vital_script_runner::{mock::*, traits::EnvFunctions};

    use vital_script_ops::instruction::{assert_output::InstructionOutputAssert, Instruction};

    #[test]
    fn test_mint_name_two_times_will_failed() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        ctx.mint_name("abcde");
        let name1 = Name::must_from("abcde");
        let name_res1 = Resource::name(name1);

        // 1. the `abcde` had mint, so this will failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, name_res1.resource_type()),
            ])
            .with_ops()
            .with_output(1000)
            .run();

        assert_err_str(res, "the name had created", "mint names two times will failed");

        // deploy a vrc will cost the name
        ctx.deploy_vrc20("abcde", 10000);

        // 2. event the name had costed, also cannot mint
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, name_res1.resource_type()),
            ])
            .with_ops()
            .with_output(1000)
            .run();

        assert_err_str(res, "the name had created", "mint names two times will failed");

        Ok(())
    }

    #[test]
    fn test_mint_vrc20_no_deployed_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        let vrc20_res1 = Resource::vrc20("abcde", 1000.into())?;

        // 1. the `abcde` not deployed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc20_res1.resource_type()),
            ])
            .with_ops()
            .with_output(1000)
            .run();

        assert_err_str(
            res,
            "not found vrc20 metadata, may not deployed",
            "the `abcde` not deployed",
        );

        ctx.deploy_vrc20("abe", 1000);

        // 2. had a `abe`, but not `abcde`
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc20_res1.resource_type()),
            ])
            .with_ops()
            .with_output(1000)
            .run();

        assert_err_str(
            res,
            "not found vrc20 metadata, may not deployed",
            "the `abcde` not deployed",
        );

        Ok(())
    }

    #[test]
    fn test_mint_vrc20_max_count() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        let vrc20_res1 = Resource::vrc20("abcde", 1000.into())?;
        let vrc20_res2 = Resource::vrc20("abe", 1000.into())?;

        ctx.deploy_vrc20_with_max("abcde", 1000, 100);
        ctx.deploy_vrc20_with_max("abe", 1000, 100);

        for i in 0..100 {
            let ctx = TestCtx::new(&env_interface)
                .with_instructions(vec![
                    Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                    Instruction::mint(0, vrc20_res1.resource_type()),
                ])
                .with_ops()
                .with_output((i + 1) * 1000) // make id diff
                .run()?;

            let out = ctx.env().get_output(0);
            assert_eq!(
                env_interface.get_resources(&out)?.ok_or(anyhow!("should found in {}", i))?,
                vrc20_res1
            );
        }

        // more than max count should failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc20_res1.resource_type()),
            ])
            .with_ops()
            .with_output(9999)
            .run();

        assert_err_str(res, "mint count had reached max", "the `abcde` not deployed");

        // other will ok
        let ctx = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc20_res2.resource_type()),
            ])
            .with_ops()
            .with_output(33333) // make id diff
            .run()?;

        let out = ctx.env().get_output(0);
        assert_eq!(env_interface.get_resources(&out)?.ok_or(anyhow!("should found"))?, vrc20_res2);

        Ok(())
    }
}
