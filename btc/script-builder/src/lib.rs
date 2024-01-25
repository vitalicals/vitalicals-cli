//! A Builder to make the script for inscribe.
//!
//! The script will like:
//!
//! OP_0
//! OP_IF
//! OP_PUSHBYTES_5 766974616c
//! OP_PUSHBYTES_XX datas1 or OP_PUSHDATA2 len datasXXX
//! OP_ENDIF

use anyhow::{bail, Context, Result};

use bdk::bitcoin::{
    opcodes::{
        all::{OP_CHECKSIG, OP_ENDIF, OP_IF},
        OP_0,
    },
    script::PushBytesBuf,
    secp256k1::XOnlyPublicKey,
    ScriptBuf,
};

use vital_primitives::consts::INSCRIBE_TAG;

const MAX_INSCRIPTION_DATAS_LEN: usize = 4096;

pub struct InscriptionScriptBuilder {
    datas: Vec<u8>,
}

impl InscriptionScriptBuilder {
    pub fn new(datas: Vec<u8>) -> Self {
        Self { datas }
    }

    pub fn into_script_by_key(self, key: &XOnlyPublicKey) -> Result<ScriptBuf> {
        if self.datas.len() >= MAX_INSCRIPTION_DATAS_LEN {
            bail!(
                "the inscription datas is too large expect {} got {}!",
                MAX_INSCRIPTION_DATAS_LEN,
                self.datas.len()
            )
        } else {
            let builder = ScriptBuf::builder()
                .push_x_only_key(key)
                .push_opcode(OP_CHECKSIG)
                .push_opcode(OP_0)
                .push_opcode(OP_IF)
                .push_slice(INSCRIBE_TAG);

            // Note the datas len must < MAX_INSCRIPTION_DATAS_LEN
            let mut buf = PushBytesBuf::with_capacity(self.datas.len());
            buf.extend_from_slice(&self.datas)?;

            let builder = builder.push_slice(buf).push_opcode(OP_ENDIF);
            let script = builder.into_script();

            Ok(script)
        }
    }

    pub fn into_script(self, key: &[u8]) -> Result<ScriptBuf> {
        let key = &XOnlyPublicKey::from_slice(key).context("key into public key")?;
        self.into_script_by_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gen_datas(l: usize) -> Vec<u8> {
        let mut res = Vec::with_capacity(l);
        for _ in 0..l {
            res.push(0xab);
        }
        res
    }

    fn test_datas_with_len(l: usize) {
        let datas = gen_datas(l);
        let script = InscriptionScriptBuilder::new(datas)
            .into_script(&hex_literal::hex!(
                "d2e7612f73d26067ae83e2a9d8bfa496193374677490dff0792242bbacba6922"
            ))
            .expect("datas");

        println!("script_{} {}", l, script);
        println!("script_{} {}", l, hex::encode(script.as_bytes()));
    }

    #[test]
    fn test_scripts() {
        test_datas_with_len(70);
        test_datas_with_len(72);
        test_datas_with_len(74);
        test_datas_with_len(75);
        test_datas_with_len(76);
        test_datas_with_len(100);
        test_datas_with_len(200);
        test_datas_with_len(255);
        test_datas_with_len(256);
        test_datas_with_len(257);
        test_datas_with_len(300);
        test_datas_with_len(500);
        test_datas_with_len(1000);
        test_datas_with_len(1022);
        test_datas_with_len(1023);

        test_datas_with_len(4000);
        test_datas_with_len(4095);
    }
}
