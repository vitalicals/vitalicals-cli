//! The name type
//!
//! Char map:
//!
//! a-z : 1 ~ 26
//! 0-9 : 27 ~ 36

mod name;
mod short_name;

pub use name::*;
pub use short_name::*;

use anyhow::{bail, Result};

const INVALID_VALUE: u8 = 0xFF;
const VALUE_MAX: u8 = 42;

fn char2u8(c: char) -> Result<u8> {
    match c {
        'a' => Ok(1),
        'b' => Ok(2),
        'c' => Ok(3),
        'd' => Ok(4),
        'e' => Ok(5),
        'f' => Ok(6),
        'g' => Ok(7),
        'h' => Ok(8),
        'i' => Ok(9),
        'j' => Ok(10),
        'k' => Ok(11),
        'l' => Ok(12),
        'm' => Ok(13),
        'n' => Ok(14),
        'o' => Ok(15),
        'p' => Ok(16),
        'q' => Ok(17),
        'r' => Ok(18),
        's' => Ok(19),
        't' => Ok(20),
        'u' => Ok(21),
        'v' => Ok(22),
        'w' => Ok(23),
        'x' => Ok(24),
        'y' => Ok(25),
        'z' => Ok(26),
        '0' => Ok(27),
        '1' => Ok(28),
        '2' => Ok(29),
        '3' => Ok(30),
        '4' => Ok(31),
        '5' => Ok(32),
        '6' => Ok(33),
        '7' => Ok(34),
        '8' => Ok(35),
        '9' => Ok(36),
        '@' => Ok(37),
        '.' => Ok(38),
        '_' => Ok(39),
        '-' => Ok(40),
        '!' => Ok(41),
        '*' => Ok(42),
        _ => bail!("Unsupported character"),
    }
}

#[cfg(feature = "std")]
fn u8_to_char(v: u8) -> Result<char> {
    if v == 0 || v > VALUE_MAX {
        bail!("Unsupported value");
    }

    const CHAR_DATA: [char; VALUE_MAX as usize + 1] = [
        ' ', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q',
        'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4', '5', '6', '7', '8',
        '9', '@', '.', '_', '-', '!', '*',
    ];

    Ok(CHAR_DATA[v as usize])
}
