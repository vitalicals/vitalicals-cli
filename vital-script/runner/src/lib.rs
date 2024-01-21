//! A Runner for vital scripts
//!
//! It will run a vital script then call the impl callback.
//!
//! A Runner need depend the env trait which mainly contains the resource interface.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use anyhow::{Context as AnyhowContext, Result};

pub mod traits;

mod context;
mod outputs;
mod resource_cache;

#[cfg(test)]
mod mock;

use context::Context;
use traits::EnvFunctions;
use vital_script_ops::instruction::{Instruction, VitalInstruction};

pub struct Runner<Functions: EnvFunctions> {
    instructions: Vec<Instruction>,
    context: Context<Functions>,
}

impl<Functions: EnvFunctions> Runner<Functions> {
    pub fn new(context: Context<Functions>) -> Result<Self> {
        let instructions = context.get_instructions().context("get instructions")?;

        Ok(Self { instructions, context })
    }

    pub fn run(&mut self) -> Result<()> {
        // 1. run pre check
        self.pre_check().context("pre check")?;

        // 2. run opcodes, cost input resources, call env traits.
        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction
                .exec(&mut self.context)
                .with_context(|| format!("execute {}", index))?;
        }

        // 3. post check
        self.post_check().context("post check")?;

        // 4. apply the resources
        self.context.apply_resources().context("apply")?;

        Ok(())
    }
}

impl<Functions: EnvFunctions> Runner<Functions> {
    // The pre checks for context and instructions.
    fn pre_check(&self) -> Result<()> {
        self.context.pre_check().context("context")?;

        for (index, instruction) in self.instructions.iter().enumerate() {
            instruction.pre_check().with_context(|| format!("instruction {}", index))?;
        }

        Ok(())
    }

    // The post check
    fn post_check(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bdk::bitcoin::OutPoint;
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
    use crate::mock::*;

    #[test]
    fn test_simple_runner() {
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

        let env_interface = EnvMock::new(tx_mock);
        let context = Context::new(env_interface.clone());

        let mut runner = Runner::new(context).expect("new runner");

        runner.run().expect("run failed");

        println!("res storage {:?}", env_interface.resource_storage);
    }

    // TODO: need move the tests

    #[test]
    fn test_mint_name_then_deploy_vrc20() {
        let mint_name_str = "abcdefg";
        let mint_name = Name::try_from(mint_name_str.to_string()).unwrap();
        let mint_amount = U256::from(10000);

        let mut env_inner1 = {
            // 1. mint a name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::name(mint_name)),
            ])
            .expect("build should ok");

            let mut tx_mock1 = TxMock::new();
            tx_mock1.push_ops(ops_bytes);
            tx_mock1.push_output(1000);

            let env_interface = EnvMock::new(tx_mock1);
            let context = Context::new(env_interface.clone());

            let mut runner = Runner::new(context).expect("new runner");

            runner.run().expect("run failed");

            env_interface
        };

        // check name
        let outpoint = env_inner1.get_output(0).expect("get output failed");
        let res = env_inner1.get_resources(&outpoint).expect("get resources failed");

        assert_eq!(res, Some(Resource::name(mint_name)));

