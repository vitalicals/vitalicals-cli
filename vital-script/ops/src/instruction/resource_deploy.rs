//! The Resource Deploy instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    consts::MAX_INPUT_INDEX,
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, Tag},
    traits::*,
    types::vrc20::VRC20MetaData,
};

use crate::op_extension::{DeployVRC20, DeployVRC20S, ExtensionOpcode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionVRC20Deploy {
    pub name_input: u8,
    pub name: Tag,
    pub meta: VRC20MetaData,
}

impl core::fmt::Display for InstructionVRC20Deploy {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VRC20Deploy:({}, {}, {:?})", self.name_input, self.name, self.meta)
    }
}

impl Instruction for InstructionVRC20Deploy {
    fn pre_check(&self) -> Result<()> {
        if self.name_input > MAX_INPUT_INDEX {
            bail!("name input too large")
        }

        if !self.name.is_valid() {
            bail!("Invalid name format");
        }

        if self.name.is_empty() {
            bail!("Invalid name by empty");
        }

        Ok(())
    }

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
            .input_resource_mut()
            .cost(&name_resource)
            .context("cost name resource input")?;

        // cost the name for deploy the vrc20
        context.env_mut().cost_name(self.name).context("cost name")?;

        // set vrc metadata
        context.env_mut().deploy_vrc20(self.name, self.meta.clone()).context("deploy")?;

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

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use vital_script_primitives::{
        resources::{Name, Resource},
        types::vrc20::{VRC20MetaData, VRC20MintMeta},
    };
    use vital_script_runner::mock::*;

    use vital_script_ops::instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert,
        resource_deploy::InstructionVRC20Deploy, Instruction,
    };

    #[test]
    fn test_deploy_without_name_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        ctx.mint_name("abcde");
        ctx.mint_name("abe");

        let outpoint02 = ctx.get_name_outpoint("abe").expect("should exist");

        let name1 = Name::must_from("abcde");
        let name2 = Name::must_from("abe");
        let name_res2 = Resource::name(name2);

        ctx.deploy_vrc20("other", 1000);
        let outpoint03 = ctx.mint_vrc20("other");
        let vrc20_res = Resource::vrc20("other", 1000.into()).expect("vrc20");

        // 1. not use a name resource will failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::Deploy(InstructionVRC20Deploy {
                name_input: 1,
                name: name1,
                meta: VRC20MetaData {
                    decimals: 5,
                    nonce: 1000000,
                    bworkc: 1000000,
                    mint: VRC20MintMeta {
                        mint_amount: 10000,
                        mint_height: 0,
                        max_mints: 100000000,
                    },
                    meta: None,
                },
            })])
            .with_ops()
            .run();

        assert_err_str(res, "not found input", "not use a name resource will failed");

        // 2. use a different name resource will failed.
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res2.clone(),
                }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 1,
                    name: name1,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        mint: VRC20MintMeta {
                            mint_amount: 10000,
                            mint_height: 0,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .with_ops()
            .with_input(outpoint02)
            .run();

        assert_err_str(
            res,
            "the name input is invalid",
            "use a different name resource will failed",
        );

        // 3. use a different type resource resource will failed.
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res.clone(),
                }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 1,
                    name: name1,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        mint: VRC20MintMeta {
                            mint_amount: 10000,
                            mint_height: 0,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .with_ops()
            .with_input(outpoint03)
            .run();

        assert_err_str(
            res,
            "the name input is invalid",
            "use a different name resource will failed",
        );

        Ok(())
    }

    #[test]
    fn test_deploy_should_costed_the_name() -> Result<()> {
        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);

        ctx.mint_name("abcde");
        ctx.mint_name("abe");

        let outpoint01 = ctx.get_name_outpoint("abcde").expect("should exist");
        let outpoint02 = ctx.get_name_outpoint("abe").expect("should exist");

        let name1 = Name::must_from("abcde");
        let name2 = Name::must_from("abe");
        let name_res1 = Resource::name(name1);
        let name_res2 = Resource::name(name2);

        // 1. the name should costed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 1,
                    name: name1,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        mint: VRC20MintMeta {
                            mint_amount: 1000,
                            mint_height: 0,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
                Instruction::move_to(0, name_res1.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(1000)
            .run();

        assert_err_str(res, "had already costed", "not use a name resource will failed");

        // 2. if had deployed, will error, note in mock env, the metadata will set even the tx run failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res1.clone(),
                }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 1,
                    name: name1,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        mint: VRC20MintMeta {
                            mint_amount: 1000,
                            mint_height: 0,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(1000)
            .run();

        assert_err_str(res, "the vrc20 had deployed", "not use a name resource will failed");

        // 3. if the name had costed, will failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: name_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, name_res2),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 1,
                    name: name2,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        mint: VRC20MintMeta {
                            mint_amount: 1000,
                            mint_height: 0,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .with_ops()
            .with_input(outpoint02)
            .with_output(1000)
            .run();

        assert_err_str(res, "had already costed", "not use a name resource will failed");

        Ok(())
    }
}
