// All the different instruction operations we're looking to handle at the moment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operation {
    REGISTER_MODE,                // no displacement
    IMMEDIATE_TO_REGISTER_MOV_8, // The first byte is set to 10110... and the instruction is 2 bytes wide. (last byte is the immediate)
    IMMEDIATE_TO_REGISTER_MOV_16, // The first byte is set to 10111... and the instruction is 3 bytes wide. (last 2 bytes is the immediate)
    //
    MEMORY_MODE_8_BIT_DISPLACEMENT,  // 8 bit displacement
    MEMORY_MODE_16_BIT_DISPLACEMENT, // 16 bit displacement
    MEMORY_MODE_NONE, // No displacement expect if R/M is 110, then it's 16 bit direct address.
    MEMORY_MODE_DIRECT, // This is mod 00 with r/m 110 16 bit displacement into a direct memory address

    // This is actually different from the first immediate to memory field because it relies on both first and second byte
    // to determine the mnemonic and size of the possible immediate.
    // If S = 1 && W = 1 then it's a 16-bit immediate, else 8-bit. -
    // We also have to check the first byte to determine the mnemonic because the add and mov instruction use the same reg bits acquired from the
    // second byte.
    // IMMEDIATE_TO_REGISTER_OR_MEMORY_16,
    // IMMEDIATE_TO_REGISTER_OR_MEMORY_8,
    IMMEDIATE_TO_MEMORY_8,
    IMMEDIATE_TO_MEMORY_16,
    IMMEDIATE_TO_REGISTER_8,
    IMMEDIATE_TO_REGISTER_16,
    UNSUPPORTED,
}
