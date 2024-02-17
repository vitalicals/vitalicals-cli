//! The Resource Move instruction

use alloc::vec::Vec;
use anyhow::{anyhow, bail, Context as AnyhowContext, Result};
use vital_script_primitives::{
    names::{NAME_LEN_MAX, SHORT_NAME_LEN_MAX},
    resources::{Resource, ResourceType},
    traits::*,
};

use crate::{
    instruction::utils::Vrc20ResourceOperand,
    op_basic::{BasicOpcode, MoveAllVRC20, MoveAllVRC20S, MoveName, MoveShortName, MoveVRC721},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMove {
    pub output_index: u8,
    pub resource: Resource,
}

impl InstructionResourceMove {
    pub fn new(index: u8, resource: impl Into<Resource>) -> Self {
        Self { output_index: index, resource: resource.into() }
    }
}

impl core::fmt::Display for InstructionResourceMove {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResourceMove:({}, {})", self.output_index, self.resource)
    }
}

impl Instruction for InstructionResourceMove {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        context
            .input_resource_mut()
            .cost(&self.resource)
            .context("cost resource failed")?;

        context
            .send_resource_to_output(self.output_index, self.resource.clone())
            .context("send to output failed")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        let raw = match self.resource {
            Resource::Name(name) => match name.len() {
                n if n <= SHORT_NAME_LEN_MAX => MoveShortName {
                    name: name.try_into().context("the name is not short")?,
                    output_index: self.output_index,
                }
                .encode_op(),
                n if n <= NAME_LEN_MAX => {
                    MoveName { name, output_index: self.output_index }.encode_op()
                }
                _ => {
                    bail!("not support long name")
                }
            },
            Resource::VRC20(vrc20) => Vrc20ResourceOperand::new(vrc20)
                .into_move_vrc20_opcode_bytes(self.output_index)
                .context("use Vrc20ResourceOperand into opcode bytes")?,
            Resource::VRC721(vrc721) => {
                MoveVRC721 { name: vrc721.name, hash: vrc721.hash, output_index: self.output_index }
                    .encode_op()
            }
        };

        Ok(raw)
    }
}

impl InstructionResourceMove {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionResourceMoveAll {
    pub output_index: u8,
    pub resource_type: ResourceType,
}

impl InstructionResourceMoveAll {
    pub fn new(index: u8, resource_type: ResourceType) -> Self {
        Self { output_index: index, resource_type }
    }
}

impl core::fmt::Display for InstructionResourceMoveAll {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResourceMoveAll:({}, {})", self.output_index, self.resource_type)
    }
}

impl Instruction for InstructionResourceMoveAll {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        let resource = context
            .input_resource()
            .get_uncosted_vrc20(self.resource_type.name)
            .ok_or_else(|| anyhow!("not found vrc20 resource by name"))?;

        context.input_resource_mut().cost(&resource).context("cost resource failed")?;

        context
            .send_resource_to_output(self.output_index, resource)
            .context("send to output failed")?;

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        if !self.resource_type.is_vrc20() {
            bail!("only vrc20 resource type support move all");
        }

        let raw = match self.resource_type.name.len() {
            n if n <= SHORT_NAME_LEN_MAX => MoveAllVRC20S {
                name: self.resource_type.name.try_into().context("the name is not short")?,
                output_index: self.output_index,
            }
            .encode_op(),
            n if n <= NAME_LEN_MAX => {
                MoveAllVRC20 { name: self.resource_type.name, output_index: self.output_index }
                    .encode_op()
            }
            _ => {
                bail!("not support long name")
            }
        };

        Ok(raw)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use vital_script_primitives::{
        resources::{Name, Resource},
        traits::{Context, EnvContext},
        U256,
    };
    use vital_script_runner::{mock::*, traits::EnvFunctions};

