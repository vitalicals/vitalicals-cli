use anyhow::{bail, Result};
use bitcoin::Transaction;
use hex_literal::hex;

use alloc::vec::Vec;

// TODO: move to primitive types

/// The tag string for inscribe
pub const INSCRIBE_TAG_STR: &str = "vital";

/// The tag len for vital
pub const INSCRIBE_TAG_LEN: usize = INSCRIBE_TAG_STR.len();

/// The tag in script, just the Ascii (String) `vital` to hex
pub const INSCRIBE_TAG: [u8; INSCRIBE_TAG_LEN] = hex!("766974616c");

pub fn check_is_vital_script(tx: &Transaction) -> bool {
    // the input and output count should < u8::MAX
    if tx.input.len() >= u8::MAX as usize {
        return false;
    }

    if tx.output.len() >= u8::MAX as usize {
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
                        at_least_one_script = true;
                    }
                }
                Err(err) => {
                    log::debug!("parse error: {}", err);
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
                    log::debug!("parse error: {}", err);
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

    let script_len = script[n] as usize;
    n += 1;

    if script.len() != n + script_len + 1 {
        bail!("data len not match {}, {}", script.len(), n + script_len);
    }

    if script[n + script_len] != 0x68 {
        bail!("OP_ENDIF")
    }

    Ok(script[n..n + script_len].to_vec())
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

    #[test]
    fn test_check_is_vital_script() {
        let tx: Transaction = serde_json::from_str(MINT_TX).expect("from");
        assert!(check_is_vital_script(&tx), "check_is_vital_script is true")
    }

    #[test]
    fn test_check_is_vital_script_match() {
        let tx: Transaction = serde_json::from_str(MINT_TX).expect("from");
        let script = parse_vital_scripts(&tx).expect("should be vital script");
        assert_eq!(script.len(), 1);
        assert_eq!(script[0].0, 0);
        assert_eq!(script[0].1, hex::decode("0a00270420c41400").expect("hex"));
    }
}
