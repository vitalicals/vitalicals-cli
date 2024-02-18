//! The resources types

use core::fmt;

use anyhow::{bail, Result};

#[cfg(feature = "std")]
use primitive_types::U256;

use parity_scale_codec::{Decode, Encode};

pub use crate::names::Name;
use crate::{names::ShortName, H256};

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
pub enum ResourceType {
    Name { name: Tag },
    VRC20 { name: Tag },
    VRC721 { hash: H256 },
}

impl fmt::Display for ResourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceType::Name { name } => {
                write!(f, "name({})", name)
            }
            ResourceType::VRC20 { name } => {
                write!(f, "vrc20({})", name)
            }
            ResourceType::VRC721 { hash } => {
                write!(f, "vrc721({})", hash)
            }
        }
    }
}

impl ResourceType {
    pub fn name(n: impl Into<Tag>) -> Self {
        Self::Name { name: n.into() }
    }

    pub fn vrc20(n: impl Into<Tag>) -> Self {
        Self::VRC20 { name: n.into() }
    }

    pub fn vrc721(hash: impl Into<H256>) -> Self {
        Self::VRC721 { hash: hash.into() }
    }

    pub fn is_vrc20(&self) -> bool {
        matches!(self, Self::VRC20 { name: _ })
    }

    pub fn get_tag(&self) -> Option<&Tag> {
        match self {
            ResourceType::Name { name } => Some(name),
            ResourceType::VRC20 { name } => Some(name),
            ResourceType::VRC721 { hash: _ } => None,
        }
    }
}

#[derive(Debug, Clone, Encode, Decode, PartialOrd, Ord, PartialEq, Eq)]
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
    pub fn vrc20(name: impl Into<String>, amount: U256) -> Result<Self> {
        let name = Name::try_from(name.into())?;

        Ok(Self::VRC20(VRC20::new(name, amount)))
    }

    pub fn as_vrc20(&self) -> Result<&VRC20> {
        if let Self::VRC20(v) = self {
            Ok(v)
        } else {
            bail!("resource is not vrc20")
        }
    }

    pub fn vrc721(hash: impl Into<H256>) -> Self {
        Self::VRC721(VRC721::new(hash.into()))
    }

    pub fn as_vrc721(&self) -> Result<&VRC721> {
        if let Self::VRC721(v) = self {
            Ok(v)
        } else {
            bail!("resource is not vrc721")
        }
    }

    pub fn name(name: impl Into<Tag>) -> Self {
        Self::Name(name.into())
    }

    pub fn resource_type(&self) -> ResourceType {
        match self {
            Self::Name(n) => ResourceType::Name { name: *n },
            Self::VRC20(v) => ResourceType::VRC20 { name: v.name },
            Self::VRC721(v) => ResourceType::VRC721 { hash: v.hash },
        }
    }

    pub fn merge(&mut self, other: &Resource) -> Result<()> {
        match (self, other) {
            (Self::VRC20(v), Self::VRC20(o)) => {
                if v.name != o.name {
                    bail!("the vrc20 not support merge by diff name")
                }

                v.amount += o.amount;
                Ok(())
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
