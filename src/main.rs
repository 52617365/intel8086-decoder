mod bits;
use bits::*;

use core::panic;
use std::{env, fs};
use crate::bits::InstructionType::{ImmediateToAccumulatorADD, ImmediateToAccumulatorCMP, ImmediateToRegisterMemory, ImmediateToRegisterMOV, ImmediateToAccumulatorSUB, RegisterMemory, JE_JUMP, JL_JUMP, JLE_JUMP, JB_JUMP, JBE_JUMP, JP_JUMP, JO_JUMP, JS_JUMP, JNE_JUMP, JNL_JUMP, LOOP, LOOPZ, JCXZ, LOOPNZ, JNS, JNO_JUMP, JNBE_JUMP, JNP_JUMP, JNB_JUMP, JNLE_JUMP};
use crate::bits::Masks::{D_BITS, IMMEDIATE_TO_REG_MOV_W_BIT};

use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement, RegisterMode};



// W bit determines the size between 8 and 16-bits, the w bit is at different places depending on the instruction.
// This function does not work with the immediate to registers because they use the s bit also, we have to take into consideration
// that bit separately.
fn is_word_size(first_byte: u8, inst_type: InstructionType) -> bool {
    return if inst_type == ImmediateToRegisterMOV {
        first_byte & IMMEDIATE_TO_REG_MOV_W_BIT as u8 != 0
    } else {
        first_byte & Masks::W_BIT as u8 != 0
    }
}

fn get_register(get_reg: bool, inst: InstructionType, memory_mode: MemoryModeEnum, first_byte: u8, second_byte: u8, is_word_size: bool) -> &'static str {
    let rm_res = second_byte & Masks::RM_BITS as u8;
    let reg_res = second_byte & Masks::REG_BITS as u8;

    if inst == ImmediateToAccumulatorSUB || inst == ImmediateToAccumulatorCMP || inst == ImmediateToAccumulatorADD {
        if is_word_size {
            return "ax"
        } else {
            return "al"
        }
    }
    if get_reg && inst != ImmediateToRegisterMOV
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
    }
    else if inst == ImmediateToRegisterMOV {
        let immediate_mov_reg_register = first_byte & IMMEDIATE_TO_MOV_REG_BITS as u8;
        return match (immediate_mov_reg_register, is_word_size) {
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
            _ => panic!("Did not expect us to get here. first_byte: {}, second_byte: {}, inst: {:?}", first_byte, second_byte, inst)
        }
    } else {
        if (memory_mode == DirectMemoryOperation || memory_mode == RegisterMode)
            || (inst == ImmediateToRegisterMemory && memory_mode != MemoryModeNoDisplacement && memory_mode != MemoryMode16Bit && memory_mode != MemoryMode8Bit)
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
            panic!("Unsupported operation - get_register - {:?}, first_byte: {:8b}, second_byte: {:8b}, memory_mode: {:?}", inst, first_byte, second_byte, memory_mode)
        }
    }
}

fn combine_bytes(high_byte: u8, low_byte: u8) -> u16 {
    ((high_byte as u16) << 8) | (low_byte as u16)
}

