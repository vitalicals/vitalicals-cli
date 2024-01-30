//! The templates for each types of scripts

mod utils;

mod deploy_vrc20;
mod mint_name;
mod mint_vrc20;
mod move_name;
mod move_vrc20;

pub use deploy_vrc20::*;
pub use mint_name::*;
pub use mint_vrc20::*;
pub use move_name::*;
pub use move_vrc20::*;

/// The outputs used by template
pub type Outputs = Vec<u8>;
