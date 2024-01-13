use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use vital_script_derive::BasicOpcode;
use vital_script_primitives::names::{Name, ShortName};

use crate::instruction::Instruction;

/// Transfer all VRC20 Res to a output for (ShortName)
#[derive(Debug, Deserialize, Serialize)]
#[derive(BasicOpcode, Encode, Decode)]
pub struct TransferAllVRC20S {
    pub name: ShortName,
    pub output_index: u8,
}

impl From<TransferAllVRC20S> for Instruction {
    fn from(_value: TransferAllVRC20S) -> Self {
        todo!()
    }
}

/// Transfer all VRC20 Res to a output for (Name)
#[derive(Debug, Deserialize, Serialize)]
#[derive(BasicOpcode, Encode, Decode)]
pub struct TransferAllVRC20 {
    pub name: Name,
    pub output_index: u8,
}

impl From<TransferAllVRC20> for Instruction {
    fn from(_value: TransferAllVRC20) -> Self {
        todo!()
    }
}

/// Transfer VRC20 Res with a amount to a output for (ShortName, u32)
#[derive(Debug, Deserialize, Serialize)]
#[derive(BasicOpcode, Encode, Decode)]
pub struct TransferVRC20Sa32 {
    pub name: ShortName,
    pub amount: u32,
    pub output_index: u8,
}

impl From<TransferVRC20Sa32> for Instruction {
    fn from(_value: TransferVRC20Sa32) -> Self {
        todo!()
    }
}

/// Transfer VRC20 Res with a amount to a output for (Name, u32)
#[derive(Debug, Deserialize, Serialize)]
#[derive(BasicOpcode, Encode, Decode)]
pub struct TransferVRC20A32 {
    pub name: Name,
    pub amount: u32,
    pub output_index: u8,
}

impl From<TransferVRC20A32> for Instruction {
    fn from(_value: TransferVRC20A32) -> Self {
        todo!()
    }
}

// TODO: more
