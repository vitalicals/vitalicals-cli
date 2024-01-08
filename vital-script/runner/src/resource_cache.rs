//! A Resource cache for runner

use alloc::vec::Vec;
use anyhow::{bail, Result};

use vital_script_primitives::{
    names::Name,
    resources::{Resource, Tag, VRC20, VRC721},
    H256, U256,
};

const VEC_CAP_SIZE: usize = 8;

const INPUT_MAX: usize = 64;
const OUTPUT_MAX: usize = 64;

pub struct ResourceCache {}

impl ResourceCache {}
