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

use vital_script_primitives::{
    resources::{Resource, ResourceType, Tag, VRC20},
    traits::{Context, Instruction as InstructionT},
    U256,
};

use self::resource_move::InstructionResourceMove;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Input(assert_input::InstructionInputAssert),
    Output(assert_output::InstructionOutputAssert),
    Mint(resource_mint::InstructionResourceMint),
    Deploy(resource_deploy::InstructionVRC20Deploy),
    Move(resource_move::InstructionResourceMove),
    MoveAll(resource_move::InstructionResourceMoveAll),
}

impl core::fmt::Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Input(i) => i.fmt(f),
            Self::Output(i) => i.fmt(f),
            Self::Mint(i) => i.fmt(f),
            Self::Deploy(i) => i.fmt(f),
            Self::Move(i) => i.fmt(f),
            Self::MoveAll(i) => i.fmt(f),
        }
    }
}

impl InstructionT for Instruction {
    fn exec(&self, context: &mut impl Context) -> Result<()> {
        match self {
            Self::Input(i) => i.exec(context),
            Self::Output(i) => i.exec(context),
            Self::Mint(i) => i.exec(context),
            Self::Deploy(i) => i.exec(context),
            Self::Move(i) => i.exec(context),
            Self::MoveAll(i) => i.exec(context),
        }
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        match self {
            Self::Input(i) => i.into_ops_bytes(),
            Self::Output(i) => i.into_ops_bytes(),
            Self::Mint(i) => i.into_ops_bytes(),
            Self::Deploy(i) => i.into_ops_bytes(),
            Self::Move(i) => i.into_ops_bytes(),
            Self::MoveAll(i) => i.into_ops_bytes(),
        }
    }
}

impl Instruction {
    pub fn mint(index: u8, resource_type: ResourceType) -> Self {
        Self::Mint(InstructionResourceMint::new(index, resource_type))
    }

    pub fn move_to(index: u8, resource: impl Into<Resource>) -> Self {
        Self::Move(InstructionResourceMove::new(index, resource))
    }

    pub fn move_vrc20_to(index: u8, name: impl Into<Tag>, amount: impl Into<U256>) -> Self {
        Self::Move(InstructionResourceMove::new(
            index,
            Resource::VRC20(VRC20::new(name.into(), amount.into())),
        ))
    }
}
