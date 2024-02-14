//! The name type

use alloc::string::String;
use anyhow::{bail, Result};
use bytes::{Buf, Bytes};
use parity_scale_codec::{Decode, Encode};

use crate::names::*;

pub const NAME_LEN_MAX: usize = 10;
pub const NAME_BYTES_LEN: usize = 8;

/// The Short Name impl by a u64
///
/// a char need 6 bits, the len max is 10, and can use 4bit (max is 15) as length.
/// |      u8     |     u8      |     u8      |     u8       |      u8     |     u8      |     u8      |       u8    |
/// | 000000 - 00 | 0000 - 0000 | 00 - 000000 | 000000 - 00  | 0000 - 0000 | 00 - 000000 | 000000 - 00 | 0000 - 0000 |
/// |   0    |    1      |     2     |    3   |   4    |    5       |     6     |   7    |   8    |    9      |  len |
///
/// The len just for 0 - 3
#[derive(Default, Debug, PartialOrd, Ord, PartialEq, Eq, Clone, Copy)]
#[derive(Encode, Decode)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct Name(pub [u8; NAME_BYTES_LEN]);

#[cfg(feature = "serde")]
impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NameVisitor;

        impl<'de> serde::de::Visitor<'de> for NameVisitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("vital name resource")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Name::try_from(v).map_err(|err| E::custom(format!("name format error {}", err)))
            }
        }

        deserializer.deserialize_str(NameVisitor)
    }
}

impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let len = self.len();

        for i in 0..len {
            write!(f, "{}", u8_to_char(self.index_value(i)).expect("the value should be valid"))?;
        }

        Ok(())
    }
}

impl Name {
    pub const SIZE: usize = NAME_BYTES_LEN;

    pub fn new(v: [u8; NAME_BYTES_LEN]) -> Self {
        Self(v)
    }

