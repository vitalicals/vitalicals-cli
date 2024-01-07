//! The resources types

use crate::names::Name;

pub mod vrc20;
pub mod vrc721;

pub use vrc20::*;
pub use vrc721::*;

#[derive(Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ResourceClass {
    Name = 0x01,
    VRC20 = 0x02,
    VRC721 = 0x03,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ResourceType {
    pub class: ResourceClass,
    pub name: Name,
}

pub enum Resource {
    Name(Name),
    VRC20(VRC20),
    VRC721(VRC721),
}