fn get_mnemonic(first_byte: u8, second_byte: u8, inst: InstructionType) -> &'static str {
    // We need this to determine the mnemonic for immediate to register moves.
    let reg_field = second_byte & Masks::REG_BITS as u8;

    if inst == JE_JUMP {
        return "je"
    }
    if inst == JL_JUMP {
        return "jl"
    }
    if inst == JLE_JUMP {
        return "jle"
    }
    if inst == JB_JUMP {
        return "jb"
    }
    if inst == JBE_JUMP {
        return "jbe"
    }
    if inst == JP_JUMP {
        return "jp"
    }
    if inst == JO_JUMP {
        return "jo"
    }
    if inst == JS_JUMP {
        return "js"
    }
    if inst == JNE_JUMP {
        return "jnz"
    }
    if inst == JNL_JUMP {
        return "jnl"
    }
    if inst == JNLE_JUMP {
        return "jg"
    }
    if inst == JNB_JUMP {
        return "jnb"
    }
    if inst == JNBE_JUMP {
        return "ja"
    }
    if inst == JNP_JUMP {
        return "jnp"
    }
    if inst == JNO_JUMP {
        return "jno"
    }
    if inst == JNS {
        return "jns"
    }
    if inst == LOOP {
        return "loop"
    }
    if inst == LOOPZ {
        return "loopz"
    }
    if inst == LOOPNZ {
        return "loopnz"
    }
    if inst == JCXZ {
        return "jcxz"
    }

    if inst == ImmediateToRegisterMOV {
        return "mov"
    }
    if inst == ImmediateToAccumulatorSUB {
        return "sub"
    }
    if inst == ImmediateToAccumulatorCMP {
        return "cmp"
    }
    if inst == ImmediateToAccumulatorADD {
        return "add"
    }

    if inst == RegisterMemory {
        return match first_byte {
            0b00000000 | 0b00000001 | 0b00000010 | 0b00000011 => "add",
            0b00101000 | 0b00101001 | 0b00101010 | 0b00101011 => "sub",
            0b00111000 | 0b00111001 | 0b00111010 | 0b00111011 => "cmp",
            0b10001000 | 0b10001001 | 0b10001010 | 0b10001011 => "mov",
            _ => panic!("unknown instruction: {:?}, first_byte: {:08b}", inst, first_byte)
        }
    } else if inst == ImmediateToRegisterMemory {
        return match (first_byte, reg_field) {
            (0b10000000, 0b00_101_000) | (0b10000001, 0b00_101_000) | (0b10000010, 0b00_101_000) | (0b10000011, 0b00_101_000) => "sub",
            (0b10000000, 0b00_111_000) | (0b10000001, 0b00_111_000) | (0b10000010, 0b00_111_000) | (0b10000011, 0b00_111_000) => "cmp",
            (0b10000000, 0b00_000_000) | (0b10000001, 0b00_000_000) | (0b10000010, 0b00_000_000) | (0b10000011, 0b00_000_000) => "add",
            (0b11000110, 0b00_000_000) | (0b11000111, 0b00_000_000) => "mov",
            _ => panic!("unknown instruction: {:?}, first_byte: {:08b}, reg_field: {:08b}", inst, first_byte, reg_field)
        }
    } else {
        todo!()
    }
}