    pub fn must_from(name_str: &str) -> Self {
        Self::try_from(name_str).unwrap_or_else(|_| panic!("the {} must be name format!", name_str))
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
        let length = self.len();

        // check if the length is valid
        if length != self.count() {
            return false;
        }

        for i in 0..length {
            let v = self.index_value(i);
            if v == 0 || v > VALUE_MAX {
                return false;
            }
        }

        if length < NAME_LEN_MAX {
            for i in length..NAME_LEN_MAX {
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

        if len >= NAME_LEN_MAX {
            bail!("index invalid");
        }

        self.set_len_nocheck(len + 1);
        self.set_nocheck(len, c)?;

        Ok(())
    }

    pub fn set(&mut self, i: usize, c: char) -> Result<()> {
        let len = self.len();
        if i > len || i >= NAME_LEN_MAX {
            bail!("index invalid");
        }

        if i == len {
            self.set_len_nocheck(i + 1);
        }

        self.set_nocheck(i, c)
    }

    #[inline]
    fn set_len_nocheck(&mut self, len: usize) {
        self.0[NAME_BYTES_LEN - 1] &= 0xf0;
        self.0[NAME_BYTES_LEN - 1] |= len as u8
    }

    #[inline]
    fn set_value_nocheck(&mut self, i: usize, v: u8) -> Result<()> {
        // from 0 - n, note every 4 chars will impl by 3 u8,
        // so we can got the `class` which from 0-3, means thich char
        // and got the `start` which means the index of 3 u8.
        //
        // For example:
        // for the char index k, the [class, start] will be:
        // 0 -> [0, 0]
        // 1 -> [1, 0]
        // 2 -> [2, 0]
        // 3 -> [3, 0]
        // 4 -> [0, 3]
        // 5 -> [1, 3]
        // 6 -> [2, 3]
        // 7 -> [3, 3]
        // 8 -> [0, 6] ...

        let class = i % 4;
        let start = (i / 4) * 3;

        match class {
            0 => {
                self.0[start] |= v << 2;
            }
            1 => {
                let pre2 = v >> 4;
                let post4 = (v & 0x0f) << 4;

                self.0[start] |= pre2;
                self.0[start + 1] |= post4;
            }
            2 => {
                let pre4 = v >> 2;
                let post2 = (v & 0x03) << 6;

                self.0[start + 1] |= pre4;
                self.0[start + 2] |= post2;
            }
            3 => {
                self.0[start + 2] |= v;
            }
            _ => {
                bail!("class invalid");
            }
        }

        Ok(())
    }

    #[inline]
    fn set_nocheck(&mut self, i: usize, c: char) -> Result<()> {
        self.set_value_nocheck(i, char2u8(c)?)
    }

    #[inline]
    pub fn len(&self) -> usize {
        (self.0[NAME_BYTES_LEN - 1] & 0x0f) as usize
    }

    pub fn is_empty(&self) -> bool {
        self.0[0] == 0
    }

    #[inline]
    pub fn count(&self) -> usize {
        for i in 0..NAME_LEN_MAX {
            if self.index_value(i) == 0 {
                return i;
            }
        }

        NAME_LEN_MAX
    }

    #[inline]
    pub fn index_value(&self, index: usize) -> u8 {
        if index >= NAME_LEN_MAX {
            return INVALID_VALUE;
        }

        // from 0 - n, note every 4 chars will impl by 3 u8,
        // so we can got the `class` which from 0-3, means thich char
        // and got the `start` which means the index of 3 u8.
        //
        // For example:
        // for the char index k, the [class, start] will be:
        // 0 -> [0, 0]
        // 1 -> [1, 0]
        // 2 -> [2, 0]
        // 3 -> [3, 0]
        // 4 -> [0, 3]
        // 5 -> [1, 3]
        // 6 -> [2, 3]
        // 7 -> [3, 3]
        // 8 -> [0, 6] ...

        let class = index % 4;
        let start = (index / 4) * 3;

        match class {
            0 => self.0[start] >> 2,
            1 => self.0[start + 1] >> 4 | ((self.0[start] & 0x03) << 4),
            2 => ((self.0[start + 1] & 0x0f) << 2) | (self.0[start + 2] >> 6),
            3 => self.0[start + 2] & 0x3f,
            _ => INVALID_VALUE,
        }
    }
}

impl TryFrom<&str> for Name {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> core::result::Result<Self, Self::Error> {
        if value.len() > NAME_LEN_MAX {
            bail!("the string len too large");
        }

        let mut res = Name::default();

        if value.is_empty() {
            return Ok(res);
        }

        for c in value.chars() {
            res.push(c)?;
        }

        if !res.is_valid() {
            bail!("the string not valid");
        }

        Ok(res)
    }
}

impl TryFrom<String> for Name {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > NAME_LEN_MAX {
            bail!("the string len too large");
        }

        let mut res = Name::default();

        if value.is_empty() {
            return Ok(res);
        }

        for c in value.chars() {
            res.push(c)?;
        }

        if !res.is_valid() {
            bail!("the string not valid");
        }

        Ok(res)
    }
}

impl From<ShortName> for Name {
    fn from(value: ShortName) -> Self {
        let mut res = Name::default();
        let mut l = 0;

        for i in 0..SHORT_NAME_LEN_MAX {
            let v = value.index_value(i);

            if v == 0 {
                break;
            } else {
                l += 1;
                res.set_value_nocheck(i, v).expect("set");
            }
        }

        res.set_len_nocheck(l);

        res
    }
}

#[cfg(test)]
mod tests {
    use crate::names::{char2u8, ShortName};

    use super::Name;

    fn test_name_new_by(name: &str) {
        let n = Name::try_from(name.to_string()).expect(format!("try from {}", name).as_str());

        println!(
            "name {:5} {:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            name, n.0[0], n.0[1], n.0[2], n.0[3], n.0[4], n.0[5], n.0[6], n.0[7]
        );

        let to_str = n.to_string();

        assert_eq!(name.to_string(), to_str);
        assert_eq!(n.len(), name.len());

        for (i, c) in name.char_indices() {
            assert_eq!(n.index_value(i), char2u8(c).expect("should ok"))
        }
    }

