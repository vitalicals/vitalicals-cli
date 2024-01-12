use anyhow::{bail, Context, Result};
use bytes::{Buf, Bytes};
use vital_script_primitives::names::{Name, ShortName};

use crate::{consts::*, instruction::Instruction, opcodes::BasicOp};

use super::*;

/// Transfer all VRC20 Res to a output for (ShortName)
pub struct TransferAllVRC20S {
    pub name: ShortName,
    pub output_index: u8,
}

impl From<TransferAllVRC20S> for Instruction {
    fn from(_value: TransferAllVRC20S) -> Self {
        todo!()
    }
}

impl Opcode for TransferAllVRC20S {
    const ID: u8 = BasicOp::TransferAllVRC20S as u8;
}

impl BasicOpcodeCodec for TransferAllVRC20S {
    const OPERAND_SIZE: usize = ShortName::SIZE + 1;

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.name.0.to_vec(), vec![self.output_index]].concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let name = ShortName::from_bytes(datas).context("name")?;
        let output_index = datas.get_u8();

        Ok(Self { name, output_index })
    }
}

/// Transfer all VRC20 Res to a output for (Name)
pub struct TransferAllVRC20 {
    pub name: Name,
    pub output_index: u8,
}

impl From<TransferAllVRC20> for Instruction {
    fn from(_value: TransferAllVRC20) -> Self {
        todo!()
    }
}

impl Opcode for TransferAllVRC20 {
    const ID: u8 = BasicOp::TransferAllVRC20 as u8;
}

impl BasicOpcodeCodec for TransferAllVRC20 {
    const OPERAND_SIZE: usize = Name::SIZE + 1;

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.name.0.to_vec(), vec![self.output_index]].concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let name = Name::from_bytes(datas).context("name")?;
        let output_index = datas.get_u8();

        Ok(Self { name, output_index })
    }
}

/// Transfer VRC20 Res with a amount to a output for (ShortName, u32)
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

impl Opcode for TransferVRC20Sa32 {
    const ID: u8 = BasicOp::TransferVRC20Sa32 as u8;
}

impl BasicOpcodeCodec for TransferVRC20Sa32 {
    const OPERAND_SIZE: usize = ShortName::SIZE + U32_SIZE + 1;

    fn encode(self) -> Vec<u8> {
        [
            vec![Self::ID],
            self.name.0.to_vec(),
            self.amount.to_be_bytes().to_vec(),
            vec![self.output_index],
        ]
        .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let name = ShortName::from_bytes(datas).context("name")?;
        let amount = datas.get_u32();
        let output_index = datas.get_u8();

        Ok(Self { name, amount, output_index })
    }
}

/// Transfer VRC20 Res with a amount to a output for (Name, u32)
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

impl Opcode for TransferVRC20A32 {
    const ID: u8 = BasicOp::TransferVRC20A32 as u8;
}

impl BasicOpcodeCodec for TransferVRC20A32 {
    const OPERAND_SIZE: usize = ShortName::SIZE + U32_SIZE + 1;

    fn encode(self) -> Vec<u8> {
        [
            vec![Self::ID],
            self.name.0.to_vec(),
            self.amount.to_be_bytes().to_vec(),
            vec![self.output_index],
        ]
        .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let name = Name::from_bytes(datas).context("name")?;
        let amount = datas.get_u32();
        let output_index = datas.get_u8();

        Ok(Self { name, amount, output_index })
    }
}

// TODO: more
