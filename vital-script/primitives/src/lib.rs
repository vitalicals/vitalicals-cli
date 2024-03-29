//! The primitive type for vital scripts.
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod consts;
pub mod names;
pub mod resources;
pub mod traits;
pub mod types;

pub use primitive_types::{H256, U256};
