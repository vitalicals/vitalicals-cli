//! The primitive type for vital scripts.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub mod names;
pub mod resources;

pub use primitive_types::{H256, U256};
