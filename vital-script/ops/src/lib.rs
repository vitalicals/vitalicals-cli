//! The opcode defines

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod basic;
pub mod builder;
pub mod instruction;
pub mod opcodes;
pub mod parser;
