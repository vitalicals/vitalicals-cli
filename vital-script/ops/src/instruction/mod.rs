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

pub use resource_mint::*;

use vital_script_primitives::{resources::Resource, traits::Context};

pub trait VitalInstruction {
    fn pre_check(&self) -> Result<()> {
        Ok(())
    }

    fn exec(&self, context: &mut impl Context) -> Result<()>;

    fn into_ops_bytes(self) -> Result<Vec<u8>>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Input(assert_input::InstructionInputAssert),
    Output(assert_output::InstructionOutputAssert),
    Mint(resource_mint::InstructionResourceMint),
}

impl VitalInstruction for Instruction {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        match self {
            Self::Input(i) => i.exec(context),
            Self::Output(i) => i.exec(context),
            Self::Mint(i) => i.exec(context),
        }
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Input(i) => i.into_ops_bytes(),
            Self::Output(i) => i.into_ops_bytes(),
            Self::Mint(i) => i.into_ops_bytes(),
        }
    }
}

impl Instruction {
    pub fn mint(index: u8, resource: impl Into<Resource>) -> Self {
        Self::Mint(InstructionResourceMint::new(index, resource))
    }
}
