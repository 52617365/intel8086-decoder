// 02.05.2023 what is all this InstructionTable stuff and ID stuff?
// We have a table containing all the instructions we want to handle (InstructionTable),
// on top of this, we have arrays that contain bit patterns from the first byte
// which determines which instruction the byte sequence belongs to.

// It turns out, we can get a pretty good understanding of the instruction
// from the first byte only, for some instructions, we need to look at the second byte,
// but it's okay, because at that point we have narrowed down the possibilities by a long shot
// resulting in cleaner code since all the logic is not pushed into one large function.
// This was my pitfall last time and I refuse to quit on this challenge.

// I'm trying to follow a similar approach to Casey Muratori, where he first did a
// Lexical analyzer type of phase to get tokens out of the bit patterns.

// InstructionTable contains all the possible instructions that we are trying to decode.
#[derive(PartialEq)]
pub enum InstructionType {
    RegisterMemory,
    ImmediateToRegisterMemory,
    ImmediateToRegisterMOV,
}

// OpCode exists because we want each bit to know which InstructionType it belongs to.
// this is because we will be iterating and matching the bit patterns and if we match we want to
// immediately know which instruction type it is.

// Contains all the possible bit patterns for the first bytes of MOV, CMP and ADD register/to/from/memory operations.
const RegisterMemoryOperation: [u8; 12] = [
    0b10001011, 0b10001001, 0b10001010, 0b10001000, 0b00111000, 0b00111001, 0b00111010, 0b00111011,
    0b00101000, 0b00101001, 0b00101010, 0b00101011,
];

// Contains all the possible ImmediateToRegisterOrMemory patterns from the first byte for MOV, ADD, CMP, SUB.
// the bit pattern is the same for ADD, SUB, CMP but different for MOV.
// MOV = 110011{1/0}
// ADD, SUB, CMP = 100000{0/1,0/1}
const ImmediateToRegisterOrMemoryID: [u8; 7] = [
    0b11000111, 0b11000110, 0b10000000, 0b10000011, 0b10000001, 0b10000010, 0b10000000,
];

// The mov immediate to register has a bit pattern of {1011{0/1,0/1,0/1,0/1}}
const ImmediateToRegisterMovID: [u8; 16] = [
    0b10110000, 0b10110001, 0b10110010, 0b10110011, 0b10110100, 0b10110101, 0b10110110, 0b10110111,
    0b10111000, 0b10111001, 0b10111010, 0b10111011, 0b10111100, 0b10111101, 0b10111110, 0b10111111,
];

//

struct OpCode {
    bit_pattern: u8,
    t: InstructionType,
}

// Constructs the OpCode structs into an array of OpCodes from all the const Instruction ID arrays,
// we do this instead of embedding the structs into the array directly because it looks dirty as hell.

// this is done because we want to determine the operation by looping over all the bit_patterns and on a match
// we will look at the InstructionType.
pub fn construct_opcodes() -> Vec<OpCode> {
    let elements_size: usize = RegisterMemoryOperation.len()
        + ImmediateToRegisterOrMemoryID.len()
        + ImmediateToRegisterMovID.len();
    let mut op_codes: Vec<OpCode> = Vec::with_capacity(elements_size);

    for reg_memory in RegisterMemoryOperation {
        let op_code = OpCode {
            bit_pattern: reg_memory,
            t: InstructionType::RegisterMemory,
        };
        op_codes.push(op_code)
    }

    for imm_to_reg_or_memory in ImmediateToRegisterOrMemoryID {
        let op_code = OpCode {
            bit_pattern: imm_to_reg_or_memory,
            t: InstructionType::ImmediateToRegisterMemory,
        };
        op_codes.push(op_code)
    }

    for imm_to_reg_mov in ImmediateToRegisterMovID {
        let op_code = OpCode {
            bit_pattern: imm_to_reg_mov,
            t: InstructionType::ImmediateToRegisterMOV,
        };
        op_codes.push(op_code)
    }
    op_codes
}

// We need to call this function because the different instructions are handled
// in different ways.
pub fn determine_instruction(op_codes: &Vec<OpCode>, first_byte: u8) -> InstructionType {
    for op_code in op_codes {
        if op_code.bit_pattern == first_byte {
            return op_code.t;
        }
    }
    panic!("unsupported operation {}", first_byte);
}

#[repr(u8)]
pub enum Masks {
    IMMEDIATE_TO_REG_MOV_W_BIT = 0b_00001000,
    W_BIT = 0b_00000001,
    RM_BITS = 0b_00000111,
}
