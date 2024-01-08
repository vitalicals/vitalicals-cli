//! The resources types

use anyhow::{bail, Result};

pub use crate::names::Name;

pub mod vrc20;
pub mod vrc721;

pub use vrc20::*;
pub use vrc721::*;

pub type Tag = Name;

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
    pub name: Tag,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Resource {
    Name(Name),
    VRC20(VRC20),
    VRC721(VRC721),
}

impl Resource {
    pub fn resource_type(&self) -> ResourceType {
        let (class, name) = match self {
            Self::Name(n) => (ResourceClass::Name, *n),
            Self::VRC20(v) => (ResourceClass::VRC20, v.name),
            Self::VRC721(v) => (ResourceClass::VRC721, v.name),
        };

        ResourceType { class, name }
    }

    pub fn merge(&mut self, other: Resource) -> Result<()> {
        match (self, other) {
            (Self::VRC20(v), Self::VRC20(o)) => {
                v.amount += o.amount;
                Ok(())
            }
            _ => {
                bail!("the resource type not support merge")
            }
        }
    }
}
