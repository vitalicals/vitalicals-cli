//! The Resource Mint instruction

use alloc::vec::Vec;
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, ResourceType, VRC20},
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
        Ok(match self.resource_type.clone() {
            ResourceType::Name { name } => Resource::name(name),
            ResourceType::VRC20 { name } => {
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

                Resource::VRC20(VRC20 { name, amount: U256::from(amount) })
            }
            ResourceType::VRC721 { hash } => Resource::vrc721(hash),
        })
    }
}

impl Instruction for InstructionResourceMint {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // println!("InstructionResourceMint");

        context.runner_mut().try_mint()?;

        let resource = self.make_mint_resource(context)?;
        match &resource {
            Resource::Name(n) => {
                // TODO: need check this in pre-check
                if !n.is_valid() {
                    bail!("Invalid name resource format");
                }

                if n.is_empty() {
                    bail!("Invalid name by empty");
                }

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
            Resource::VRC721(v) => {
                // for vrc721, need check if the h256 had mint
                if context.env().vrc721_had_mint(v.hash)? {
                    bail!("vrc721 had mint");
                } else {
                    context.env_mut().mint_vrc721(v.hash).context("mint 721")?;
                }
            }
        }

        context.send_resource_to_output(self.output_index, resource)?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let bytes = match self.resource_type {
            ResourceType::Name { name } => {
                let l = name.len();
                if l <= SHORT_NAME_LEN_MAX {
                    let name = name.try_into().expect("the name should be short");
                    MintShortName { name, index: self.output_index }.encode_op()
                } else if l <= NAME_LEN_MAX {
                    MintName { name, index: self.output_index }.encode_op()
                } else {
                    bail!("not support long name")
                }
            }
            ResourceType::VRC20 { name } => {
                let l = name.len();
                if l <= SHORT_NAME_LEN_MAX {
                    let name = name.try_into().expect("the name should be short");
                    MintShortVRC20 { name, index: self.output_index }.encode_op()
                } else if l <= NAME_LEN_MAX {
                    MintVRC20 { name, index: self.output_index }.encode_op()
                } else {
                    bail!("not support long name")
                }
            }
            ResourceType::VRC721 { hash } => {
                MintVRC721 { hash, index: self.output_index }.encode_op()
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
        H256,
    };
    use vital_script_runner::{mock::*, traits::EnvFunctions};

    use vital_script_ops::instruction::{assert_output::InstructionOutputAssert, Instruction};

    #[test]
    fn test_mint_short_name_invalid_will_failed() -> Result<()> {
        let env_interface = EnvMock::new();

        // mint abc :  0a00 27 0420c000 00
        // mint abcde: 0a00 27 0420c414 00

        TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00270420c00000").unwrap())
            .with_output(1000)
            .run()
            .expect("mint `abc` should ok");

        TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00270420c41400").unwrap())
            .with_output(2000)
            .run()
            .expect("mint `abcde` should ok");

        // use a new env for test.
        let env_interface = EnvMock::new();

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a0027f420c00000").unwrap()) // note this f4 not a value
            .with_output(1000)
            .run();
        assert_err_str(res, "Invalid name resource format", "1");

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00270420c4ff00").unwrap()) // note this f4 not a value
            .with_output(2000)
            .run();
        assert_err_str(res, "Invalid name resource format", "2");

        Ok(())
    }

    #[test]
    fn test_mint_name_invalid_will_failed() -> Result<()> {
        let env_interface = EnvMock::new();

        // mint abc@de : 0a00280420e5105000000600
        // 0a00
        // op  name            output
        // 28 0420e51050000006   00

        // mint abc@de1122 : 0a00280420e510571c75da00
        // 0a00
        // op  name            output
        // 28 0420e510571c75da   00

        TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280420e5105000000600").unwrap())
            .with_output(3000)
            .run()
            .expect("mint `abc@de` should ok");

        TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280420e510571c75da00").unwrap())
            .with_output(4000)
            .run()
            .expect("mint `abc@de1122` should ok");

        // use a new env for test.
        let env_interface = EnvMock::new();

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280ff0e5105000000600").unwrap())
            .with_output(1000)
            .run();
        assert_err_str(res, "Invalid name resource format", "1");

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280ff0e510571c75da00").unwrap())
            .with_output(2000)
            .run();
        assert_err_str(res, "Invalid name resource format", "2");

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a002804200510571c75da00").unwrap())
            .with_output(3000)
            .run();
        assert_err_str(res, "Invalid name resource format", "3");

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280420e510571c75d900").unwrap()) // length 0xa -> 0x9
            .with_output(4000)
            .run();
        assert_err_str(res, "Invalid name resource format", "4");

        let res = TestCtx::new(&env_interface)
            .with_ops_bytes(&hex::decode("0a00280ff0e5105000000700").unwrap()) // length 0x6 -> 0x7
            .with_output(5000)
            .run();
        assert_err_str(res, "Invalid name resource format", "5");

        Ok(())
    }

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

    #[test]
    fn mint_vrc721_should_work() -> Result<()> {
        let env_interface = EnvMock::new();

        let hash1 = H256::random();
        let hash2 = H256::random();

        let vrc721_res1 = Resource::vrc721(hash1);
        let vrc721_res2 = Resource::vrc721(hash2);

        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc721_res1.resource_type()),
            ])
            .with_ops()
            .with_output(2000)
            .run()?;

        let outpoint10 = context1.env().get_output(0);

        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            Some(vrc721_res1),
            "the new should be some"
        );

        let context2 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 10] }),
                Instruction::mint(10, vrc721_res2.resource_type()),
            ])
            .with_ops()
            .with_outputs(11, 2000)
            .run()?;

        let outpoint210 = context2.env().get_output(10);

        assert_eq!(
            env_interface.get_resources(&outpoint210).expect("get resource"),
            Some(vrc721_res2),
            "the new should be some"
        );

        Ok(())
    }

    #[test]
    fn mint_vrc721_two_times_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();

        let hash1 = H256::random();

        let vrc721_res1 = Resource::vrc721(hash1);

        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, vrc721_res1.resource_type()),
            ])
            .with_ops()
            .with_output(2000)
            .run()?;

        let outpoint10 = context1.env().get_output(0);

        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            Some(vrc721_res1.clone()),
            "the new should be some"
        );

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 10] }),
                Instruction::mint(10, vrc721_res1.resource_type()),
            ])
            .with_ops()
            .with_outputs(11, 2000)
            .run();

        assert_err_str(res, "vrc721 had mint", "mint_vrc721_two_times_should_failed");

        Ok(())
    }

    #[test]
    fn mint_vrc721_two_times_in_one_tx_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();

        let hash1 = H256::random();
        let hash2 = H256::random();

        let vrc721_res1 = Resource::vrc721(hash1);
        let vrc721_res2 = Resource::vrc721(hash2);

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 10] }),
                Instruction::mint(0, vrc721_res1.resource_type()),
                Instruction::mint(10, vrc721_res2.resource_type()),
            ])
            .with_ops()
            .with_outputs(11, 2000)
            .run();

        assert_err_str(
            res,
            "each tx can only have one mint",
            "mint_vrc721_two_times_should_failed",
        );

        Ok(())
    }
}
