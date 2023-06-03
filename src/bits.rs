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

use crate::bits::Masks::{D_BITS, MOD_BITS};
use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement, RegisterMode};
use crate::is_word_size;

// InstructionTable contains all the possible instructions that we are trying to decode.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum InstructionType {
    RegisterMemory,
    ImmediateToRegisterMemory,
    ImmediateToRegisterMOV,
    ImmediateToAccumulatorADD,
    ImmediateToAccumulatorSUB,
    ImmediateToAccumulatorCMP,
    JE_JUMP,
    JL_JUMP,
    JLE_JUMP,
    JB_JUMP,
    JBE_JUMP,
    JP_JUMP,
    JO_JUMP,
    JS_JUMP,
    JNE_JUMP,
    JNL_JUMP,
    JNLE_JUMP,
    JNB_JUMP,
    JNBE_JUMP,
    JNP_JUMP,
    JNO_JUMP,
    JNS,
    LOOP,
    LOOPZ,
    LOOPNZ,
    JCXZ,
}

pub fn instruction_is_conditional_jump(instruction: InstructionType) -> bool {
    match instruction {
        InstructionType::JE_JUMP => true,
        InstructionType::JL_JUMP => true,
        InstructionType::JLE_JUMP => true,
        InstructionType::JB_JUMP => true,
        InstructionType::JBE_JUMP => true,
        InstructionType::JP_JUMP => true,
        InstructionType::JO_JUMP => true,
        InstructionType::JS_JUMP => true,
        InstructionType::JNE_JUMP => true,
        InstructionType::JNL_JUMP => true,
        InstructionType::JNLE_JUMP => true,
        InstructionType::JNB_JUMP => true,
        InstructionType::JNBE_JUMP => true,
        InstructionType::JNP_JUMP => true,
        InstructionType::JNO_JUMP => true,
        InstructionType::JNS => true,
        InstructionType::LOOP => true,
        InstructionType::LOOPZ => true,
        InstructionType::LOOPNZ => true,
        InstructionType::JCXZ => true,
        _ => false,
    }
}

pub fn instruction_is_immediate_to_register(instruction: InstructionType) -> bool {
    match instruction {
        InstructionType::ImmediateToRegisterMemory => true,
        InstructionType::ImmediateToRegisterMOV => true,
        InstructionType::ImmediateToAccumulatorADD => true,
        InstructionType::ImmediateToAccumulatorSUB => true,
        InstructionType::ImmediateToAccumulatorCMP => true,
        _ => false,
    }
}
// OpCode exists because we want each bit to know which InstructionType it belongs to.
// this is because we will be iterating and matching the bit patterns and if we match we want to
// immediately know which instruction type it is.

// Contains all the possible bit patterns for the first bytes of MOV, CMP, SUB and ADD register/to/from/memory operations.
const REGISTER_MEMORY_OPERATION: [u8; 20] = [
    0b10001011, 0b10001001, 0b10001010, 0b10001000, 0b00111000, 0b00111001, 0b00111010, 0b00111011,
    0b00101000, 0b00101001, 0b00101010, 0b00101011, 0b00101000, 0b00101001, 0b00101010, 0b00101011,
    0b00000000, 0b00000001, 0b00000010, 0b00000011
];

// Contains all the possible ImmediateToRegisterOrMemory patterns from the first byte for MOV, ADD, CMP, SUB.
// the bit pattern is the same for ADD, SUB, CMP but different for MOV.
// MOV = 110011{1/0}
// ADD, SUB, CMP = 100000{0/1,0/1}
const IMMEDIATE_TO_REGISTER_OR_MEMORY_ID: [u8; 7] = [
    0b11000111, 0b11000110, 0b10000000, 0b10000011, 0b10000001, 0b10000010, 0b10000000
];

// The mov immediate to register has a bit pattern of {1011{0/1,0/1,0/1,0/1}}
const IMMEDIATE_TO_REGISTER_MOV_ID: [u8; 16] = [
    0b10110000, 0b10110001, 0b10110010, 0b10110011, 0b10110100, 0b10110101, 0b10110110, 0b10110111,
    0b10111000, 0b10111001, 0b10111010, 0b10111011, 0b10111100, 0b10111101, 0b10111110, 0b10111111,
];

const IMMEDIATE_TO_ACCUMULATOR_ADD_ID: [u8; 2] = [
    0b00000100, 0b00000101
];

const IMMEDIATE_TO_ACCUMULATOR_SUB_ID: [u8; 2] = [
    0b00101100, 0b00101101
];

const IMMEDIATE_TO_ACCUMULATOR_CMP_ID: [u8; 2] = [
    0b00111100, 0b00111101
];