// TODO: Next we need to emulate the instructions, this means that we need to keep state of the values passed into the registers/memory locations.
// it could possibly be wise to create a vector of structs that contain the register name, and the value associated with it.
// we can then do a linear loop through the vector to see if the register we care about matches and get the value out that way.

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

        let instruction = determine_instruction(&op_codes, first_byte);
        let mnemonic = get_mnemonic(first_byte, second_byte, instruction);
        let is_word_size = is_word_size(first_byte, instruction);
        let memory_mode = determine_memory_mode(second_byte);
        let is_s_bit_set = first_byte & S_BIT_M as u8 == 0b00000010;
        let instruction_size = determine_instruction_byte_size(instruction, is_word_size, memory_mode, mnemonic, is_s_bit_set);
        let reg_is_dest = first_byte & D_BITS as u8 != 0;

        let mut reg_or_immediate = String::new();
        let mut rm_or_immediate = String::new();

        // We are doing this if statement because in the case of an ImmediateToRegisterMemory (NON MOV one)
        // we actually do not have a REG register. the immediate value is always moved into the R/M register.

        if instruction == ImmediateToRegisterMemory {
            if !is_word_size {
                // TODO: Do we have to handle 8 and 16-bit memory modes here in its own branch?
                let third_byte = binary_contents[i + 2];
                reg_or_immediate = (third_byte as usize).to_string();
            } else { // is_word_size
                // MOV doesn't care about the s_bit. CMP, SUB, ADD do.
                // if w=1 and s=0 and mnemonic is sub/add/cmp, it's an 16-bit immediate.
                match (mnemonic, is_s_bit_set) {
                    ("mov", _) | ("cmp", false) | ("add", false) | ("sub", false) => {
                        if memory_mode == MemoryMode16Bit || memory_mode == MemoryMode8Bit || memory_mode == DirectMemoryOperation {
                            // the immediate is guaranteed to be 16-bit because the s bit is set to 0 in this branch.
                            let fifth_byte = binary_contents[i + 4];
                            let sixth_byte = binary_contents[i + 5];
                            let combined = combine_bytes(sixth_byte, fifth_byte);
                            reg_or_immediate = (combined as usize).to_string();
                        } else {
                            let third_byte = binary_contents[i + 2];
                            let fourth_byte = binary_contents[i + 3];
                            let combined = combine_bytes(fourth_byte, third_byte);
                            reg_or_immediate = (combined as usize).to_string();
                        }
                    },
                    ("cmp", true) | ("add", true) | ("sub", true) => {
                        if memory_mode == MemoryMode16Bit || memory_mode == MemoryMode8Bit || memory_mode == DirectMemoryOperation {
                            // In this branch we guarantee that the s bit is not set. Therefore the immediate can not be a 16-bit value.
                            // With 16-bit memory mode operations the immediate is in the fifth and sixth bytes depending on the size.
                            let fifth_byte = binary_contents[i + 4];
                            reg_or_immediate = (fifth_byte as usize).to_string();
                        }
                        else {
                            let third_byte = binary_contents[i + 2];
                            reg_or_immediate = (third_byte as usize).to_string();
                        }
                    }
                    _ => panic!("Unknown (mnemonic, s_bit_is_set): ({}, {})", mnemonic, is_s_bit_set)
                }
            }
        } else if instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP{
            if is_word_size {
                let third_byte = binary_contents[i + 2];
                let combined = combine_bytes(third_byte, second_byte);
                reg_or_immediate = (combined as usize).to_string();
            } else {
                reg_or_immediate = (second_byte as usize).to_string();
            }
        }
        else {
            // In this case its actually not an immediate, instead the string gets populated with the reg register.
            reg_or_immediate = get_register(true, instruction, memory_mode, first_byte, second_byte, is_word_size).parse().unwrap();
        }

        // This case is actually the complete opposite from the previous one.
        // The immediate to register MOV instruction actually does not have the R/M register
        // but has the REG register it used to move immediate values to.
        if instruction == ImmediateToRegisterMOV {
            // and the R/M Register actually is not used at all with the MOV immediate instruction.

            // With the immediate to register mov instruction, the immediate is stored in the second (and third byte if word sized).
            if is_word_size {
                let third_byte = binary_contents[i + 2];
                let combined = combine_bytes(third_byte, second_byte);
                rm_or_immediate = (combined as usize).to_string();
            } else {
                rm_or_immediate = (second_byte as usize).to_string();
            }
        } else {
            // In this case its actually not an immediate, instead the string gets populated with the R/M register.
            rm_or_immediate = get_register(false, instruction, memory_mode, first_byte, second_byte, is_word_size).parse().unwrap();
        }

        let formatted_instruction = format_instruction(&binary_contents, i, first_byte, second_byte, instruction, mnemonic, is_word_size, memory_mode, reg_is_dest, &mut reg_or_immediate, &mut rm_or_immediate);
        println!("{}", formatted_instruction);
        instruction_count += 1;
        i += instruction_size;
        // print!("size: {}, count: {} - ", instruction_size, instruction_count);
    }
}

