//! The opcode defines

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod builder;
pub mod instruction;
pub mod op_basic;
pub mod op_extension;
pub mod opcodes;
pub mod parser;

mod utils;
