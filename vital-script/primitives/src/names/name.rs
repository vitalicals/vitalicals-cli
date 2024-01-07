//! The name type

use anyhow::{bail, Result};

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
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Name(pub [u8; NAME_BYTES_LEN]);

impl Name {
    pub fn new(v: [u8; NAME_BYTES_LEN]) -> Self {
        Self(v)
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
            for i in length..NAME_LEN_MAX + 1 {
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
    fn set_nocheck(&mut self, i: usize, c: char) -> Result<()> {
        let v = char2u8(c)?;

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

#[cfg(feature = "std")]
impl std::string::ToString for Name {
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
impl TryFrom<String> for Name {
    type Error = String;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        if value.len() > NAME_LEN_MAX {
            return Err("the string len too large".to_string());
        }

        let mut res = Name::default();

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
mod tests {
    #[cfg(feature = "std")]
    use crate::names::char2u8;

    #[cfg(feature = "std")]
    use super::Name;

    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
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
    #[cfg(feature = "std")]
    fn test_name_new_failed() {
        assert!(Name::try_from(" ".to_string()).is_err());
        assert!(Name::try_from("a a".to_string()).is_err());
        assert!(Name::try_from("(".to_string()).is_err());
        assert!(Name::try_from("aaaaaaaaaaa".to_string()).is_err());
        assert!(Name::try_from("aaaaaaaaaaaaaaaaaaaa".to_string()).is_err());
    }
}
