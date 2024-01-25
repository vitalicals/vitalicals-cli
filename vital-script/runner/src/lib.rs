//! A Runner for vital scripts
//!
//! It will run a vital script then call the impl callback.
//!
//! A Runner need depend the env trait which mainly contains the resource interface.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub(crate) const TARGET: &str = "vital::runner";

use anyhow::{Context as AnyhowContext, Result};

pub mod traits;

mod context;
mod resource_cache;

#[cfg(test)]
mod mock;

pub use context::{script::check_is_vital_script, Context};

use vital_script_ops::instruction::Instruction;
use vital_script_primitives::traits::{Context as ContextT, Instruction as InstructionT};

pub struct Runner<Context: ContextT<Instruction = Instruction>> {
    _marker: core::marker::PhantomData<Context>,
}

impl<Context: ContextT<Instruction = Instruction>> Runner<Context> {
    pub fn new() -> Self {
        Self { _marker: Default::default() }
    }

    pub fn run(&mut self, context: &mut Context) -> Result<()> {
        log::debug!(target: TARGET, "run instructions");

        let instructions = context.get_instructions().context("get instructions")?;
        log::debug!(target: TARGET, "run instructions len {}", instructions.len());

        // 1. run pre check
        context.pre_check().context("context")?;

        for (index, instruction) in instructions.iter().enumerate() {
            instruction
                .pre_check()
                .with_context(|| alloc::format!("instruction {}", index))?;
        }

        // 2. run opcodes, cost input resources, call env traits.
        for (index, instruction) in instructions.iter().enumerate() {
            log::debug!(target: TARGET, "run instruction {} : {}", index, instruction);

            instruction.exec(context).with_context(|| alloc::format!("execute {}", index))?;
        }

        // 3. post check

        // 4. apply the resources
        context.apply_resources().context("apply")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vital_script_ops::{
        builder::instruction::ScriptBuilderFromInstructions,
        instruction::{
            assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert,
            resource_deploy::InstructionVRC20Deploy,
        },
    };
    use vital_script_primitives::{
        names::Name,
        resources::{Resource, ResourceType},
        traits::{Context as ContextT, EnvContext, MetaDataType},
        types::vrc20::{VRC20MetaData, VRC20MintMeta},
        U256,
    };

    use super::*;
    use crate::{mock::*, traits::EnvFunctions};

    #[test]
    fn test_simple_runner() {
        init_logger();

        let mint_name = Name::try_from("abcdefg".to_string()).unwrap();
        let instructions = vec![
            Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
            Instruction::mint(0, ResourceType::name(mint_name)),
        ];
        let ops_bytes =
            ScriptBuilderFromInstructions::build(instructions).expect("build should ok");

        let mut tx_mock = TxMock::new();
        tx_mock.push_ops(ops_bytes);
        tx_mock.push_output(1000);

        let mut context = ContextMock::new(tx_mock, EnvMock::new());

        Runner::new().run(&mut context).expect("run failed");
    }

    // TODO: need move the tests

    #[test]
    fn test_mint_name_then_deploy_vrc20() {
        init_logger();

        let mint_name_str = "abcdefg";
        let mint_name = Name::try_from(mint_name_str.to_string()).unwrap();
        let mint_amount = U256::from(10000);

        let env_interface = EnvMock::new();

        let mut context1 = {
            // 1. mint a name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::name(mint_name)),
            ])
            .expect("build should ok");

            let mut tx_mock1 = TxMock::new();
            tx_mock1.push_ops(ops_bytes);
            tx_mock1.push_output(1000);

            let mut context = ContextMock::new(tx_mock1, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let is_costed = context1
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(false));

        let outpoint = context1.env().get_output(0);
        let res = env_interface.get_resources(&outpoint).expect("get resources failed");
        assert_eq!(res, Some(Resource::name(mint_name)));

        let mut context2 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 0,
                    resource: Resource::Name(mint_name),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 0,
                    name: mint_name,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        max: U256::from(1000000000000000_u64),
                        mint: VRC20MintMeta {
                            mint_type: 1,
                            mint_amount,
                            mint_height: 10,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock2 = TxMock::new();
            tx_mock2.push_input(outpoint);
            tx_mock2.push_ops(ops_bytes);
            tx_mock2.push_output(2000);

            let mut context = ContextMock::new(tx_mock2, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let is_costed = context2
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(true));

        // check name
        let outpoint = context2.env().get_output(0);
        let res = env_interface.get_resources(&outpoint).expect("get resources failed");

        // name should had costed
        assert_eq!(res, None);

        // check deployed
        let vrc20 = context2.env().get_vrc20_metadata(mint_name).expect("should have metadata");
        assert!(vrc20.is_some());

        // 3. mint vrc20
        let vrc20_in_2 = Resource::vrc20(mint_name_str, mint_amount).expect("res");

        let mut context3 = {
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::vrc20(mint_name)),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock3 = TxMock::new();
            tx_mock3.push_output(2000);
            tx_mock3.push_ops(ops_bytes);

            let mut context = ContextMock::new(tx_mock3, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            let outpoint = context.env().get_output(0);
            let res = env_interface.get_resources(&outpoint).expect("get resources failed");

            assert_eq!(res, Some(vrc20_in_2.clone()));

            context
        };

        // 4. transfer vrc20
        let mut context4 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 0,
                    resource: vrc20_in_2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, vrc20_in_2),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: 4 {:?}", hex::encode(&ops_bytes));

            // the minted vrc20s
            let mut tx_mock4 = TxMock::new();
            tx_mock4.push_input(context3.env().get_output(0));
            tx_mock4.push_output(2000);
            tx_mock4.push_ops(ops_bytes);

            let mut context = ContextMock::new(tx_mock4, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let res = env_interface
            .get_resources(&context4.env().get_output(0))
            .expect("get resources failed");

        assert_eq!(res, Some(Resource::vrc20(mint_name_str, mint_amount).expect("res")));
    }

    #[test]
    fn test_mint_short_name_then_deploy_vrc20() {
        init_logger();

        let mint_name_str = "abc";
        let mint_name = Name::try_from(mint_name_str.to_string()).unwrap();
        let mint_amount = U256::from(10000);

        let env_interface = EnvMock::new();

        let mut context1 = {
            // 1. mint a name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::name(mint_name)),
            ])
            .expect("build should ok");

            let mut tx_mock1 = TxMock::new();
            tx_mock1.push_ops(ops_bytes);
            tx_mock1.push_output(1000);

            let mut context = ContextMock::new(tx_mock1, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let is_costed = context1
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(false));

        let outpoint = context1.env().get_output(0);
        let res = env_interface.get_resources(&outpoint).expect("get resources failed");
        assert_eq!(res, Some(Resource::name(mint_name)));

        let mut context2 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 0,
                    resource: Resource::Name(mint_name),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::Deploy(InstructionVRC20Deploy {
                    name_input: 0,
                    name: mint_name,
                    meta: VRC20MetaData {
                        decimals: 5,
                        nonce: 1000000,
                        bworkc: 1000000,
                        max: U256::from(1000000000000000_u64),
                        mint: VRC20MintMeta {
                            mint_type: 1,
                            mint_amount,
                            mint_height: 10,
                            max_mints: 100000000,
                        },
                        meta: None,
                    },
                }),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock2 = TxMock::new();
            tx_mock2.push_input(outpoint);
            tx_mock2.push_ops(ops_bytes);
            tx_mock2.push_output(2000);

            let mut context = ContextMock::new(tx_mock2, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let is_costed = context2
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(true));

        // check name
        let outpoint = context2.env().get_output(0);
        let res = env_interface.get_resources(&outpoint).expect("get resources failed");

        // name should had costed
        assert_eq!(res, None);

        // check deployed
        let vrc20 = context2.env().get_vrc20_metadata(mint_name).expect("should have metadata");
        assert!(vrc20.is_some());

        // 3. mint vrc20
        let vrc20_in_2 = Resource::vrc20(mint_name_str, mint_amount).expect("res");

        let mut context3 = {
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::vrc20(mint_name)),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock3 = TxMock::new();
            tx_mock3.push_output(2000);
            tx_mock3.push_ops(ops_bytes);

            let mut context = ContextMock::new(tx_mock3, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            let outpoint = context.env().get_output(0);
            let res = env_interface.get_resources(&outpoint).expect("get resources failed");

            assert_eq!(res, Some(vrc20_in_2.clone()));

            context
        };

        // 4. transfer vrc20
        let mut context4 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Input(InstructionInputAssert {
                    index: 0,
                    resource: vrc20_in_2.clone(),
                }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, vrc20_in_2),
            ])
            .expect("build should ok");

            log::info!("ops_bytes: 4 {:?}", hex::encode(&ops_bytes));

            // the minted vrc20s
            let mut tx_mock4 = TxMock::new();
            tx_mock4.push_input(context3.env().get_output(0));
            tx_mock4.push_output(2000);
            tx_mock4.push_ops(ops_bytes);

            let mut context = ContextMock::new(tx_mock4, env_interface.clone());
            Runner::new().run(&mut context).expect("run failed");

            context
        };

        let res = env_interface
            .get_resources(&context4.env().get_output(0))
            .expect("get resources failed");

        assert_eq!(res, Some(Resource::vrc20(mint_name_str, mint_amount).expect("res")));
    }
}