        let is_costed = Context::new(env_inner1.clone())
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(false));

        // println!("env {:?}", env_inner1);

        let mut env_inner2 = {
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

            println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let tx_mock1_id = env_inner1.current_tx.txid.clone();
            let mut tx_mock2 = TxMock::new();
            tx_mock2.push_input(OutPoint::new(tx_mock1_id, 0));
            tx_mock2.push_ops(ops_bytes);
            tx_mock2.push_output(2000);

            env_inner1.next_psbt(tx_mock2);

            let context = Context::new(env_inner1.clone());
            let mut runner = Runner::new(context).expect("new runner");
            runner.run().expect("run failed");

            env_inner1
        };

        // check name
        let outpoint = env_inner2.get_output(0).expect("get output failed");
        let res = env_inner2.get_resources(&outpoint).expect("get resources failed");

        // name should had costed
        assert_eq!(res, None);

        let mut ctx = Context::new(env_inner2.clone());
        let is_costed = ctx
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(true));

        // check deployed
        let vrc20 = ctx.env().get_vrc20_metadata(mint_name).expect("should have metadata");
        assert!(vrc20.is_some());

        // 3. mint vrc20
        let env_inner3 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::vrc20(mint_name)),
            ])
            .expect("build should ok");

            println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock3 = TxMock::new();
            tx_mock3.push_output(2000);
            tx_mock3.push_ops(ops_bytes);

            env_inner2.next_psbt(tx_mock3);

            let context = Context::new(env_inner2.clone());
            let mut runner = Runner::new(context).expect("new runner");
            runner.run().expect("run failed");

            env_inner2
        };

        let outpoint = env_inner3.get_output(0).expect("get output failed");
        let res = env_inner3.get_resources(&outpoint).expect("get resources failed");

        assert_eq!(res, Some(Resource::vrc20(mint_name_str, mint_amount).expect("res")));
    }

    #[test]
    fn test_mint_short_name_then_deploy_vrc20() {
        let mint_name_str = "abc";
        let mint_name = Name::try_from(mint_name_str.to_string()).unwrap();
        let mint_amount = U256::from(10000);

        let mut env_inner1 = {
            // 1. mint a name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::name(mint_name)),
            ])
            .expect("build should ok");

            let mut tx_mock1 = TxMock::new();
            tx_mock1.push_ops(ops_bytes);
            tx_mock1.push_output(1000);

            let env_interface = EnvMock::new(tx_mock1);
            let context = Context::new(env_interface.clone());

            let mut runner = Runner::new(context).expect("new runner");

            runner.run().expect("run failed");

            env_interface
        };

        // check name
        let outpoint = env_inner1.get_output(0).expect("get output failed");
        let res = env_inner1.get_resources(&outpoint).expect("get resources failed");

        assert_eq!(res, Some(Resource::name(mint_name)));

        let is_costed = Context::new(env_inner1.clone())
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(false));

        // println!("env {:?}", env_inner1);

        let mut env_inner2 = {
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

            println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let tx_mock1_id = env_inner1.current_tx.txid.clone();
            let mut tx_mock2 = TxMock::new();
            tx_mock2.push_input(OutPoint::new(tx_mock1_id, 0));
            tx_mock2.push_ops(ops_bytes);
            tx_mock2.push_output(2000);

            env_inner1.next_psbt(tx_mock2);

            let context = Context::new(env_inner1.clone());
            let mut runner = Runner::new(context).expect("new runner");
            runner.run().expect("run failed");

            env_inner1
        };

        // check name
        let outpoint = env_inner2.get_output(0).expect("get output failed");
        let res = env_inner2.get_resources(&outpoint).expect("get resources failed");

        // name should had costed
        assert_eq!(res, None);

        let mut ctx = Context::new(env_inner2.clone());
        let is_costed = ctx
            .env()
            .get_metadata::<bool>(mint_name, MetaDataType::Name)
            .expect("should have metadata");
        assert_eq!(is_costed, Some(true));

        // check deployed
        let vrc20 = ctx.env().get_vrc20_metadata(mint_name).expect("should have metadata");
        assert!(vrc20.is_some());

        // 3. mint vrc20
        let env_inner3 = {
            // 2. deploy a vrc20 by the name
            let ops_bytes = ScriptBuilderFromInstructions::build(vec![
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::mint(0, ResourceType::vrc20(mint_name)),
            ])
            .expect("build should ok");

            println!("ops_bytes: {:?}", hex::encode(&ops_bytes));

            let mut tx_mock3 = TxMock::new();
            tx_mock3.push_output(2000);
            tx_mock3.push_ops(ops_bytes);

            env_inner2.next_psbt(tx_mock3);

            let context = Context::new(env_inner2.clone());
            let mut runner = Runner::new(context).expect("new runner");
            runner.run().expect("run failed");

            env_inner2
        };

        let outpoint = env_inner3.get_output(0).expect("get output failed");
        let res = env_inner3.get_resources(&outpoint).expect("get resources failed");

        assert_eq!(res, Some(Resource::vrc20(mint_name_str, mint_amount).expect("res")));
    }
}
