//! The short name type

use anyhow::{bail, Result};
use bytes::{Buf, Bytes};
use parity_scale_codec::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::names::*;

pub const SHORT_NAME_LEN_MAX: usize = 5;

/// The Short Name impl by a u32
///
/// The short name 's length <= 5.
/// |      u8     |     u8      |     u8      |     u8       |
/// | 000000 - 00 | 0000 - 0000 | 00 - 000000 | 000000 - 00  |
/// |   0    |    1      |     2     |    3   |   4    | len |
///
/// The len just for 0 - 3
#[derive(Debug, Default, Deserialize, Serialize)]
#[derive(Encode, Decode)]
pub struct ShortName(pub [u8; 4]);

impl ShortName {
    pub const SIZE: usize = 4;

    pub fn new(v: [u8; Self::SIZE]) -> Self {
        Self(v)
    }

    pub fn from_bytes(datas: &mut Bytes) -> Result<Self> {
        if datas.remaining() < Self::SIZE {
            bail!("ShortName bytes not enough");
        }

        let mut v = [0_u8; Self::SIZE];

        datas.copy_to_slice(&mut v);

        Ok(Self::new(v))
    }

    pub fn is_valid(&self) -> bool {
        // The u8[3] ls x is 00
        if self.0[3] & 0xfc != 0 {
            return false;
        }

        let length = self.len();

        for i in 0..length {
            let v = self.index_value(i);
            if v == 0 || v > VALUE_MAX {
                return false;
            }
        }

        if length < SHORT_NAME_LEN_MAX {
            for i in length..SHORT_NAME_LEN_MAX + 1 {
                let v = self.index_value(i);
                if v != 0 {
                    return false;
                }
            }
        }

        true
    }

    pub fn push(&mut self, c: char) -> Result<()> {
        let len = self.len();

        if len >= SHORT_NAME_LEN_MAX {
            bail!("index invalid");
        }

        self.set_nocheck(len, c)
    }

    pub fn set(&mut self, i: usize, c: char) -> Result<()> {
        let len = self.len();
        if i > len || i >= SHORT_NAME_LEN_MAX {
            bail!("index invalid");
        }

        self.set_nocheck(i, c)
    }

    fn set_nocheck(&mut self, i: usize, c: char) -> Result<()> {
        let v = char2u8(c)?;

        match i {
            0 => {
                self.0[0] |= v << 2;
            }
            1 => {
                let pre2 = v >> 4;
                let post4 = (v & 0x0f) << 4;

                self.0[0] |= pre2;
                self.0[1] |= post4;
            }
            2 => {
                let pre4 = v >> 2;
                let post2 = (v & 0x03) << 6;

                self.0[1] |= pre4;
                self.0[2] |= post2;
            }
            3 => {
                self.0[2] |= v;
            }
            4 => {
                let p = v << 2;
                self.0[3] |= p;
            }
            _ => {
                bail!("the short name is too long after push");
            }
        }

        Ok(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        for i in 0..SHORT_NAME_LEN_MAX {
            if self.index_value(i) == 0 {
                return i;
            }
        }

        SHORT_NAME_LEN_MAX
    }

    pub fn is_empty(&self) -> bool {
        self.0[0] == 0
    }

    #[inline]
    pub fn index_value(&self, index: usize) -> u8 {
        match index {
            0 => self.0[0] >> 2,
            1 => self.0[1] >> 4 | ((self.0[0] & 0x03) << 4),
            2 => ((self.0[1] & 0x0f) << 2) | (self.0[2] >> 6),
            3 => self.0[2] & 0x3f,
            4 => self.0[3] >> 2,
            _ => INVALID_VALUE,
        }
    }
}

impl TryFrom<Name> for ShortName {
    type Error = anyhow::Error;

    fn try_from(value: Name) -> Result<Self> {
        let len = value.len();
        if len > SHORT_NAME_LEN_MAX {
            bail!("the name is too long")
        }

        let mut res = ShortName::default();
        for i in 0..SHORT_NAME_LEN_MAX {
            let v = value.index_value(i);
            if v == 0 {
                break;
            } else {
                res.push(u8_to_char(v).expect("should valid")).expect("short should ok");
            }
        }

        Ok(res)
    }
}

#[cfg(feature = "std")]
impl std::string::ToString for ShortName {
    fn to_string(&self) -> String {
        let len = self.len();
        let mut res = String::with_capacity(len + 1);

        for i in 0..len {
            res.push(u8_to_char(self.index_value(i)).expect("the value should be valid"))
        }

        res
    }
}

#[cfg(feature = "std")]
impl TryFrom<String> for ShortName {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.len() > SHORT_NAME_LEN_MAX {
            return Err("the string len too large".to_string());
        }

        let mut res = ShortName::default();

        if value.is_empty() {
            return Ok(res);
        }

        for c in value.chars() {
            res.push(c).map_err(|err| err.to_string())?;
        }

        Ok(res)
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use crate::names::char2u8;

    use super::ShortName;

    fn test_short_name_new_by(name: &str) {
        let n = ShortName::try_from(name.to_string()).expect(format!("try from {}", name).as_str());

        // println!("short name {:5} {:02x}-{:02x}-{:02x}-{:02x}", name, n.0[0], n.0[1], n.0[2], n.0[3]);

        let to_str = n.to_string();

        assert_eq!(name.to_string(), to_str);
        assert_eq!(n.len(), name.len());

        for (i, c) in name.char_indices() {
            assert_eq!(n.index_value(i), char2u8(c).expect("should ok"))
        }
    }

    #[test]
    fn test_short_name_new() {
        test_short_name_new_by("");
        test_short_name_new_by("a");
        test_short_name_new_by("b");
        test_short_name_new_by("z");
        test_short_name_new_by("1");
        test_short_name_new_by("2");

        test_short_name_new_by("abc");
        test_short_name_new_by("aaa");
        test_short_name_new_by("xxx");
        test_short_name_new_by("123");

        test_short_name_new_by("123--");
        test_short_name_new_by("123**");
        test_short_name_new_by("123aa");

        test_short_name_new_by("a");
        test_short_name_new_by("aa");
        test_short_name_new_by("aaa");
        test_short_name_new_by("aaaa");
        test_short_name_new_by("aaaaa");

        test_short_name_new_by("abcde");
        test_short_name_new_by("fghij");
        test_short_name_new_by("klmno");
        test_short_name_new_by("pqrst");
        test_short_name_new_by("uvwxy");
        test_short_name_new_by("z0123");
        test_short_name_new_by("45678");
        test_short_name_new_by("90@-_");
        test_short_name_new_by("*!.");
    }

    #[test]
    fn test_short_name_new_failed() {
        assert!(ShortName::try_from(" ".to_string()).is_err());
        assert!(ShortName::try_from("a a".to_string()).is_err());
        assert!(ShortName::try_from("(".to_string()).is_err());
        assert!(ShortName::try_from("aaaaaa".to_string()).is_err());
        assert!(ShortName::try_from("aaaaaaaaaaaaaaaaaaaa".to_string()).is_err());
    }
}
