mod bits;
mod operation;
use bits::*;

use core::panic;
use std::{env, fs};

use crate::bits::MemoryMode::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement};

/*
    TODO: we have to take into consideration the s bit in the first byte.
    As it turns out, it's actually required in the immediate to register/memory operations
    to know if the operation is actually moving a 8 or 16-bit immediate value into a register/memory.

    As of currently, we are not taking the s bit into consideration anywhere.

    its always set to 1 with the immediate to register/memory MOV instruction.
    Normally it's not hardcoded so with the mov immediate instruction we just have to check the w bit to know
    if its a 8 or 16-bit operation but with the others we have to check both the w and s bits.


    TODO:
    Figure out what the s bit actually means because with the CMP instruction,
    the instruction sheet is saying that there is 16-bit data if s:w=1, what does that even mean????
    and then for example with the SUB instruction there is 16-bit data if s:w=01, what the hell? Is this a mistake?
 */

/* FIXME: The third instruction in the listing_0041 is not being decoded correctly.
    This might actually be related to the TODO added on top of the page.
    The first TODO might actually fix the immediate values talked about in this FIXME.

   The instruction is add si, 2
   We are decoding the register correctly, but the immediate is wrong.
   the immediate is currently being decoded as 50434 when it should be 2.

   The same is happening with the other immediate value moves.

   On top of this, there is something strange going on with the increments still even after the recent fix.
   Some operation are being skipped which means that we are incrementing too fast.

   Immediate to register increments are currently wrong.
*/
// W bit determines the size between 8 and 16-bits, the w bit is at different places depending on the instruction.
fn is_word_size(first_byte: u8, inst_type: InstructionType) -> bool {
    return if inst_type == InstructionType::ImmediateToRegisterMOV {
        first_byte & Masks::IMMEDIATE_TO_REG_MOV_W_BIT as u8 != 0
    } else {
        first_byte & Masks::W_BIT as u8 != 0
    }
}

fn get_register(get_reg: bool, inst: InstructionType, memory_mode: MemoryMode, first_byte: u8, second_byte: u8) -> &'static str {
    let rm_res = second_byte & Masks::RM_BITS as u8;
    let reg_res = second_byte & Masks::REG_BITS as u8;
    let is_word_size = is_word_size(first_byte, inst);

    if get_reg
        || inst == InstructionType::ImmediateToRegisterMOV
    {
            return match (reg_res, is_word_size) {
            (0b_00_000_000, true) => "ax",
            (0b_00_001_000, true) => "cx",
            (0b_00_010_000, true) => "dx",
            (0b_00_011_000, true) => "bx",
            (0b_00_100_000, true) => "sp",
            (0b_00_101_000, true) => "bp",
            (0b_00_110_000, true) => "si",
            (0b_00_111_000, true) => "di",
            //
            (0b_00_000_000, false) => "al",
            (0b_00_001_000, false) => "cl",
            (0b_00_010_000, false) => "dl",
            (0b_00_011_000, false) => "bl",
            (0b_00_100_000, false) => "ah",
            (0b_00_101_000, false) => "ch",
            (0b_00_110_000, false) => "dh",
            (0b_00_111_000, false) => "bh",
            _ => panic!("unknown register - get_register - get_reg branch\nreg was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", reg_res, first_byte, second_byte),
        };
    } else {
        if memory_mode == DirectMemoryOperation
            || inst == InstructionType::ImmediateToRegisterMemory
        {
            return match (rm_res, is_word_size) {
                (0b_00_000_000, true) => "ax",
                (0b_00_000_001, true) => "cx",
                (0b_00_000_010, true) => "dx",
                (0b_00_000_011, true) => "bx",
                (0b_00_000_100, true) => "sp",
                (0b_00_000_101, true) => "bp",
                (0b_00_000_110, true) => "si",
                (0b_00_000_111, true) => "di",
                //
                (0b_00_000_000, false) => "al",
                (0b_00_000_001, false) => "cl",
                (0b_00_000_010, false) => "dl",
                (0b_00_000_011, false) => "bl",
                (0b_00_000_100, false) => "ah",
                (0b_00_000_101, false) => "ch",
                (0b_00_000_110, false) => "dh",
                (0b_00_000_111, false) => "bh",
                _ => panic!("unknown register - get_register - Operation::REGISTER_MODE\nreg was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", reg_res, first_byte, second_byte),
            };
        } else if memory_mode == MemoryModeNoDisplacement {
            // 10/01/00
            return match rm_res {
                0b_00_000_000 => "bx + si",
                0b_00_000_001 => "bx + di",
                0b_00_000_010 => "bp + si",
                0b_00_000_011 => "bp + di",
                0b_00_000_100 => "si",
                0b_00_000_101 => "di",
                0b_00_000_110 => panic!(
                    "This: {:08b} should never be hit because it's handled by the direct memory operation.", rm_res),
                0b_00_000_111 => "bx",
                _ => panic!("unknown register - get_register - Operation::MEMORY_MODE_NONE\n R/M was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", rm_res, first_byte, second_byte),
            };
        } else if memory_mode == MemoryMode8Bit ||
           memory_mode == MemoryMode16Bit
        {
            return match rm_res {
                // NOTE:
                // When calling this function, we then check what the memory_mode was
                // to see what the displacement should be.
                // It will be either none, 8-bits or 16-bits depending on the result.
                // Here it will be either 8 or 16-bits.
                // the displacement is then added after the registers.

                // we get the register from the r/m field.
                0b_00_000_000 => "bx + si",
                0b_00_000_001 => "bx + di",
                0b_00_000_010 => "bp + si",
                0b_00_000_011 => "bp + di",
                0b_00_000_100 => "si",
                0b_00_000_101 => "di",
                0b_00_000_110 => "bp",
                0b_00_000_111 => "bx",
                _ => panic!(
                    "unknown register - get_register - R/M was: {:08b}, first_byte was: {:08b}, second_byte was: {:08b}", rm_res, first_byte, second_byte
                ),
            };
        } else if memory_mode == DirectMemoryOperation {
            // 00 + 110 RM
            "" // we return an empty string because MEMORY_MODE_DIRECT does not have a register, instead it's a direct 16-bit address that will be fetched later.
        } else {
            panic!("Unsupported operation - get_register {:?}, first_byte: {:8b}, second_byte: {:8b}", inst, first_byte, second_byte)
        }
    }
}

