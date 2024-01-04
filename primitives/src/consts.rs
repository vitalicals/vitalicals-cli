use hex_literal::hex;

/// The tag string for inscribe
pub const INSCRIBE_TAG_STR: &str = "vital";

/// The tag len for vital
pub const INSCRIBE_TAG_LEN: usize = INSCRIBE_TAG_STR.len();

/// The tag in script, just the Ascii (String) `vital` to hex
pub const INSCRIBE_TAG: [u8; INSCRIBE_TAG_LEN] = hex!("766974616c");
