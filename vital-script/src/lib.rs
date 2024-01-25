//! vital script

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod block_handler;

// re-export
pub use vital_script_ops as ops;
pub use vital_script_primitives as primitives;
pub use vital_script_runner as runner;

pub(crate) const TARGET: &str = "vital-script";