// NOTE: Jumps have a 8 bit displacement in the second byte.

pub struct OpCode {
    bit_pattern: u8,
    t: InstructionType,
}

// Constructs the OpCode structs into an array of OpCodes from all the const Instruction ID arrays,
// we do this instead of embedding the structs into the array directly because it looks dirty as hell.

// this is done because we want to determine the operation by looping over all the bit_patterns and on a match
// we will look at the InstructionType.
pub fn construct_opcodes() -> Vec<OpCode> {
    let elements_size: usize = REGISTER_MEMORY_OPERATION.len()
        + IMMEDIATE_TO_REGISTER_OR_MEMORY_ID.len()
        + IMMEDIATE_TO_REGISTER_MOV_ID.len()
        + 15; // 15 = jumps
    let mut op_codes: Vec<OpCode> = Vec::with_capacity(elements_size);

    for reg_memory in REGISTER_MEMORY_OPERATION {
        let op_code = OpCode {
            bit_pattern: reg_memory,
            t: InstructionType::RegisterMemory,
        };
        op_codes.push(op_code)
    }

    for imm_to_reg_or_memory in IMMEDIATE_TO_REGISTER_OR_MEMORY_ID {
        let op_code = OpCode {
            bit_pattern: imm_to_reg_or_memory,
            t: InstructionType::ImmediateToRegisterMemory,
        };
        op_codes.push(op_code)
    }

    for imm_to_reg_mov in IMMEDIATE_TO_REGISTER_MOV_ID {
        let op_code = OpCode {
            bit_pattern: imm_to_reg_mov,
            t: InstructionType::ImmediateToRegisterMOV,
        };
        op_codes.push(op_code)
    }

    for imm in IMMEDIATE_TO_ACCUMULATOR_ADD_ID {
        let op_code = OpCode {
            bit_pattern: imm,
            t: InstructionType::ImmediateToAccumulatorADD,
        };
        op_codes.push(op_code)
    }

    for imm in IMMEDIATE_TO_ACCUMULATOR_CMP_ID {
        let op_code = OpCode {
            bit_pattern: imm,
            t: InstructionType::ImmediateToAccumulatorCMP,
        };
        op_codes.push(op_code)
    }

    for imm in IMMEDIATE_TO_ACCUMULATOR_SUB_ID {
        let op_code = OpCode {
            bit_pattern: imm,
            t: InstructionType::ImmediateToAccumulatorSUB,
        };
        op_codes.push(op_code)
    }
    op_codes.push(OpCode{bit_pattern: 0b01110100, t: InstructionType::JE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111100, t: InstructionType::JL_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111110, t: InstructionType::JLE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110010, t: InstructionType::JB_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110110, t: InstructionType::JBE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111010, t: InstructionType::JP_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110000, t: InstructionType::JO_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111000, t: InstructionType::JS_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110101, t: InstructionType::JNE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111101, t: InstructionType::JNL_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111111, t: InstructionType::JNLE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110011, t: InstructionType::JNB_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110111, t: InstructionType::JNBE_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01111011, t: InstructionType::JNP_JUMP});
    op_codes.push(OpCode{bit_pattern: 0b01110001, t: InstructionType::JNO_JUMP});
    // Loop stuff
    op_codes.push(OpCode{bit_pattern: 0b01111001, t: InstructionType::JNS});
    op_codes.push(OpCode{bit_pattern: 0b11100010, t: InstructionType::LOOP});
    op_codes.push(OpCode{bit_pattern: 0b11100001, t: InstructionType::LOOPZ});
    op_codes.push(OpCode{bit_pattern: 0b11100000, t: InstructionType::LOOPNZ});
    op_codes.push(OpCode{bit_pattern: 0b11100011, t: InstructionType::JCXZ});

    op_codes
}
// We need to call this function because the different instructions are handled
// in different ways.
pub fn determine_instruction(op_codes: &Vec<OpCode>, first_byte: u8) -> InstructionType {
    for op_code in op_codes {
        if op_code.bit_pattern == first_byte {
            return op_code.t.clone();
        }
    }

    panic!("unsupported operation, first_byte: {:08b}", first_byte);
}


// MemoryMode is determined by the MOD field in the second byte.
// 00 = Memory Mode, no displacement
// 01 = Memory Mode, 8 bit displacement
// 10 = Memory Mode, 16 bit displacement

