//! The id def for opcode

#[repr(u8)]
pub enum BasicOp {
    OutputIndexAssert = 0x0a,
    OutputIndexFlag16Assert = 0x0b,
    OutputIndexFlag32Assert = 0x0c,

    InputVRC20AssertSa32 = 0x0d,
    InputVRC20AssertSa64 = 0x0e,
    InputVRC20AssertSa128 = 0x0f,
    InputVRC20AssertSa256 = 0x10,

    InputVRC20AssertA32 = 0x11,
    InputVRC20AssertA64 = 0x12,
    InputVRC20AssertA128 = 0x13,
    InputVRC20AssertA256 = 0x14,

    InputVRC721Assert = 0x15,

    TransferAllVRC20S = 0x16,
    TransferAllVRC20 = 0x17,

    TransferVRC20Sa32 = 0x18,
    TransferVRC20A32 = 0x19,
    // Mint
    // Burn
}
