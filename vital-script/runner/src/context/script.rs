use anyhow::{bail, Context, Result};
use bitcoin::{
    opcodes::all::{OP_PUSHBYTES_75, OP_PUSHDATA1, OP_PUSHDATA2},
    Transaction,
};
use hex_literal::hex;

use alloc::vec::Vec;

use crate::traits::EnvFunctions;

// TODO: move to primitive types

const TARGET: &str = "vital::check";

/// The tag string for inscribe
pub const INSCRIBE_TAG_STR: &str = "vital";

/// The tag len for vital
pub const INSCRIBE_TAG_LEN: usize = INSCRIBE_TAG_STR.len();

/// The tag in script, just the Ascii (String) `vital` to hex
pub const INSCRIBE_TAG: [u8; INSCRIBE_TAG_LEN] = hex!("766974616c");

pub fn maybe_vital_commit_tx_with_input_resource<F: EnvFunctions>(
    tx: &Transaction,
    interface: &F,
) -> Result<bool> {
    // if a tx use resource as its input, need storage.
    if !tx.output.iter().any(|output| output.script_pubkey.is_p2tr()) {
        return Ok(false);
    }

    if tx.input.len() <= 1 {
        return Ok(false);
    }

    for input in tx.input.iter() {
        let resource = interface
            .get_resources(&input.previous_output)
            .with_context(|| alloc::format!("get resources {}", input.previous_output))?;
        if let Some(_resource) = resource {
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn check_is_vital_script(tx: &Transaction) -> bool {
    // the input and output count should < u8::MAX
    if tx.input.len() >= u8::MAX as usize {
        log::debug!(target: TARGET, "no vital by input len too long: {}", tx.input.len());
        return false;
    }

    if tx.output.len() >= u8::MAX as usize {
        log::debug!(target: TARGET, "no vital by output len too long: {}", tx.output.len());
        return false;
    }

    let mut at_least_one_script = false;

    for input in tx.input.iter() {
        let witness = &input.witness;

        if let Some(script) = witness.tapscript() {
            let script_bytes = script.to_bytes();
            match try_get_vital_script(&script_bytes) {
                Ok(s) => {
                    if !s.is_empty() {
                        log::debug!(target: TARGET, "is vital script {}", input.previous_output);
                        at_least_one_script = true;
                    } else {
                        log::debug!(target: TARGET, "no vital script bytes len zero {}", input.previous_output);
                    }
                }
                Err(err) => {
                    log::debug!(target: TARGET, "parse error: {}", err);
                }
            }
        }
    }

    at_least_one_script
}

pub fn parse_vital_scripts(tx: &Transaction) -> Result<Vec<(u8, Vec<u8>)>> {
    // the input and output count should < u8::MAX
    if tx.input.len() >= u8::MAX as usize {
        bail!("input too large");
    }

    if tx.output.len() >= u8::MAX as usize {
        bail!("output too large");
    }

    let mut res = Vec::with_capacity(tx.input.len());

    for (input_index, input) in tx.input.iter().enumerate() {
        let witness = &input.witness;

        if let Some(script) = witness.tapscript() {
            let script_bytes = script.to_bytes();
            match try_get_vital_script(&script_bytes) {
                Ok(s) => {
                    if !s.is_empty() {
                        res.push((input_index as u8, s));
                    }
                }
                Err(err) => {
                    log::debug!(target: TARGET, "parse error: {}", err);
                }
            }
        }
    }

    Ok(res)
}

const VITAL_SCRIPT_MIN_LEN: usize = 40 + 1 + 1;

pub fn try_get_vital_script(script: &[u8]) -> Result<Vec<u8>> {
    // 20d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922ac006305766974616c080a00270420c4140068
    // 20: OP_PUSHBYTES_32
    // d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922: key [u8; 32]
    // ac: OP_CHECKSIG
    // 00: OP_0
    // 63: OP_IF
    // 05: OP_PUSHBYTES_5
    // 766974616c: fixed
    // 0x: OP_PUSHBYTES_XX
    // datas
    // 68: OP_ENDIF

    // TODO: support multiple push bytes

    if script.len() <= VITAL_SCRIPT_MIN_LEN {
        bail!("script len min");
    }

    let mut n = 0;

    if script[n] != 0x20 {
        bail!("first should be OP_PUSHBYTES_32");
    }

    n += 1;

    n += 33;
    if script[n] != 0x00 {
        bail!("OP_0");
    }

    n += 1;
    if script[n] != 0x63 {
        bail!("OP_IF");
    }

    n += 1;
    if script[n] != 0x05 {
        bail!("OP_PUSHBYTES_5");
    }

    n += 1;
    if script[n..n + INSCRIBE_TAG_LEN].to_vec() != INSCRIBE_TAG.to_vec() {
        bail!("vital constants");
    }

    n += INSCRIBE_TAG_LEN;

    let script_bytes = try_get_script_bytes(&script[n..]).context("try_get_script_bytes")?;

    Ok(script_bytes)
}

fn try_get_script_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    let push_op = bytes[0];
    if push_op <= OP_PUSHBYTES_75.to_u8() {
        // use OP_PUSHBYTES_XX, the op is eq the len
        let script_len = push_op as usize;
        // the bytes will be [OP_PUSHBYTES_XX, [script_bytes..], OP_ENDIF]
        if bytes.len() != 1 + script_len + 1 {
            bail!("data len not match {}, {}", bytes.len(), 1 + script_len);
        }

        if bytes[1 + script_len] != 0x68 {
            bail!("OP_ENDIF")
        }

        return Ok(bytes[1..=script_len].to_vec())
    }

    if push_op == OP_PUSHDATA1.to_u8() {
        // use OP_PUSHDATA1(0x4c), the bytes will be:
        // [OP_PUSHDATA1(0x4c), Len(u8), [script_bytes..], OP_ENDIF]
        let script_len = bytes[1] as usize;

        let expect_len = 1 + 1 + script_len + 1;
        if bytes.len() != expect_len {
            bail!("data len not match {}, {}", bytes.len(), expect_len);
        }

        if bytes[1 + 1 + script_len] != 0x68 {
            bail!("OP_ENDIF")
        }

        return Ok(bytes[2..=(1 + script_len)].to_vec())
    }

    if push_op == OP_PUSHDATA2.to_u8() {
        // use OP_PUSHDATA2(0x4d), the bytes will be:
        // [OP_PUSHDATA1(0x4d), Len([u8[2], u8[1]]), [script_bytes..], OP_ENDIF]
        let script_len = u16::from_le_bytes([bytes[1], bytes[2]]) as usize;

        let expect_len = 1 + 2 + script_len + 1;
        if bytes.len() != expect_len {
            bail!("data len not match {}, {}", bytes.len(), expect_len);
        }

        if bytes[1 + 2 + script_len] != 0x68 {
            bail!("OP_ENDIF")
        }

        return Ok(bytes[(1 + 2)..=(2 + script_len)].to_vec())
    }

    bail!("currently not support {}", push_op);
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINT_TX: &str = "{
        \"version\": 1,
        \"lock_time\": 0,
        \"input\": [
          {
            \"previous_output\": \"e2adb8e239191c8b3b9ecdc52a6f04fcee3467387debf74355d981aecc27c8eb:1\",
            \"script_sig\": \"\",
            \"sequence\": 4294967293,
            \"witness\": [
              \"694d2e8aae72be30ab2f68485c4616ffe87d03a35e848248c13bfeb67a35ccb12e0f07d10eafa6ba8890ca984de4ff182c37d0554dd8f2deaf327defc7f8afec01\",
              \"20d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922ac006305766974616c080a00270420c4140068\",
              \"c0d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922\"
            ]
          }
        ],
        \"output\": [
          {
            \"value\": 1000,
            \"script_pubkey\": \"51208de147fc78c74363ad51d75b9f6e2ee82ff11f35b46416e075b37c4d3ed39bf5\"
          }
        ]
      }";

    const MINT_TX2: &str = "{
        \"version\": 1,
        \"lock_time\": 0,
        \"input\": [
          {
            \"previous_output\": \"e2adb8e239191c8b3b9ecdc52a6f04fcee3467387debf74355d981aecc27c8eb:1\",
            \"script_sig\": \"\",
            \"sequence\": 4294967293,
            \"witness\": [
              \"7f55716ac0527032967ebe766552f2976e4a07a4f6eba27fead625b3987a62eadc0ad2bea97fc60b3d258633a5e49bc7ca4b1d6cfe1174699386d786edfc3fb101\",
              \"20d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922ac006305766974616c4c850b030010e8030000154200000010e8030000154200000110e8030000154200000210e8030000154200000310e8030000154200000410e8030000154200000510e8030000154200000610e8030000154200000710e8030000154200000810e8030000154200000910e8030000154200000a1e1542000036290000001e15420000c20100000168\",
              \"c1d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922\"
            ]
          }
        ],
        \"output\": [
          {
            \"value\": 1000,
            \"script_pubkey\": \"51208de147fc78c74363ad51d75b9f6e2ee82ff11f35b46416e075b37c4d3ed39bf5\"
          }
        ]
      }";

    const MINT_TX3: &str = "{
        \"version\": 1,
        \"lock_time\": 0,
        \"input\": [
          {
            \"previous_output\": \"e2adb8e239191c8b3b9ecdc52a6f04fcee3467387debf74355d981aecc27c8eb:1\",
            \"script_sig\": \"\",
            \"sequence\": 4294967293,
            \"witness\": [
              \"18b699622ed7f091866742470d1a54bb739e2327e7821cbe7ffd93b0868c46fa92fe1e32498131ae599e46cf764f58d8296a0720a8986884e57b04784b6bec7d01\",
              \"20d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922ac006305766974616c4d25010b030010c2010000154200000010e8030000154200000110e8030000154200000210e8030000154200000310e8030000154200000410e8030000154200000510e8030000154200000610e8030000154200000710e8030000154200000810e8030000154200000910e8030000154200000a10e8030000154200000b10e8030000154200000c10e8030000154200000d10e8030000154200000e10e8030000154200000f10e8030000154200001010e8030000154200001110e8030000154200001210e8030000154200001310e8030000154200001410e8030000154200001510e8030000154200001610e8030000154200001710e8030000154200001810e803000015420000191036290000154200001a1e154200009f8c0000001e15420000010000000168\",
              \"c1d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922\"
            ]
          }
        ],
        \"output\": [
          {
            \"value\": 1000,
            \"script_pubkey\": \"51208de147fc78c74363ad51d75b9f6e2ee82ff11f35b46416e075b37c4d3ed39bf5\"
          }
        ]
      }";

    #[test]
    fn test_check_is_vital_script() {
        // init_logger();

        let tx: Transaction = serde_json::from_str(MINT_TX).expect("from");
        assert!(check_is_vital_script(&tx), "check_is_vital_script is true");
    }

    #[test]
    fn test_check_is_vital_script_match() {
        // init_logger();

        let tx: Transaction = serde_json::from_str(MINT_TX).expect("from");
        let script = parse_vital_scripts(&tx).expect("should be vital script");
        assert_eq!(script.len(), 1);
        assert_eq!(script[0].0, 0);
        assert_eq!(script[0].1, hex::decode("0a00270420c41400").expect("hex"));
    }

    #[test]
    fn test_check_is_vital_script_for_push_data1() {
        // init_logger();

        let tx: Transaction = serde_json::from_str(MINT_TX2).expect("from");
        assert!(check_is_vital_script(&tx), "check_is_vital_script is true");
    }

    #[test]
    fn test_check_is_vital_script_match_for_push_data1() {
        let tx: Transaction = serde_json::from_str(MINT_TX2).expect("from");
        let script = parse_vital_scripts(&tx).expect("should be vital script");
        assert_eq!(script.len(), 1);
        assert_eq!(script[0].0, 0);
        assert_eq!(script[0].1, hex::decode("0b030010e8030000154200000010e8030000154200000110e8030000154200000210e8030000154200000310e8030000154200000410e8030000154200000510e8030000154200000610e8030000154200000710e8030000154200000810e8030000154200000910e8030000154200000a1e1542000036290000001e15420000c201000001").expect("hex"));
    }

    #[test]
    fn test_check_is_vital_script_for_push_data2() {
        // init_logger();

        let tx: Transaction = serde_json::from_str(MINT_TX3).expect("from");
        assert!(check_is_vital_script(&tx), "check_is_vital_script is true");
    }

    #[test]
    fn test_check_is_vital_script_match_for_push_data2() {
        let tx: Transaction = serde_json::from_str(MINT_TX3).expect("from");
        let script = parse_vital_scripts(&tx).expect("should be vital script");
        assert_eq!(script.len(), 1);
        assert_eq!(script[0].0, 0);
        assert_eq!(script[0].1, hex::decode("0b030010c2010000154200000010e8030000154200000110e8030000154200000210e8030000154200000310e8030000154200000410e8030000154200000510e8030000154200000610e8030000154200000710e8030000154200000810e8030000154200000910e8030000154200000a10e8030000154200000b10e8030000154200000c10e8030000154200000d10e8030000154200000e10e8030000154200000f10e8030000154200001010e8030000154200001110e8030000154200001210e8030000154200001310e8030000154200001410e8030000154200001510e8030000154200001610e8030000154200001710e8030000154200001810e803000015420000191036290000154200001a1e154200009f8c0000001e154200000100000001").expect("hex"));
    }
}