fn format_instruction(binary_contents: &Vec<u8>, i: usize, first_byte: u8, second_byte: u8, instruction: InstructionType, mnemonic: &str, is_word_size: bool, memory_mode: MemoryModeEnum, reg_is_dest: bool, reg_or_immediate: &mut String, rm_or_immediate: &mut String) -> String {
    if instruction == ImmediateToRegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if is_word_size {
                return format!("{} word [{}], {}", mnemonic, rm_or_immediate, reg_or_immediate);
            } else {
                return format!("{} byte [{}], {}", mnemonic, rm_or_immediate, reg_or_immediate);
            }
        } else if memory_mode == MemoryMode8Bit {
            let displacement = get_8_bit_displacement(binary_contents, i);
            if is_word_size {
                return format!("{} word [{} + {}], {}", mnemonic, rm_or_immediate, displacement, reg_or_immediate);
            } else {
                return format!("{} byte [{} + {}], {}", mnemonic, rm_or_immediate, displacement, reg_or_immediate);
            }
        } else if memory_mode == MemoryMode16Bit {
            let displacement = get_16_bit_displacement(binary_contents, i);
            if is_word_size {
                return format!("{} word [{} + {}], {}", mnemonic, rm_or_immediate, displacement, reg_or_immediate);
            } else {
                return format!("{} byte [{} + {}], {}", mnemonic, rm_or_immediate, displacement, reg_or_immediate);
            }
        } else if memory_mode == DirectMemoryOperation {
            let displacement = get_16_bit_displacement(binary_contents, i);
            if is_word_size {
                // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                if reg_is_dest {
                    return format!("{} word [{}], {}", mnemonic, displacement, reg_or_immediate);
                } else {
                    return format!("{} word {}, [{}]", mnemonic, reg_or_immediate, displacement);
                }
            } else {
                // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                if reg_is_dest {
                    // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                    return format!("{} byte [{}], {}", mnemonic, reg_or_immediate, displacement);
                } else {
                    return format!("{} byte {}, [{}]", mnemonic, displacement, reg_or_immediate);
                }
            }
        } else if memory_mode == RegisterMode {
            if reg_is_dest {
                return format!("{} {}, {}", mnemonic, rm_or_immediate, reg_or_immediate);
            } else {
                return format!("{} {}, {}", mnemonic, reg_or_immediate, rm_or_immediate);
            }
        } else {
            panic!("Invalid memory mode {:?}.", memory_mode);
        }
    } else if instruction == ImmediateToRegisterMOV {
        return format!("{} {}, {}", mnemonic, reg_or_immediate, rm_or_immediate);
    } else if instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP {

        // NOTE!!!!: with the ImmediateToAccumulator operations, the registers are not specified in the bits,
        // instead, they are hard coded. if W = 1 then the register an immediate is getting moved to is ax, else al.
        // the reason why we are printing the reg_or_immediate variable is because we store the immediate value in there.
        // this is because we don't want to make a new variable for just one operation. The name is misleading but live with it.

        let ax_or_al = get_register(true, instruction, memory_mode, first_byte, second_byte, is_word_size);
        return format!("{} {}, {}", mnemonic, ax_or_al, reg_or_immediate);
    } else if instruction == RegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if reg_is_dest {
                return format!("{} {}, [{}]", mnemonic, reg_or_immediate, rm_or_immediate)
            } else {
                return format!("{} [{}], {}", mnemonic, rm_or_immediate, reg_or_immediate)
            }
        } else if memory_mode == MemoryMode8Bit {
            let disp = get_8_bit_displacement(binary_contents, i);
            if reg_is_dest {
                return format!("{} {}, [{} + {}]", mnemonic, reg_or_immediate, rm_or_immediate, disp)
            } else {
                return format!("{} [{} + {}], {}", mnemonic, rm_or_immediate, disp, reg_or_immediate)
            }
        } else if memory_mode == MemoryMode16Bit {
            let displacement = get_16_bit_displacement(binary_contents, i);
            if reg_is_dest {
                return format!("{} {}, [{} + {}]", mnemonic, reg_or_immediate, rm_or_immediate, displacement)
            } else {
                return format!("{} [{} + {}], {}", mnemonic, rm_or_immediate, displacement, reg_or_immediate)
            }
        } else if memory_mode == RegisterMode {
            if reg_is_dest {
                return format!("{} {}, {}", mnemonic, reg_or_immediate, rm_or_immediate)
            } else {
                return format!("{} {}, {}", mnemonic, rm_or_immediate, reg_or_immediate)
            }
        } else if memory_mode == DirectMemoryOperation {
            let displacement = get_16_bit_displacement(binary_contents, i);
            if reg_is_dest {
                return format!("{} {}, [{}]", mnemonic, displacement, rm_or_immediate)
            } else {
                return format!("{} {}, [{}]", mnemonic, rm_or_immediate, displacement)
            }
        } else {
            panic!("Unknown memory mode: {:?}, did not expect to get here.", memory_mode);
        }
    } else if instruction == JE_JUMP
        || instruction == JL_JUMP
        || instruction == JLE_JUMP
        || instruction == JB_JUMP
        || instruction == JBE_JUMP
        || instruction == JP_JUMP
        || instruction == JO_JUMP
        || instruction == JS_JUMP
        || instruction == JNE_JUMP
        || instruction == JNL_JUMP
        || instruction == JNLE_JUMP
        || instruction == JNB_JUMP
        || instruction == JNBE_JUMP
        || instruction == JNP_JUMP
        || instruction == JNO_JUMP
        || instruction == JNS
        || instruction == LOOP
        || instruction == LOOPZ
        || instruction == LOOPNZ
        || instruction == JCXZ
    {
        return format!("{} {}", mnemonic, second_byte as usize);
    } else {
        panic!("Unknown instruction: {:?}, did not expect to get here.", instruction);
    }
}

fn get_16_bit_displacement(binary_contents: &Vec<u8>, i: usize) -> usize {
    let first_disp = binary_contents[i + 2];
    let second_disp = binary_contents[i + 3];
    let displacement = combine_bytes(second_disp, first_disp);
    displacement as usize
}

fn get_8_bit_displacement(binary_contents: &Vec<u8>, i: usize) -> usize {
    let first_disp = binary_contents[i + 2];
    return first_disp as usize
}