// 11 = Register Mode, no displacement, expect when R/M Field is 110.
// when MOD is 11 and R/M is 110, it means its a direct memory mode operation
// the direct memory is a 16 bit address.

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum MemoryModeEnum {
    MemoryModeNoDisplacement,
    MemoryMode8Bit,
    MemoryMode16Bit,
    RegisterMode,
    DirectMemoryOperation,
}
#[repr(u8)]
pub enum Masks {
    IMMEDIATE_TO_REG_MOV_W_BIT = 0b_00001000,
    MOD_BITS = 0b_11000000,
    W_BIT = 0b_00000001,
    RM_BITS = 0b_00000111,
    REG_BITS = 0b_00111000,
    D_BITS = 0b_00000010,
}
pub const S_BIT_M: Masks = D_BITS;
// It's the same bits but I want to express it in the name.
pub const IMMEDIATE_TO_MOV_REG_BITS: Masks = Masks::RM_BITS;


pub fn determine_memory_mode(second_byte: u8) -> MemoryModeEnum {
    let mod_field = second_byte & MOD_BITS as u8;
    match mod_field{
        0b_00000000 => {
            // So the rm_res determines if the memory mode with no displacement is actually
            // a 16-bit memory operation. Direct memory operation has R/M set to 110.
            let rm_res = second_byte & Masks::RM_BITS as u8;
            return if rm_res == 0b_00_000_110 {
                DirectMemoryOperation
            } else {
                MemoryModeNoDisplacement
            }
        }
        0b_01000000 => MemoryMode8Bit,
        0b_10000000 => MemoryMode16Bit,
        0b_11000000 => RegisterMode,
        _ => panic!("Invalid second_byte bit pattern, could not determine memory mode: {}", second_byte),
    }
}

pub fn determine_instruction_byte_size(inst: InstructionType, is_word_size: bool, memory_mode: MemoryModeEnum, mnemonic: &'static str, s_bit_set: bool) -> usize {
    match inst {
        InstructionType::RegisterMemory => {
            if memory_mode == MemoryModeNoDisplacement {
                return 2;
            } else if memory_mode == MemoryMode8Bit {
                return 3;
            } else if memory_mode == MemoryMode16Bit {
                return 4;
            } else if memory_mode == RegisterMode {
                return 2;
            } else if memory_mode == DirectMemoryOperation {
                return 4;
            } else {
                panic!("Unknown memory_mode operation. We did not expect to get here.\nmemory_mode: {:?}, inst: {:?}, is_word_size: {}", memory_mode, inst, is_word_size);
            }
        }
        InstructionType::ImmediateToRegisterMemory => {
            // TODO: handle direct memory operation.
            if mnemonic == "mov" {
                if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit || memory_mode == DirectMemoryOperation { // TODO does DirectMemoryOperation belong here?
                    if is_word_size {
                        return 6;
                    } else {
                        return 5;
                    }
                } else {
                    if is_word_size {
                        return 4;
                    } else {
                        return 3;
                    }
                }
            } else if mnemonic == "add" || mnemonic == "sub" || mnemonic == "cmp" {
                // add is 01 sw for 16-bit
                // this means that s bit has to be set to 0 if w is 1 for it to be 6 bytes wide.
                if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit || memory_mode == DirectMemoryOperation { // TODO does DirectMemoryOperation belong here?
                    if is_word_size && !s_bit_set {
                        return 6;
                    } else {
                        return 5;
                    }
                } else {
                    if is_word_size && !s_bit_set {
                        return 4;
                    } else {
                        return 3;
                    }
              }
            } else {
                panic!("Unknown mnemonic, we could not handle it. instruction type: {:?}, mnemonic: {}", inst, mnemonic)
            }
        }
        InstructionType::ImmediateToRegisterMOV | InstructionType::ImmediateToAccumulatorSUB | InstructionType::ImmediateToAccumulatorADD  | InstructionType::ImmediateToAccumulatorCMP => {
            if is_word_size {
                return 3;
            } else {
                return 2;
            }
        },
        InstructionType::JE_JUMP
        | InstructionType::JL_JUMP
        | InstructionType::JLE_JUMP
        | InstructionType::JB_JUMP
        | InstructionType::JBE_JUMP
        | InstructionType::JP_JUMP
        | InstructionType::JO_JUMP
        | InstructionType::JS_JUMP
        | InstructionType::JNE_JUMP
        | InstructionType::JNL_JUMP
        | InstructionType::JNLE_JUMP
        | InstructionType::JNB_JUMP
        | InstructionType::JNBE_JUMP
        | InstructionType::JNP_JUMP
        | InstructionType::JNO_JUMP
        | InstructionType::JNS
        | InstructionType::LOOP
        | InstructionType::LOOPZ
        | InstructionType::LOOPNZ
        | InstructionType::JCXZ => {
            return 2
        }
    }
}