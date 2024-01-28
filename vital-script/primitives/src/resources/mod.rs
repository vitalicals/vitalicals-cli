//! The resources types

use core::fmt;

use anyhow::{bail, Result};

#[cfg(feature = "std")]
use primitive_types::U256;

use parity_scale_codec::{Decode, Encode};

pub use crate::names::Name;
use crate::names::ShortName;

pub mod vrc20;
pub mod vrc721;

pub use vrc20::*;
pub use vrc721::*;

pub type Tag = Name;

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[repr(u16)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ResourceClass {
    Name = 0x01,
    VRC20 = 0x02,
    VRC721 = 0x03,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResourceType {
    pub class: ResourceClass,
    pub name: Tag,
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.class {
            ResourceClass::Name => {
                write!(f, "name({})", self.name)
            }
            ResourceClass::VRC20 => {
                write!(f, "vrc20({})", self.name)
            }
            ResourceClass::VRC721 => {
                write!(f, "vrc721({})", self.name)
            }
        }
    }
}

impl ResourceType {
    pub fn name(n: impl Into<Tag>) -> Self {
        Self { class: ResourceClass::Name, name: n.into() }
    }

    pub fn vrc20(n: impl Into<Tag>) -> Self {
        Self { class: ResourceClass::VRC20, name: n.into() }
    }

    pub fn vrc721(n: impl Into<Tag>) -> Self {
        Self { class: ResourceClass::VRC721, name: n.into() }
    }

    pub fn is_vrc20(&self) -> bool {
        self.class == ResourceClass::VRC20
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Resource {
    Name(Name),
    VRC20(VRC20),
    VRC721(VRC721),
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(n) => {
                write!(f, "name({})", n)
            }
            Self::VRC20(v) => {
                write!(f, "vrc20({})", v)
            }
            Self::VRC721(v) => {
                write!(f, "vrc721({})", v)
            }
        }
    }
}

impl Default for Resource {
    fn default() -> Self {
        Self::Name(Name::default())
    }
}

impl Resource {
    #[cfg(feature = "std")]
    pub fn vrc20(name: impl Into<String>, amount: U256) -> Result<Self> {
        let name = Name::try_from(name.into())?;

        Ok(Self::VRC20(VRC20::new(name, amount)))
    }

    pub fn name(name: impl Into<Tag>) -> Self {
        Self::Name(name.into())
    }

    pub fn resource_type(&self) -> ResourceType {
        let (class, name) = match self {
            Self::Name(n) => (ResourceClass::Name, *n),
            Self::VRC20(v) => (ResourceClass::VRC20, v.name),
            Self::VRC721(v) => (ResourceClass::VRC721, v.name),
        };

        ResourceType { class, name }
    }

    pub fn merge(&mut self, other: &Resource) -> Result<()> {
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

    pub fn merge_into(self, other: &Resource) -> Result<Self> {
        match (self, other) {
            (Self::VRC20(mut v), Self::VRC20(o)) => {
                v.amount += o.amount;
                Ok(Resource::VRC20(v))
            }
            _ => {
                bail!("the resource type not support merge")
            }
        }
    }
}

impl From<ShortName> for Resource {
    fn from(value: ShortName) -> Self {
        Self::Name(value.into())
    }
}

impl From<Name> for Resource {
    fn from(value: Name) -> Self {
        Self::Name(value)
    }
}

impl From<VRC20> for Resource {
    fn from(value: VRC20) -> Self {
        Self::VRC20(value)
    }
}

impl From<VRC721> for Resource {
    fn from(value: VRC721) -> Self {
        Self::VRC721(value)
    }
}