    use vital_script_ops::instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert,
        resource_move::InstructionResourceMoveAll, Instruction,
    };

    #[allow(dead_code)]
    pub fn init_logger() {
        let _ = env_logger::Builder::from_default_env()
            .format_module_path(true)
            .format_level(true)
            .filter_level(log::LevelFilter::Info)
            .parse_filters(format!("{}=debug", crate::TARGET).as_str())
            .parse_filters("vital::runner=debug")
            .try_init();
    }

    fn test_move_name_impl(test_name: &str) -> Result<()> {
        let test = Name::try_from(test_name).expect("name format");
        let test_res = Resource::Name(test);

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.mint_name(test_name);

        let outpoint1 = ctx.get_name_outpoint(test_name).expect("should mint");

        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, test_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint1)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        let outpoint2 = context1.env().get_output(0);

        assert_eq!(
            env_interface.get_resources(&outpoint1).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint2).expect("get resource"),
            Some(test_res.clone()),
            "the new should be some"
        );

        let context2 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_to(1, test_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint2)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        let outpoint3 = context2.env().get_output(1);

        assert_eq!(
            env_interface.get_resources(&outpoint2).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface
                .get_resources(&context2.env().get_output(0))
                .expect("get resource"),
            None,
            "the no move should be none"
        );
        assert_eq!(
            env_interface
                .get_resources(&context2.env().get_output(2))
                .expect("get resource"),
            None,
            "the no move should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint3).expect("get resource"),
            Some(test_res.clone()),
            "the new should be some"
        );

        test_move_name_had_costed_failed_impl(test_name)?;

        Ok(())
    }

    fn test_move_name_had_costed_failed_impl(test_name: &str) -> Result<()> {
        let test = Name::try_from(test_name).expect("name format");
        let test_res = Resource::Name(test);

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.mint_name(test_name);

        let outpoint1 = ctx.get_name_outpoint(test_name).expect("should mint");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_to(0, test_res.clone()),
                Instruction::move_to(1, test_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint1)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "had already costed", format!("name {} costed", test_name).as_str());

        Ok(())
    }

    #[test]
    fn test_move_name() -> Result<()> {
        test_move_name_impl("abcde").expect("test move abcde should ok");
        test_move_name_impl("a").expect("test move a should ok");
        test_move_name_impl("a1").expect("test move a1 should ok");
        test_move_name_impl("abcde1234").expect("test move abcde1234 should ok");
        test_move_name_impl("abcde@1234").expect("test move abcde@1234 should ok");

        Ok(())
    }

    fn test_move_vrc20_impl(test_name: &str) -> Result<()> {
        let mint_amount = u128::MAX - 1;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name, mint_amount);

        let mint_res = Resource::vrc20(test_name, mint_amount.into()).expect("should vrc20");
        let outpoint01 = ctx.mint_vrc20(test_name);
        let outpoint02 = ctx.mint_vrc20(test_name);
        let outpoint03 = ctx.mint_vrc20(test_name);

        // move to 1
        let test_res1 = Resource::vrc20(test_name, 100.into()).expect("should vrc20");
        let charge_res1 =
            Resource::vrc20(test_name, (mint_amount - 200).into()).expect("should vrc20");
        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: mint_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2, 3] }),
                Instruction::move_to(0, test_res1.clone()),
                Instruction::move_to(1, test_res1.clone()),
                Instruction::move_to(2, charge_res1.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        let outpoint10 = context1.env().get_output(0);
        let outpoint11 = context1.env().get_output(1);
        let outpoint12 = context1.env().get_output(2);
        let outpoint13 = context1.env().get_output(3);

        assert_eq!(
            env_interface.get_resources(&outpoint01).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            Some(test_res1.clone()),
            "the new should be some"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint11).expect("get resource"),
            Some(test_res1.clone()),
            "the new should be some"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint12).expect("get resource"),
            Some(charge_res1.clone()),
            "the new should be some"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint13).expect("get resource"),
            None,
            "the new should be none"
        );

        let test_res2 =
            Resource::vrc20(test_name, U256::from(u128::MAX) + U256::from(u128::MAX) + 8)
                .expect("should vrc20");
        let test_res_no_enough =
            Resource::vrc20(test_name, U256::from(u128::MAX) + U256::from(u128::MAX) + 10)
                .expect("should vrc20");
        let charge_res2 = Resource::vrc20(test_name, 90.into()).expect("should vrc20");

        let should_no_enough = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: test_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert { index: 2, resource: mint_res.clone() }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: mint_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_to(1, test_res_no_enough.clone()),
                Instruction::move_to(2, charge_res2.clone()),
            ])
            .with_ops()
            .with_input(outpoint10)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(3000)
            .with_output(3000)
            .with_output(3000)
            .run();

        assert_eq!(
            should_no_enough.err().expect("should not have enough").root_cause().to_string(),
            "not enough inputs"
        );

        let context2 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: test_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert { index: 2, resource: mint_res.clone() }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: mint_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_to(1, test_res2.clone()),
                Instruction::move_to(2, charge_res2.clone()),
            ])
            .with_ops()
            .with_input(outpoint10)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(3000)
            .with_output(3000)
            .with_output(3000)
            .run()
            .expect("transfer name failed");

        let outpoint30 = context2.env().get_output(0);
        let outpoint31 = context2.env().get_output(1);
        let outpoint32 = context2.env().get_output(2);

        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint02).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint03).expect("get resource"),
            None,
            "the old should be none"
        );

        assert_eq!(
            env_interface.get_resources(&outpoint30).expect("get resource"),
            None,
            "the no move should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint31).expect("get resource"),
            Some(test_res2.clone()),
            "the new should be some"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint32).expect("get resource"),
            Some(charge_res2.clone()),
            "the new should be some"
        );

        // move one output 3 times by same vrc20 resource
        let context4 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: charge_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_to(1, Resource::vrc20(test_name, 30.into())?),
                Instruction::move_to(1, Resource::vrc20(test_name, 30.into())?),
                Instruction::move_to(1, Resource::vrc20(test_name, 30.into())?),
            ])
            .with_ops()
            .with_input(outpoint32)
            .with_output(3000)
            .with_output(3000)
            .run()
            .expect("transfer name failed");

        let outpoint41 = context4.env().get_output(1);
        assert_eq!(
            env_interface.get_resources(&outpoint41).expect("get resource"),
            Some(charge_res2.clone()),
            "the new should be some"
        );

        Ok(())
    }

    fn test_move_all_vrc20_impl(test_name: &str) -> Result<()> {
        let test = Name::try_from(test_name).expect("name format");
        let mint_amount = u128::MAX - 1;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name, mint_amount);

        let mint_res = Resource::vrc20(test_name, mint_amount.into()).expect("should vrc20");
        let outpoint01 = ctx.mint_vrc20(test_name);
        let outpoint02 = ctx.mint_vrc20(test_name);
        let outpoint03 = ctx.mint_vrc20(test_name);

        // move to 1
        let test_res1 = Resource::vrc20(test_name, 100.into()).expect("should vrc20");
        let charge_res1 = Resource::vrc20(
            test_name,
            U256::from(mint_amount - 200) + U256::from(mint_amount) + U256::from(mint_amount),
        )
        .expect("should vrc20");
        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: mint_res.clone() }),
                Instruction::Input(InstructionInputAssert { index: 2, resource: mint_res.clone() }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: mint_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2, 3] }),
                Instruction::move_vrc20_to(0, test, 100),
                Instruction::move_vrc20_to(1, test, 100),
                Instruction::MoveAll(InstructionResourceMoveAll::new(2, mint_res.resource_type())),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        assert_eq!(
            env_interface.get_resources(&outpoint01).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint02).expect("get resource"),
            None,
            "the old should be none"
        );
        assert_eq!(
            env_interface.get_resources(&outpoint03).expect("get resource"),
            None,
            "the old should be none"
        );

        assert_eq!(
            env_interface
                .get_resources(&context1.env().get_output(0))
                .expect("get resource"),
            Some(test_res1.clone()),
            "the new should be some"
        );

        assert_eq!(
            env_interface
                .get_resources(&context1.env().get_output(1))
                .expect("get resource"),
            Some(test_res1.clone()),
            "the new should be some"
        );

        assert_eq!(
            env_interface
                .get_resources(&context1.env().get_output(2))
                .expect("get resource"),
            Some(charge_res1),
            "the new should be some"
        );

        assert_eq!(
            env_interface
                .get_resources(&context1.env().get_output(3))
                .expect("get resource"),
            None,
            "the new should be none"
        );

        Ok(())
    }

    #[test]
    fn test_move_vrc20() -> Result<()> {
        // init_logger();

        test_move_vrc20_impl("abcde").expect("test move abcde should ok");
        test_move_vrc20_impl("a").expect("test move a should ok");
        test_move_vrc20_impl("a1").expect("test move a1 should ok");
        test_move_vrc20_impl("abcde1234").expect("test move abcde1234 should ok");
        test_move_vrc20_impl("abcde@1234").expect("test move abcde@1234 should ok");

        test_move_all_vrc20_impl("abcde").expect("test move abcde should ok");
        test_move_all_vrc20_impl("a").expect("test move a should ok");
        test_move_all_vrc20_impl("a1").expect("test move a1 should ok");
        test_move_all_vrc20_impl("abcde1234").expect("test move abcde1234 should ok");
        test_move_all_vrc20_impl("abcde@1234").expect("test move abcde@1234 should ok");

        Ok(())
    }

    #[test]
    fn test_move_into_output_two_times_will_failed() -> Result<()> {
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
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: name_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_to(2, name_res1.clone()),
                Instruction::move_to(2, name_res2.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "the resource type not support merge", "merge name diff type");

        Ok(())
    }

    #[test]
    fn test_move_vrc20_merged() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name2);
        ctx.mint_name("abcde");

        let outpoint03 = ctx.get_name_outpoint("abcde").expect("should exist");

        let name_res = Resource::name(Name::must_from("abcde"));
        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into())?;

        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: name_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 100),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 200),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 300),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 400),
                Instruction::move_vrc20_to(1, Name::must_from(test_name2), 500),
                Instruction::move_vrc20_to(1, Name::must_from(test_name2), 500),
                Instruction::move_to(2, name_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        let outpoint10 = context1.env().get_output(0);
        let outpoint11 = context1.env().get_output(1);
        let outpoint12 = context1.env().get_output(2);

        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            Some(vrc20_res1),
            "the new should be some"
        );

        assert_eq!(
            env_interface.get_resources(&outpoint11).expect("get resource"),
            Some(vrc20_res2),
            "the new should be some"
        );

        assert_eq!(
            env_interface.get_resources(&outpoint12).expect("get resource"),
            Some(name_res),
            "the new should be some"
        );

        Ok(())
    }

    #[test]
    fn test_move_vrc20_merged_by_move_all() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let test1 = Name::must_from(test_name1);
        let test2 = Name::must_from(test_name2);

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name1);
        let outpoint03 = ctx.mint_vrc20(test_name1);
        let outpoint04 = ctx.mint_vrc20(test_name1);
        let outpoint05 = ctx.mint_vrc20(test_name2);
        let outpoint06 = ctx.mint_vrc20(test_name2);

        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into())?;

        let context1 = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 3,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 4,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 5,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 6,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_vrc20_to(0, test1, 100),
                Instruction::move_vrc20_to(1, test1, 100),
                Instruction::move_vrc20_to(2, test2, 100),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    1,
                    vrc20_res1.resource_type(),
                )),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    2,
                    vrc20_res2.resource_type(),
                )),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_input(outpoint04)
            .with_input(outpoint05)
            .with_input(outpoint06)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run()
            .expect("transfer name failed");

        let outpoint10 = context1.env().get_output(0);
        let outpoint11 = context1.env().get_output(1);
        let outpoint12 = context1.env().get_output(2);

        assert_eq!(
            env_interface.get_resources(&outpoint10).expect("get resource"),
            Some(Resource::vrc20("test1", 100.into())?),
            "the new should be some"
        );

        assert_eq!(
            env_interface.get_resources(&outpoint11).expect("get resource"),
            Some(Resource::vrc20("test1", 3900.into())?),
            "the new should be some"
        );

        assert_eq!(
            env_interface.get_resources(&outpoint12).expect("get resource"),
            Some(Resource::vrc20("test2", 2000.into())?),
            "the new should be some"
        );

        Ok(())
    }

    #[test]
    fn test_move_vrc20_merged_diff_type_should_failed() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name2);
        ctx.mint_name("abcde");

        let outpoint03 = ctx.get_name_outpoint("abcde").expect("should exist");

        let name_res = Resource::name(Name::must_from("abcde"));
        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into())?;

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: name_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 100),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 200),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 300),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 400),
                Instruction::move_vrc20_to(0, Name::must_from(test_name2), 500),
                Instruction::move_vrc20_to(0, Name::must_from(test_name2), 500),
                Instruction::move_to(2, name_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "the vrc20 not support merge by diff name", "merge vrc20 diff type");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Input(InstructionInputAssert { index: 3, resource: name_res.clone() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 100),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 200),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 300),
                Instruction::move_vrc20_to(0, Name::must_from(test_name1), 400),
                Instruction::move_vrc20_to(1, Name::must_from(test_name2), 500),
                Instruction::move_vrc20_to(1, Name::must_from(test_name2), 500),
                Instruction::move_to(1, name_res.clone()),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "the resource type not support merge",
            "merge vrc20 and name diff type",
        );

        Ok(())
    }

    #[test]
    fn test_move_vrc20_move_all_should_cost_all() -> Result<()> {
        let test_name1 = "test1";
        let test1 = Name::must_from(test_name1);
        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name1);
        let outpoint03 = ctx.mint_vrc20(test_name1);
        let outpoint04 = ctx.mint_vrc20(test_name1);

        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 3,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 4,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1, 2] }),
                Instruction::move_vrc20_to(0, test1, 100),
                Instruction::move_vrc20_to(1, test1, 100),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    1,
                    vrc20_res1.resource_type(),
                )),
                Instruction::move_vrc20_to(2, test1, 100), // this will failed
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_input(outpoint04)
            .with_output(2000)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "will move all, then all move will failed");

        Ok(())
    }

    #[test]
    fn test_move_no_input_enough_should_failed() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let test1 = Name::must_from(test_name1);

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name1);
        let outpoint03 = ctx.mint_vrc20(test_name1);
        let outpoint05 = ctx.mint_vrc20(test_name2);

        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into())?;

        // no input, will failed, even had a valid input
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_vrc20_to(0, test1, 100),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "not found res in inputs",
            "no input, will failed, even had a valid input",
        );

        // had a not match input, should failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_vrc20_to(0, test1, 100),
            ])
            .with_ops()
            .with_input(outpoint05)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "not found res in inputs",
            "no input, will failed, even had a valid input",
        );

        // had input, but move too much in one move ins
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_vrc20_to(0, test1, 1001),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "no input, will failed, even had a valid input");

        // had input, but move too much in many move ins
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_vrc20_to(0, test1, 300),
                Instruction::move_vrc20_to(1, test1, 701),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "no input, will failed, even had a valid input");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_vrc20_to(0, test1, 1000),
                Instruction::move_vrc20_to(1, test1, 1),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "no input, will failed, even had a valid input");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 3,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0, 1] }),
                Instruction::move_vrc20_to(0, test1, 3000),
                Instruction::move_vrc20_to(1, test1, 1),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_input(outpoint03)
            .with_output(2000)
            .with_output(2000)
            .run();

        assert_err_str(res, "not enough inputs", "no input, will failed, even had a valid input");

        Ok(())
    }

    #[test]
    fn test_move_move_all_with_no_enough_input_should_failed() -> Result<()> {
        let test_name1 = "test1";
        let test_name2 = "test2";

        let test1 = Name::must_from(test_name1);

        let mint_amount = 1000;

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.deploy_vrc20(test_name1, mint_amount);
        ctx.deploy_vrc20(test_name2, mint_amount);

        let outpoint01 = ctx.mint_vrc20(test_name1);
        let outpoint02 = ctx.mint_vrc20(test_name1);
        let outpoint05 = ctx.mint_vrc20(test_name2);

        let vrc20_res1 = Resource::vrc20(test_name1, mint_amount.into())?;
        let vrc20_res2 = Resource::vrc20(test_name2, mint_amount.into())?;

        // no input, will failed, even had a valid input
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    0,
                    vrc20_res1.resource_type(),
                )),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "not found vrc20 resource by name",
            "no input, will failed, even had a valid input",
        );

        // had a not match input, should failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    0,
                    vrc20_res1.resource_type(),
                )),
            ])
            .with_ops()
            .with_input(outpoint05)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "not found vrc20 resource by name",
            "no input, will failed, even had a valid input",
        );

        // had input, but move too much in one move ins
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 1,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Input(InstructionInputAssert {
                    index: 2,
                    resource: vrc20_res1.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_vrc20_to(0, test1, 2000),
                Instruction::MoveAll(InstructionResourceMoveAll::new(
                    0,
                    vrc20_res1.resource_type(),
                )),
            ])
            .with_ops()
            .with_input(outpoint01)
            .with_input(outpoint02)
            .with_output(2000)
            .run();

        assert_err_str(
            res,
            "not found vrc20 resource by name",
            "no input, will failed, even had a valid input",
        );

        Ok(())
    }
}
