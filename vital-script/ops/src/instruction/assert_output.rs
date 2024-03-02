//! The output assert instruction

use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::traits::*;

use crate::op_basic::{
    BasicOpcode, OutputIndexAssert, OutputIndexFlag16Assert, OutputIndexFlag32Assert,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstructionOutputAssert {
    pub indexs: Vec<u8>,
}

impl core::fmt::Display for InstructionOutputAssert {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OutputAssert:({:?})", self.indexs)
    }
}

impl Instruction for InstructionOutputAssert {
    fn pre_check(&self) -> Result<()> {
        Ok(())
    }

    fn exec(&self, context: &mut impl Context) -> Result<()> {
        for index in self.indexs.iter() {
            // 1. ensure if current output index is not asserted.
            context.runner_mut().try_assert_output(*index)?;
        }

        Ok(())
    }

    fn into_ops_bytes(self) -> Result<Vec<u8>> {
        if self.indexs.is_empty() {
            bail!("no index for output")
        }

        if self.indexs.len() == 1 {
            let op = OutputIndexAssert { index: self.indexs[0] };

            return Ok(op.encode_op());
        }

        // check if can use flags16
        let is_all_less_than_16 = self.indexs.iter().all(|a| *a < 16_u8);
        if is_all_less_than_16 {
            let mut mask = 0_u16;
            for i in self.indexs {
                let m = 1_u16 << i;
                mask |= m;
            }

            let op = OutputIndexFlag16Assert { index_flag: mask.to_le_bytes() };

            return Ok(op.encode_op());
        }

        let is_all_less_than_32 = self.indexs.iter().all(|a| *a < 32_u8);
        if is_all_less_than_32 {
            let mut mask = 0_u32;
            for i in self.indexs {
                let m = 1_u32 << i;
                mask |= m;
            }

            let op = OutputIndexFlag32Assert { index_flag: mask.to_le_bytes() };

            return Ok(op.encode_op());
        }

        todo!("not support output index >= 32")
    }
}

#[cfg(test)]
mod tests {
    use crate::opcodes::BasicOp;
    use vital_script_runner::mock::*;

    use super::*;

    fn output_index_assert_ops_bytes(index: u8) {
        let output = InstructionOutputAssert { indexs: vec![index] };

        let bytes = output
            .into_ops_bytes()
            .expect(format!("into should ok for {:?}", index).as_str());

        assert_eq!(bytes.len(), 1 + 1);
        assert_eq!(bytes, vec![BasicOp::OutputIndexAssert as u8, index], "assert {:?}", index);
    }

    #[test]
    fn test_output_into_ops_bytes() -> Result<()> {
        for i in 0..=u8::MAX {
            output_index_assert_ops_bytes(i);
        }

        let should_failed = InstructionOutputAssert { indexs: Vec::new() }.into_ops_bytes();
        assert_err_str(should_failed, "no index for output", "empty outputs should failed");

        Ok(())
    }

    fn output_index_flag16_assert_ops_bytes(indexs: Vec<u8>, o1: u8, o2: u8) {
        let output = InstructionOutputAssert { indexs: indexs.clone() };

        let bytes = output
            .into_ops_bytes()
            .expect(format!("into should ok for {:?}", indexs).as_str());

        assert_eq!(bytes.len(), 1 + 2);
        assert_eq!(
            bytes,
            vec![BasicOp::OutputIndexFlag16Assert as u8, o1, o2],
            "assert {:?}",
            indexs
        );
    }

    #[test]
    fn test_output_16_into_ops_bytes() -> Result<()> {
        output_index_flag16_assert_ops_bytes(vec![0, 1, 2, 3], 0b00001111, 0b00000000);
        output_index_flag16_assert_ops_bytes(vec![0, 1], 0b00000011, 0b00000000);
        output_index_flag16_assert_ops_bytes(vec![0, 1, 2, 3, 4, 5, 6, 7], 0b11111111, 0b00000000);
        output_index_flag16_assert_ops_bytes(
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
            0b11111111,
            0b00000001,
        );
        output_index_flag16_assert_ops_bytes((0..16).into_iter().collect(), 0b11111111, 0b11111111);
        output_index_flag16_assert_ops_bytes((0..8).into_iter().collect(), 0b11111111, 0b00000000);
        output_index_flag16_assert_ops_bytes((8..16).into_iter().collect(), 0b00000000, 0b11111111);
        output_index_flag16_assert_ops_bytes((7..16).into_iter().collect(), 0b10000000, 0b11111111);
        output_index_flag16_assert_ops_bytes(vec![14, 15], 0b00000000, 0b11000000);

        Ok(())
    }

    fn output_index_flag32_assert_ops_bytes(indexs: Vec<u8>, o: [u8; 4]) {
        let output = InstructionOutputAssert { indexs: indexs.clone() };

        let bytes = output
            .into_ops_bytes()
            .expect(format!("into should ok for {:?}", indexs).as_str());

        assert_eq!(bytes.len(), 1 + 4);
        assert_eq!(
            bytes,
            vec![BasicOp::OutputIndexFlag32Assert as u8, o[0], o[1], o[2], o[3]],
            "assert {:?}",
            indexs
        );
    }

    #[test]
    fn test_output_32_into_ops_bytes() -> Result<()> {
        output_index_flag32_assert_ops_bytes(
            vec![0, 31],
            [0b00000001, 0b00000000, 0b00000000, 0b10000000],
        );
        output_index_flag32_assert_ops_bytes(
            vec![15, 16],
            [0b00000000, 0b10000000, 0b00000001, 0b00000000],
        );
        output_index_flag32_assert_ops_bytes(
            vec![0, 7, 8, 15, 16, 23, 24, 31],
            [0b10000001, 0b10000001, 0b10000001, 0b10000001],
        );
        output_index_flag32_assert_ops_bytes(
            (0..32).into_iter().collect(),
            [0b11111111, 0b11111111, 0b11111111, 0b11111111],
        );

        Ok(())
    }
}