    #[test]
    fn test_name_new() {
        test_name_new_by("");
        test_name_new_by("a");
        test_name_new_by("b");
        test_name_new_by("z");
        test_name_new_by("1");
        test_name_new_by("2");

        test_name_new_by("abc");
        test_name_new_by("aaa");
        test_name_new_by("xxx");
        test_name_new_by("123");

        test_name_new_by("123--");
        test_name_new_by("123**");
        test_name_new_by("123aa");

        test_name_new_by("a");
        test_name_new_by("aa");
        test_name_new_by("aaa");
        test_name_new_by("aaaa");
        test_name_new_by("aaaaa");
        test_name_new_by("aaaaaa");
        test_name_new_by("aaaaaaa");
        test_name_new_by("aaaaaaaa");
        test_name_new_by("aaaaaaaaa");
        test_name_new_by("aaaaaaaaaa");

        test_name_new_by("abcdefghij");
        test_name_new_by("klmnopqrst");
        test_name_new_by("uvwxyz0123");
        test_name_new_by("4567890@-_");
        test_name_new_by("*!.");

        test_name_new_by("*");
        test_name_new_by("**");
        test_name_new_by("***");
        test_name_new_by("****");
        test_name_new_by("*****");
        test_name_new_by("******");
        test_name_new_by("*******");
        test_name_new_by("********");
        test_name_new_by("*********");
        test_name_new_by("**********");
    }

    #[test]
    fn test_name_new_failed() {
        assert!(Name::try_from(" ".to_string()).is_err());
        assert!(Name::try_from("a a".to_string()).is_err());
        assert!(Name::try_from("(".to_string()).is_err());
        assert!(Name::try_from("aaaaaaaaaaa".to_string()).is_err());
        assert!(Name::try_from("aaaaaaaaaaaaaaaaaaaa".to_string()).is_err());
    }

    #[test]
    fn test_name_from_short_name() {
        assert_eq!(Name::from(ShortName::try_from("".to_string()).unwrap()).to_string(), "");
        assert_eq!(Name::from(ShortName::try_from("a".to_string()).unwrap()).to_string(), "a");
        assert_eq!(Name::from(ShortName::try_from("b".to_string()).unwrap()).to_string(), "b");
        assert_eq!(Name::from(ShortName::try_from("22".to_string()).unwrap()).to_string(), "22");
        assert_eq!(Name::from(ShortName::try_from("222".to_string()).unwrap()).to_string(), "222");
        assert_eq!(Name::from(ShortName::try_from("333".to_string()).unwrap()).to_string(), "333");
        assert_eq!(
            Name::from(ShortName::try_from("....".to_string()).unwrap()).to_string(),
            "...."
        );
        assert_eq!(
            Name::from(ShortName::try_from("@@@@@".to_string()).unwrap()).to_string(),
            "@@@@@"
        );
        assert_eq!(
            Name::from(ShortName::try_from("abced".to_string()).unwrap()).to_string(),
            "abced"
        );
        assert_eq!(
            Name::from(ShortName::try_from("erc20".to_string()).unwrap()).to_string(),
            "erc20"
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_for_name() {
        use serde::{Deserialize, Serialize};

        let name = Name::try_from("abcedfg").unwrap();

        #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
        struct Obj {
            n: Name,
        }

        let datas: Obj = serde_json::from_str("{\"n\": \"abcedfg\"}").unwrap();

        println!("name : {:?}", datas);
        println!("name : {:?}", datas.n.to_string());

        assert_eq!(name, datas.n);

        let datas_str = serde_json::to_string_pretty(&datas).unwrap();
        println!("name data : {}", datas_str);

        let datas_de: Obj = serde_json::from_str(datas_str.as_str()).unwrap();

        assert_eq!(datas, datas_de);
    }
}
