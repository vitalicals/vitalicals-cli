use bytes::{Buf, Bytes};
use vital_script_primitives::{H256, U256};

pub const U32_SIZE: usize = u32::BITS as usize / 8;
pub const U64_SIZE: usize = u64::BITS as usize / 8;
pub const U128_SIZE: usize = u128::BITS as usize / 8;
pub const U256_SIZE: usize = 256_usize / 8;
pub const H256_SIZE: usize = 256_usize / 8;

pub fn u256_from_bytes(datas: &mut Bytes) -> U256 {
    let mut raw = [0_u8; U256_SIZE];
    datas.copy_to_slice(&mut raw);

    U256::from_big_endian(&raw)
}

pub fn h256_from_bytes(datas: &mut Bytes) -> H256 {
    let mut raw = [0_u8; H256_SIZE];
    datas.copy_to_slice(&mut raw);

    H256::from_slice(&raw)
}
