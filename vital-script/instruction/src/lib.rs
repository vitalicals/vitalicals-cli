//! The instructions for the script runner.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use anyhow::Result;

pub mod assert_input;
pub mod assert_output;
pub mod resource_burn;
pub mod resource_deploy;
pub mod resource_mint;
pub mod resource_move;

use vital_script_primitives::traits::Context;

pub trait Instruction {
    fn exec(self, context: &mut impl Context) -> Result<()>;
}
