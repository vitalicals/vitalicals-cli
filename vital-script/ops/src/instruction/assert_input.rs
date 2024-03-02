//! The input assert instruction

use alloc::vec::Vec;
use anyhow::{bail, Context as AnyhowContext, Result};
use parity_scale_codec::Encode;
use vital_script_primitives::{
    consts::*,
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, Tag, VRC20, VRC721},
    traits::*,
};

use crate::{
    instruction::utils::*,
    op_basic::{BasicOpcodeBase, InputAssertName, InputAssertShortName, InputVRC721Assert},
    TARGET,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionInputAssert {
    pub index: u8,
    pub resource: Resource,
}

impl core::fmt::Display for InstructionInputAssert {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "InputAssert:({}, {})", self.index, self.resource)
    }
}

impl Instruction for InstructionInputAssert {
    fn pre_check(&self) -> Result<()> {
        if self.index > MAX_INPUT_INDEX {
            bail!("index too large")
        }

        Ok(())
    }

    fn exec(&self, context: &mut impl Context) -> Result<()> {
        // 1. ensure if current input index is not asserted.
        context.runner_mut().try_assert_input(self.index)?;

        // 2. ensure the resource is expected by index.
        if context.run_mod().is_skip_check() {
            log::debug!(target: TARGET, "skip input resource check by in sim mode" )
        } else {
            let resource_from_env =
                context.env().get_input_resource(self.index).context("get input resource")?;

            if resource_from_env != self.resource {
                log::debug!(target: TARGET, "resource from {:?} expect {:?}", resource_from_env, self.resource);
                bail!("the resource not expected")
            }
        }

        // 3. push the resource into resources.
        context
            .input_resource_mut()
            .push(self.index, self.resource.clone())
            .context("push input resource")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self.resource {
            Resource::Name(name) => Self::into_input_name(name, self.index),
            Resource::VRC20(vrc20) => Self::into_input_vrc20(vrc20, self.index),
            Resource::VRC721(vrc721) => Self::into_input_vrc721(vrc721, self.index),
        }
    }
}

impl InstructionInputAssert {
    fn into_input_name(n: Tag, index: u8) -> Result<Vec<u8>> {
        // TODO: use a common way to process different name
        let name_len = n.len();
        let (opcode, mut res) = if name_len <= SHORT_NAME_LEN_MAX {
            (
                <InputAssertShortName as BasicOpcodeBase>::ID,
                InputAssertShortName { name: n.try_into().expect("should ok"), index }.encode(),
            )
        } else if name_len <= NAME_LEN_MAX {
            (<InputAssertName as BasicOpcodeBase>::ID, InputAssertName { name: n, index }.encode())
        } else {
            bail!("not support long name")
        };

        let mut bytes = Vec::with_capacity(4 + res.len());
        bytes.push(opcode);
        bytes.append(&mut res);

        Ok(bytes)
    }

    fn into_input_vrc20(v: VRC20, index: u8) -> Result<Vec<u8>> {
        Vrc20ResourceOperand::new(v).into_input_vrc20_opcode_bytes(index)
    }

    fn into_input_vrc721(v: VRC721, index: u8) -> Result<Vec<u8>> {
        let op = InputVRC721Assert { hash: v.hash, index };

        let mut bytes = Vec::with_capacity(512);
        bytes.push(<InputVRC721Assert as BasicOpcodeBase>::ID);
        bytes.append(&mut op.encode());

        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use vital_script_primitives::resources::{Name, Resource};
    use vital_script_runner::mock::*;

    use vital_script_ops::instruction::{assert_input::InstructionInputAssert, Instruction};

    #[test]
    fn test_input_assert_not_match_should_failed() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);
        ctx.mint_name("abcde");

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint03 = ctx.get_name_outpoint("abcde").expect("should exist");

        let name_res = Resource::name(Name::must_from("abcde"));
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into()).expect("vrc20");

        let name_res_not_match = Resource::name(Name::must_from("abcd"));
        let vrc20_res1_not_match = Resource::vrc20(test_name1, 999.into()).expect("vrc20");

        // 1. different vrc20 by name
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: vrc20_res2.clone(),
            })])
            .with_ops()
            .with_input(outpoint01)
            .run();

        assert_err_str(res, "the resource not expected", "input not match 1");

        // 2. different vrc20 by amount
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: vrc20_res1_not_match.clone(),
            })])
            .with_ops()
            .with_input(outpoint01)
            .run();

        assert_err_str(res, "the resource not expected", "input not match 2");

        // 3. different res type
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: name_res.clone(),
            })])
            .with_ops()
            .with_input(outpoint01)
            .run();

        assert_err_str(res, "the resource not expected", "input not match 3");

        // 4. different name
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::Input(InstructionInputAssert {
                index: 1,
                resource: name_res_not_match.clone(),
            })])
            .with_ops()
            .with_input(outpoint03)
            .run();

        assert_err_str(res, "the resource not expected", "input not match 3");

        Ok(())
    }
}
