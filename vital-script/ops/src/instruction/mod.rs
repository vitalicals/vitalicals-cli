//! The instructions for the script runner.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use anyhow::Result;

mod utils;

pub mod assert_input;
pub mod assert_output;
pub mod resource_burn;
pub mod resource_deploy;
pub mod resource_mint;
pub mod resource_move;

use vital_script_primitives::traits::Context;

pub trait VitalInstruction {
    fn exec(self, context: &mut impl Context) -> Result<()>;

    fn into_ops_bytes(self) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Input(assert_input::InstructionInputAssert),
    Output(assert_output::InstructionOutputAssert),
}

impl VitalInstruction for Instruction {
    fn exec(self, context: &mut impl Context) -> Result<()> {
        match self {
            Self::Input(i) => i.exec(context),
            Self::Output(i) => i.exec(context),
        }
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Input(i) => i.into_ops_bytes(),
            Self::Output(i) => i.into_ops_bytes(),
        }
    }
}
