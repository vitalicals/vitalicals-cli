use anyhow::{bail, Context, Result};
use bytes::{Buf, Bytes};
use vital_script_primitives::{
    names::{Name, ShortName},
    resources::{Resource, VRC20, VRC721},
    H256, U256,
};

use crate::{
    consts::*,
    instruction::{assert_input::InstructionInputAssert, Instruction},
    opcodes::BasicOp,
};

use super::BasicOpcode;

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20AssertSa32 {
    pub amount: u32,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa32 {
    const OPERAND_SIZE: usize = U32_SIZE + ShortName::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertSa32 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u32();
        let name = ShortName::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (ShortName, u64 amount)
pub struct InputVRC20AssertSa64 {
    pub amount: u64,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa64 {
    const OPERAND_SIZE: usize = U64_SIZE + ShortName::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertSa64 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u64();
        let name = ShortName::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (ShortName, u128 amount)
pub struct InputVRC20AssertSa128 {
    pub amount: u128,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa128 {
    const OPERAND_SIZE: usize = U128_SIZE + ShortName::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertSa128 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u128();
        let name = ShortName::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (ShortName, u256 amount)
pub struct InputVRC20AssertSa256 {
    pub amount: U256,
    pub name: ShortName,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertSa256 {
    const OPERAND_SIZE: usize = U256_SIZE + ShortName::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertSa256 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name.into(), self.amount)),
        })
    }

    fn encode(self) -> Vec<u8> {
        let mut amount_bytes = [0_u8; 32];
        self.amount.to_big_endian(&mut amount_bytes);

        [vec![Self::ID], amount_bytes.to_vec(), self.name.0.to_vec(), vec![self.index]].concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = u256_from_bytes(datas);
        let name = ShortName::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (ShortName, u32 amount)
pub struct InputVRC20AssertA32 {
    pub amount: u32,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA32 {
    const OPERAND_SIZE: usize = U32_SIZE + Name::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertA32 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u32();
        let name = Name::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (Name, u64 amount)
pub struct InputVRC20AssertA64 {
    pub amount: u64,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA64 {
    const OPERAND_SIZE: usize = U64_SIZE + Name::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertA64 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u64();
        let name = Name::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (Name, u128 amount)
pub struct InputVRC20AssertA128 {
    pub amount: u128,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA128 {
    const OPERAND_SIZE: usize = U128_SIZE + Name::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertA128 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount.into())),
        })
    }

    fn encode(self) -> Vec<u8> {
        [vec![Self::ID], self.amount.to_be_bytes().to_vec(), self.name.0.to_vec(), vec![self.index]]
            .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = datas.get_u128();
        let name = Name::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC20 Res Assert for (Name, u256 amount)
pub struct InputVRC20AssertA256 {
    pub amount: U256,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC20AssertA256 {
    const OPERAND_SIZE: usize = U256_SIZE + Name::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC20AssertA256 as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC20(VRC20::new(self.name, self.amount)),
        })
    }

    fn encode(self) -> Vec<u8> {
        let mut amount_bytes = [0_u8; 32];
        self.amount.to_big_endian(&mut amount_bytes);

        [vec![Self::ID], amount_bytes.to_vec(), self.name.0.to_vec(), vec![self.index]].concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let amount = u256_from_bytes(datas);
        let name = Name::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { amount, name, index })
    }
}

/// Input VRC721 Res Assert for (Name, hash256 )
pub struct InputVRC721Assert {
    pub hash: H256,
    pub name: Name,
    pub index: u8,
}

impl BasicOpcode for InputVRC721Assert {
    const OPERAND_SIZE: usize = H256_SIZE + Name::SIZE + 1;
    const ID: u8 = BasicOp::InputVRC721Assert as u8;

    fn into_instruction(self) -> Instruction {
        Instruction::Input(InstructionInputAssert {
            index: self.index,
            resource: Resource::VRC721(VRC721::new(self.name, self.hash)),
        })
    }

    fn encode(self) -> Vec<u8> {
        [
            vec![Self::ID],
            self.hash.to_fixed_bytes().to_vec(),
            self.name.0.to_vec(),
            vec![self.index],
        ]
        .concat()
    }

    fn decode_operand(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::OPERAND_SIZE {
            bail!("not enough bytes for {}, expect {}", Self::ID, Self::OPERAND_SIZE);
        }

        let hash = h256_from_bytes(datas);
        let name = Name::from_bytes(datas).context("name")?;
        let index = datas.get_u8();

        Ok(Self { hash, name, index })
    }
}
