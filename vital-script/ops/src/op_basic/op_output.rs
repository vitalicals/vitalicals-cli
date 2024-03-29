use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};

use vital_script_derive::BasicOpcode;

use crate::instruction::{assert_output::InstructionOutputAssert, Instruction};

#[inline]
fn u8_to_pos(i: u8, c: u8) -> Vec<u8> {
    let mut res = Vec::new();

    for pos in 0..8 {
        let mask = 1 << pos;
        if (mask & i) != 0 {
            res.push(8 * c + pos);
        }
    }

    res
}

/// Output Index Assert by 1 indexs
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexAssert {
    pub index: u8,
}

impl From<OutputIndexAssert> for Instruction {
    fn from(value: OutputIndexAssert) -> Self {
        Instruction::Output(InstructionOutputAssert { indexs: [value.index].to_vec() })
    }
}

/// Output Index Assert By A u16 as FlagMask
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexFlag16Assert {
    pub index_flag: [u8; 2],
}

impl From<OutputIndexFlag16Assert> for Instruction {
    fn from(value: OutputIndexFlag16Assert) -> Self {
        let indexs =
            [u8_to_pos(value.index_flag[0], 0), u8_to_pos(value.index_flag[1], 1)].concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}

/// Output Index Assert By A u32 as FlagMask
#[derive(Debug, BasicOpcode, Encode, Decode)]
pub struct OutputIndexFlag32Assert {
    pub index_flag: [u8; 4],
}

impl From<OutputIndexFlag32Assert> for Instruction {
    fn from(value: OutputIndexFlag32Assert) -> Self {
        let indexs = [0_u8, 1, 2, 3].map(|c| u8_to_pos(value.index_flag[c as usize], c)).concat();

        Instruction::Output(InstructionOutputAssert { indexs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use anyhow::Result;

    use vital_script_ops::instruction::Instruction;
    use vital_script_primitives::resources::{Name, Resource};
    use vital_script_runner::mock::*;

    use crate::op_basic::tests::check_ops_encode_and_decode;

    #[test]
    fn test_u8_to_pos() {
        assert!(u8_to_pos(0, 0).is_empty());
        assert_eq!(u8_to_pos(1, 0), vec![0]);
        assert_eq!(u8_to_pos(2, 0), vec![1]);
        assert_eq!(u8_to_pos(3, 0), vec![0, 1]);
        assert_eq!(u8_to_pos(4, 0), vec![2]);
        assert_eq!(u8_to_pos(5, 0), vec![0, 2]);
        assert_eq!(u8_to_pos(6, 0), vec![1, 2]);
        assert_eq!(u8_to_pos(7, 0), vec![0, 1, 2]);

        assert_eq!(u8_to_pos(10, 0), vec![1, 3]);
        assert_eq!(u8_to_pos(24, 0), vec![3, 4]);
        assert_eq!(u8_to_pos(25, 0), vec![0, 3, 4]);
        assert_eq!(u8_to_pos(153, 0), vec![0, 3, 4, 7]);
        assert_eq!(u8_to_pos(240, 0), vec![4, 5, 6, 7]);

        assert_eq!(u8_to_pos(0xf0, 0), vec![4, 5, 6, 7]);
        assert_eq!(u8_to_pos(0x0f, 0), vec![0, 1, 2, 3]);
        assert_eq!(u8_to_pos(0xff, 0), vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(u8_to_pos(0xf0, 1), vec![12, 13, 14, 15]);
        assert_eq!(u8_to_pos(0x0f, 1), vec![8, 9, 10, 11]);
        assert_eq!(u8_to_pos(0xff, 1), vec![8, 9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_move_name_ops_encode_and_decode() {
        check_ops_encode_and_decode(OutputIndexAssert { index: 0 });
        check_ops_encode_and_decode(OutputIndexAssert { index: 1 });
        check_ops_encode_and_decode(OutputIndexAssert { index: 31 });
        check_ops_encode_and_decode(OutputIndexAssert { index: 16 });

        check_ops_encode_and_decode(OutputIndexFlag16Assert {
            index_flag: [0b00000001, 0b10000000],
        });
        check_ops_encode_and_decode(OutputIndexFlag16Assert {
            index_flag: [0b00111100, 0b10101010],
        });
        check_ops_encode_and_decode(OutputIndexFlag16Assert {
            index_flag: [0b11111111, 0b11111111],
        });

        check_ops_encode_and_decode(OutputIndexFlag32Assert {
            index_flag: [0b00000001, 0b00000000, 0b00000000, 0b10000000],
        });
        check_ops_encode_and_decode(OutputIndexFlag32Assert {
            index_flag: [0b00111100, 0b10101010, 0b10101010, 0b10101010],
        });
        check_ops_encode_and_decode(OutputIndexFlag32Assert {
            index_flag: [0b11111111, 0b11111111, 0b11111111, 0b11111111],
        });
    }

    #[test]
    fn test_no_output_assert_should_failed() -> Result<()> {
        let env_interface = EnvMock::new();

        let name1 = Name::must_from("abcde");
        let name_res1 = Resource::name(name1);

        // 1. the `abcde` had mint, so this will failed
        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![Instruction::mint(0, name_res1.resource_type())])
            .with_ops()
            .with_output(1000)
            .run();

        assert_err_str(res, "the output is not asserted", "1");

        Ok(())
    }
}