// sig stand for significant
fn combine_bytes(most_sig_byte: u8, least_sig_byte: u8) -> u16 {
    let combined_bytes: u16 = ((most_sig_byte as u16) << 8) | (least_sig_byte as u16);
    return combined_bytes;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();
    let op_codes = construct_opcodes();

    let mut i: usize = 0;
    let mut instruction_count: usize = 1;
    while i < binary_contents.len() {
        let first_byte = binary_contents[i];
        let second_byte = binary_contents[i + 1];


        let memory_mode = determine_memory_mode(second_byte);
        let instruction = determine_instruction(&op_codes, first_byte);
        let is_word_size = is_word_size(first_byte, instruction);

        let mut reg_or_immediate = String::new();
        let mut rm_or_immediate = String::new();

        // We are doing this if statement because in the case of an ImmediateToRegisterMemory (NON MOV one)
        // we actually do not have a REG register. the immediate value is always moved into the R/M register.
        if instruction == InstructionType::ImmediateToRegisterMemory {
            if is_word_size {
                // the fifth and sixth byte contain the immediate value because w is set to 1 (word size), we combine these two bytes and then cast it to a decimal.
                let fifth_byte = binary_contents[i + 4];
                let sixth_byte = binary_contents[i + 5];
                let combined = combine_bytes(fifth_byte, sixth_byte);
                reg_or_immediate = (combined as usize).to_string();
            } else {
                let fifth_byte = binary_contents[i + 4];
                reg_or_immediate = (fifth_byte as usize).to_string();
            }
        } else {
            // In this case its actually not an immediate, instead the string gets populated with the reg register.
            reg_or_immediate = get_register(true, instruction, memory_mode, first_byte, second_byte).parse().unwrap();
        }

        // This case is actually the complete opposite from the previous one.
        // The immediate to register MOV instruction actually does not have the R/M register
        // but has the REG register it used to move immediate values to.
        if instruction == InstructionType::ImmediateToRegisterMOV {
            // and the R/M Register actually is not used at all with the MOV immediate instruction.

            // With the immediate to register mov instruction, the immediate is stored in the second (and third byte if word sized).
            if is_word_size {
                let third_byte = binary_contents[i + 4];
                let combined = combine_bytes(third_byte, second_byte);
                reg_or_immediate = (combined as usize).to_string();
            } else {
                reg_or_immediate = (second_byte as usize).to_string();
            }
        } else {
            // In this case its actually not an immediate, instead the string gets populated with the R/M register.
            rm_or_immediate = get_register(false, instruction, memory_mode, first_byte, second_byte).parse().unwrap();
        }

        if instruction == InstructionType::ImmediateToRegisterMemory {
            if is_word_size {
                i += 6;
            } else {
                i += 5;
            }
            println!("Immediate value: {}, R/M Register: {}, operation: {:?}, memory_mode: {:?}, instruction: {}, first_byte: {:08b}, second_byte: {:08b}, index: {}", reg_or_immediate, rm_or_immediate, instruction, memory_mode, instruction_count, first_byte, second_byte,i);
        } else if instruction == InstructionType::ImmediateToRegisterMOV {
            if is_word_size {
                i += 3;
            } else {
                i += 2;
            }
            println!("Immediate value: {}, REG Register: {}, operation: {:?}, memory_mode: {:?}, instruction: {}, first_byte: {:08b}, second_byte: {:08b}, index: {}", rm_or_immediate, reg_or_immediate, instruction, memory_mode, instruction_count, first_byte, second_byte,i);
        } else if instruction == InstructionType::RegisterMemory{
            // TODO: how much do we increment in the case of direct memory address (16-bit)?
            if memory_mode == MemoryModeNoDisplacement {
                i += 2;
            } else if memory_mode == MemoryMode16Bit {
                i += 4;
            } else if memory_mode == MemoryMode8Bit {
                i += 3;
            } else if memory_mode == DirectMemoryOperation {
                i += 4;
            } else {
                panic!("Memory mode: {:?}, did not expect to get here.", memory_mode);
            }
            println!(
                "reg_register: {}, rm_register: {}, operation: {:?}, memory mode: {:?}, instruction: {}, first_byte: {:08b}, second_byte: {:08b}, index: {}",
                reg_or_immediate, rm_or_immediate, instruction, memory_mode, instruction_count, first_byte, second_byte, i
            );
        }
        else
        {
            panic!("Unknown instruction: {:?}, did not expect to get here.", instruction);
        }
        instruction_count += 1;
    }
}
