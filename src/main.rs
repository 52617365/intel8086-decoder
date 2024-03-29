mod bits;
mod registers;
mod flag_registers;
mod memory;

/*
TODO: On top of the testing we want to do, we also need to support the old homework because during the newer homework, the old ones broke.
*/


use bits::*;

use crate::memory::{bits_struct, get_displacement, load_memory_contents_as_decimal_and_optionally_update_original_value, memory_contents, memory_struct, store_memory_value};
use crate::bits::combine_bytes;
use core::panic;
use std::{env, fs};
use crate::bits::InstructionType::{ImmediateToAccumulatorADD, ImmediateToAccumulatorCMP, ImmediateToRegisterMemory, ImmediateToRegisterMOV, ImmediateToAccumulatorSUB, RegisterMemory, JE_JUMP, JL_JUMP, JLE_JUMP, JB_JUMP, JBE_JUMP, JP_JUMP, JO_JUMP, JS_JUMP, JNE_JUMP, JNL_JUMP, LOOP, LOOPZ, JCXZ, LOOPNZ, JNS, JNO_JUMP, JNBE_JUMP, JNP_JUMP, JNB_JUMP, JNLE_JUMP};
use crate::bits::Masks::{D_BITS, IMMEDIATE_TO_REG_MOV_W_BIT};

use crate::flag_registers::{number_is_signed, twos_complement};
use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement, RegisterMode};
use crate::registers::{Value, ValueEnum, construct_registers, get_register_state, Register, register_contains_multiple_registers, update_original_register_value, update_register_value, print_out_state_of_all_registers};
use crate::flag_registers::{construct_flag_registers, set_flags, get_all_currently_set_flags, clear_flags_registers, flag_register_is_set, FlagRegister};

// W bit determines the size between 8 and 16-bits, the w bit is at different places depending on the instruction.
// This function does not work with the immediate to registers because they use the s bit also, we have to take into consideration
// that bit separatelyemoveOccurrence
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
                // We add the displacement after the registers in the end.
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
            "" // we return an empty string because DirectMemoryOperation does not have a register, instead it's a direct 16-bit address that will be fetched later.
        } else {
            panic!("Unsupported operation - get_register - {:?}, first_byte: {:8b}, second_byte: {:8b}, memory_mode: {:?}", inst, first_byte, second_byte, memory_mode)
        }
    }
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
        panic!("this is not supported, why did we get here?");
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();

    let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct{bits: 0, initialized: false}, original_bits: bits_struct{bits: 0, initialized: false}} }; 100000];

    let mut registers = construct_registers();
    let op_codes = construct_opcodes();
    let mut flag_registers = construct_flag_registers();

    let mut old_instruction_pointer: usize = 0;
    let mut instruction_pointer: usize = 0;
    let simulate_code = true;
    let mut instruction_count = 0;
    while instruction_pointer < binary_contents.len() {
        instruction_count += 1;
        old_instruction_pointer = instruction_pointer;
        let first_byte = binary_contents[instruction_pointer];
        let instruction = determine_instruction(&op_codes, first_byte);
        let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, simulate_code);

        if simulate_code {
            println!("{} | {} -> {} | flags: {:?}, IP: {} -> {}", decoded_instruction.formatted_instruction, decoded_instruction.original_value.get_string_number_from_bits(), decoded_instruction.updated_value.get_string_number_from_bits(), decoded_instruction.flags, old_instruction_pointer, instruction_pointer);
        } else {
            println!("{}", decoded_instruction.formatted_instruction);
        }
    }
    println!("\nFinal registers:");
    print_out_state_of_all_registers(registers);
    print!("\tip: {}", instruction_pointer);
    get_all_currently_set_flags(&mut flag_registers);

    println!("\nInstruction count: {}", instruction_count);
}

#[derive(Clone, Debug)]
struct instruction_data {
   formatted_instruction: String,
   original_value: Value,
   updated_value: Value,
   flags: Vec<&'static str>,
}

impl PartialEq for instruction_data {
    fn eq(&self, other: &Self) -> bool {
        self.formatted_instruction == other.formatted_instruction &&
            self.original_value == other.original_value &&
            self.updated_value == other.updated_value &&
            self.flags == other.flags
    }
}
fn instruction_has_immediate_value_in_rm_register(instruction: InstructionType) -> bool {
    return instruction == ImmediateToRegisterMOV;
}

fn instruction_has_immediate_value_in_reg_register(instruction: InstructionType) -> bool {
    return instruction == ImmediateToRegisterMemory || instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP
}


fn get_immediate_from_rm_register(instruction: InstructionType, is_word_size: bool, instruction_pointer: usize, binary_contents: &Vec<u8>) -> Value {
            // This case is actually the complete opposite from the previous one.
            // The immediate to register MOV instruction actually does not have the R/M register
            // but has the REG register it used to move immediate values to.
            if instruction == ImmediateToRegisterMOV {
                // and the R/M Register actually is not used at all with the MOV immediate instruction.

                // With the immediate to register mov instruction, the immediate is stored in the second (and third byte if word sized).
                let second_byte = binary_contents[instruction_pointer + 1];
                if is_word_size {
                    let third_byte = binary_contents[instruction_pointer + 2];
                    let combined = combine_bytes(third_byte, second_byte);
                    let value = ValueEnum::WordSize(combined);
                    return Value {
                        value,
                        is_signed: number_is_signed(value),
                    };
                } else {
                    let value = ValueEnum::ByteSize(second_byte);
                    return Value {
                        value,
                        is_signed: number_is_signed(value),
                    };
            }
    }
    panic!("we thought rm register contained an immediate when it did not.")
}

fn get_immediate_from_reg_register(mnemonic: &str, instruction: InstructionType, is_s_bit_set: bool, is_word_size: bool, memory_mode: MemoryModeEnum, instruction_pointer: usize, binary_contents: &Vec<u8>) -> Value {
    if instruction == ImmediateToRegisterMemory {
        if !is_word_size {
            let third_byte = binary_contents[instruction_pointer + 2];
            let value = ValueEnum::ByteSize(third_byte);
            return Value{
                value,
                is_signed: number_is_signed(value),
            };
        } else { // is_word_size
            // MOV doesn't care about the s_bit. CMP, SUB, ADD do.
            // if w=1 and s=0 and mnemonic is sub/add/cmp, it's an 16-bit immediate.
            match (mnemonic, is_s_bit_set) {
                ("mov", _) | ("cmp", false) | ("add", false) | ("sub", false) => {
                    if memory_mode == MemoryMode8Bit {
                        let fourth_byte = binary_contents[instruction_pointer + 3];
                        let fifth_byte = binary_contents[instruction_pointer + 4];
                        let combined = combine_bytes(fifth_byte, fourth_byte);
                        let value = ValueEnum::WordSize(combined);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };
                    } else if memory_mode == MemoryMode16Bit || memory_mode == DirectMemoryOperation {
                        // the immediate is guaranteed to be 16-bit because the s bit is set to 0 in this branch.
                        let fifth_byte = binary_contents[instruction_pointer + 4];
                        let sixth_byte = binary_contents[instruction_pointer + 5];
                        let combined = combine_bytes(sixth_byte, fifth_byte);
                        let value = ValueEnum::WordSize(combined);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };
                    } else {
                        let third_byte = binary_contents[instruction_pointer + 2];
                        let fourth_byte = binary_contents[instruction_pointer + 3];
                        let combined = combine_bytes(fourth_byte, third_byte);

                        let value = ValueEnum::WordSize(combined);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };
                    }
                },
                ("cmp", true) | ("add", true) | ("sub", true) => {
                    if memory_mode == MemoryMode16Bit || memory_mode == MemoryMode8Bit || memory_mode == DirectMemoryOperation {
                        // In this branch we guarantee that the s bit is not set. Therefore the immediate can not be a 16-bit value.
                        // With 16-bit memory mode operations the immediate is in the fifth and sixth bytes depending on the size.
                        let fifth_byte = binary_contents[instruction_pointer + 4];
                        let value = ValueEnum::ByteSize(fifth_byte);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };

                    } else {
                        let third_byte = binary_contents[instruction_pointer + 2];

                        let value = ValueEnum::ByteSize(third_byte);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };
                    }
                }
                _ => panic!("Unknown (mnemonic, s_bit_is_set): ({})", mnemonic)
            }
        }
    } else if instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP {
        let second_byte = binary_contents[instruction_pointer + 1];
        if is_word_size {
            let third_byte = binary_contents[instruction_pointer + 2];
            let combined = combine_bytes(third_byte, second_byte);
            let value = ValueEnum::WordSize(combined);
            return Value{
                value,
                is_signed: number_is_signed(value),
            };

        } else {
            let value = ValueEnum::ByteSize(second_byte);
            return Value{
                value,
                is_signed: number_is_signed(value),
            };
        }
    }
    panic!("We thought that the reg register contained an immediate when it did not.")
}

fn decode_instruction(binary_contents: &Vec<u8>, instruction: InstructionType, registers: &mut Vec<Register>, flag_registers: &mut [FlagRegister; 2], memory: &mut [memory_struct; 100000], instruction_pointer: &mut usize, simulate: bool) -> instruction_data {
    let first_byte = binary_contents[*instruction_pointer];
    let second_byte = binary_contents[*instruction_pointer + 1];

    let mnemonic = get_mnemonic(first_byte, second_byte, instruction);
    let is_word_size = is_word_size(first_byte, instruction);
    let memory_mode = determine_memory_mode(second_byte);
    let is_s_bit_set = first_byte & S_BIT_M as u8 == 0b00000010;
    let instruction_size = determine_instruction_byte_size(instruction, is_word_size, memory_mode, mnemonic, is_s_bit_set);
    let reg_is_dest = first_byte & D_BITS as u8 != 0;

    let mut reg_register = String::new();
    let mut reg_immediate: Value = Value{value: ValueEnum::Uninitialized, is_signed: false};
    let mut rm_register = String::new();
    let mut rm_immediate: Value = Value{value: ValueEnum::Uninitialized, is_signed: false};

    if instruction_has_immediate_value_in_reg_register(instruction) {
        reg_immediate = get_immediate_from_reg_register(mnemonic, instruction, is_s_bit_set, is_word_size, memory_mode, *instruction_pointer, &binary_contents)
    } else {
        reg_register = get_register(true, instruction, memory_mode, first_byte, second_byte, is_word_size).parse().unwrap();
    }

    if instruction_has_immediate_value_in_rm_register(instruction) {
        rm_immediate = get_immediate_from_rm_register(instruction, is_word_size, *instruction_pointer, &binary_contents)
    } else {
        rm_register = get_register(false, instruction, memory_mode, first_byte, second_byte, is_word_size).parse().unwrap();
    }


    if simulate {
        if instruction == ImmediateToRegisterMOV {
            // With the ImmediateToRegisterMOV instruction, get_reg does not matter at all.
            let reg = get_register_state(&reg_register, &registers);
            update_register_value(reg.register, rm_immediate.value, registers, instruction, memory_mode, mnemonic, is_word_size);
        } else if instruction_is_immediate_to_register(instruction) && instruction != ImmediateToRegisterMOV {
            if instruction_uses_memory(memory_mode) {
                let address_from_disp = get_displacement(&binary_contents, *instruction_pointer, memory_mode);
                if memory_mode == DirectMemoryOperation {
                    store_memory_value(memory, address_from_disp, 0, reg_immediate, mnemonic, is_word_size);
                } else {
                    if register_contains_multiple_registers(&rm_register) {
                        let (first_register, second_register) = get_registers_from_multiple_register(registers, &rm_register);
                        // TODO: fill this.
                    } else {
                        let rm = get_register_state(&rm_register, registers).updated_value;
                        if let ValueEnum::Uninitialized = rm.value {}
                        else {
                            if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit {
                                let rm_value_casted = rm.value.get_usize();
                                let memory_address_displacement = get_displacement(&binary_contents, *instruction_pointer, memory_mode);

                                store_memory_value(memory, rm_value_casted, memory_address_displacement, reg_immediate, mnemonic, is_word_size);
                            } else {
                                let rm_value_casted = rm.value.get_usize();
                                store_memory_value(memory, rm_value_casted, 0, reg_immediate, mnemonic, is_word_size);
                            }
                        }
                    }
                }
            } else if reg_is_dest && instruction != ImmediateToRegisterMemory || instruction == ImmediateToRegisterMOV {
                let reg = get_register_state(&reg_register, registers);
                // in this branch we can just update the value with the immediate.
                update_register_value(reg.register, rm_immediate.value, registers, instruction, memory_mode, mnemonic, is_word_size);
            } else {
                let rm = get_register_state(&rm_register, registers);
                // in this branch we can just update the value with the immediate.
                update_register_value(rm.register, reg_immediate.value, registers, instruction, memory_mode, mnemonic, is_word_size);
            }
        } else if instruction == RegisterMemory && instruction_uses_memory(memory_mode) {
            let memory_address_displacement = get_displacement(&binary_contents, *instruction_pointer, memory_mode);
            let reg = get_register_state(&reg_register, registers);
            if reg_is_dest {
                if register_contains_multiple_registers(&rm_register) {
                    let combined_registers_from_rm = combine_register_containing_multiple_registers(registers, &rm_register);
                    let combined_registers_to_usize = combined_registers_from_rm.value.get_usize();
                    let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, combined_registers_to_usize, memory_address_displacement, is_word_size, false);
                    update_register_value(reg.register, memory_contents.original_value.value, registers, instruction, memory_mode, mnemonic, is_word_size)
                } else {
                    let rm = get_register_state(&rm_register, registers);
                    let rm_value_to_usize = rm.updated_value.value.get_usize();
                    let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, rm_value_to_usize, memory_address_displacement, is_word_size, false);
                    update_register_value(reg.register, memory_contents.original_value.value, registers, instruction, memory_mode, mnemonic, is_word_size);
                }
            } else {
                if register_contains_multiple_registers(&rm_register) {
                    let combined_registers_from_rm = combine_register_containing_multiple_registers(registers, &rm_register);
                    let combined_registers_to_usize = combined_registers_from_rm.value.get_usize();
                    store_memory_value(memory, combined_registers_to_usize, memory_address_displacement, reg.original_value, mnemonic, is_word_size);
                } else {
                    let rm = get_register_state(&rm_register, &registers);
                    let rm_value_to_usize = rm.updated_value.value.get_usize();
                    store_memory_value(memory, rm_value_to_usize, memory_address_displacement, reg.original_value, mnemonic, is_word_size);
                }
            }
        }
        else if instruction == RegisterMemory && memory_mode == RegisterMode {
            let reg = get_register_state(&reg_register, &registers);
            let rm = get_register_state(&rm_register, &registers);
            if reg_is_dest {
                update_register_value(reg.register, rm.original_value.value, registers, instruction, memory_mode, mnemonic, is_word_size);
            } else {
                update_register_value(rm.register, reg.original_value.value, registers, instruction, memory_mode, mnemonic, is_word_size)
            }
        }


        if !instruction_is_conditional_jump(instruction) {
            if mnemonic != "mov" {
                let mut value: ValueEnum = ValueEnum::Uninitialized;

                if instruction_is_immediate_to_register(instruction) && instruction_uses_memory(memory_mode) {
                    // rm register is always dest.
                    if register_contains_multiple_registers(&rm_register) {
                        if mnemonic == "cmp" {
                            let combined_registers_from_rm = combine_register_containing_multiple_registers(registers, &rm_register);
                            let combined_registers_value = combined_registers_from_rm.value;
                            let result = combined_registers_value.wrap_sub(reg_immediate.value);
                            value = result;
                        } else {
                            value = combine_register_containing_multiple_registers(registers, &rm_register).value;
                        }
                    } else {
                        if mnemonic == "cmp" {
                            let rm = get_register_state(&rm_register, registers);
                            let result = rm.updated_value.value.wrap_sub(reg_immediate.value);
                            value = result;
                        } else {
                            let rm = get_register_state(&rm_register, registers);
                            value = rm.updated_value.value;
                        }
                    }
                } else if instruction == RegisterMemory && instruction_uses_memory(memory_mode) {
                    if memory_mode == DirectMemoryOperation {
                        if reg_is_dest {
                            if mnemonic == "cmp" {
                                let reg = get_register_state(&reg_register, registers);
                                let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, rm_immediate.value.get_usize(), 0, is_word_size, false);
                                value = reg.updated_value.value.wrap_sub(memory_contents.original_value.value);
                            } else {
                                let reg = get_register_state(&reg_register, registers);
                                value = reg.updated_value.value;
                            }
                        } else {
                            let rm = get_register_state(&rm_register, registers);
                            let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, reg_immediate.value.get_usize(), 0, is_word_size, false);
                            value = rm.updated_value.value.wrap_sub(memory_contents.modified_value.value);
                        }
                    } else {
                       panic!("Not implemented");
                    }
                }
                if reg_is_dest && instruction != ImmediateToRegisterMemory { // FIXME: This is a weird ass branch lol, smelly code.
                    if mnemonic == "cmp" {
                        let reg = get_register_state(&reg_register, registers);
                        let rm = get_register_state(&rm_register, registers);
                        value = reg.updated_value.value.wrap_sub(rm.updated_value.value);
                    } else {
                        let reg = get_register_state(&reg_register, registers);
                        value = reg.updated_value.value;
                    }
                } else {
                    if mnemonic == "cmp" {
                        let rm = get_register_state(&rm_register, registers);
                        let reg = get_register_state(&reg_register, registers);
                        value = rm.updated_value.value.wrap_sub(reg.updated_value.value);
                    } else {
                        let rm = get_register_state(&rm_register, registers);
                        value = rm.updated_value.value;
                    }
                }
                set_flags(value, flag_registers);
            } else {
                // We don't clear if it's a conditional jump because the jnz conditional jump for example relies on the flags to know when to stop jumping.
                clear_flags_registers(flag_registers);
            }
        }
    }

    let formatted_instruction = format_instruction(&binary_contents, *instruction_pointer, first_byte, second_byte, instruction, mnemonic, is_word_size, memory_mode, reg_is_dest, &reg_register, &rm_register, reg_immediate, rm_immediate, instruction_size);


    let instruction_details: instruction_data;

    if instruction == ImmediateToRegisterMemory && instruction_uses_memory(memory_mode) {
        let disp = get_displacement(&binary_contents, *instruction_pointer, memory_mode);

        if memory_mode == DirectMemoryOperation {
            let decimal_memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, 0, disp, is_word_size, true);
            instruction_details = instruction_data{
                formatted_instruction,
                original_value: decimal_memory_contents.original_value,
                updated_value: decimal_memory_contents.modified_value,
                flags: get_all_currently_set_flags(flag_registers),
            }
        } else {
            // rm register contains the dest.
            if register_contains_multiple_registers(&rm_register) {
                let combined_rm_registers = combine_register_containing_multiple_registers(registers, &rm_register);
                let decimal_memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, combined_rm_registers.value.get_usize(), disp, is_word_size, true);
                instruction_details = instruction_data{
                    formatted_instruction,
                    original_value: decimal_memory_contents.original_value,
                    updated_value: decimal_memory_contents.modified_value,
                    flags: get_all_currently_set_flags(flag_registers),
                };
            } else {
                let rm = get_register_state(&rm_register, &registers);

                let decimal_memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, rm.updated_value.value.get_usize(), disp, is_word_size, true);
                instruction_details = instruction_data{
                    formatted_instruction,
                    original_value: decimal_memory_contents.original_value,
                    updated_value: decimal_memory_contents.modified_value,
                    flags: get_all_currently_set_flags(flag_registers),
                };
            }
        }
    } else if reg_is_dest && instruction != ImmediateToRegisterMemory || instruction == ImmediateToRegisterMOV {
        let reg = get_register_state(&reg_register, &registers);
        instruction_details = instruction_data{
            formatted_instruction,
            original_value: reg.original_value,
            updated_value: reg.updated_value,
            flags: get_all_currently_set_flags(flag_registers),
        };
    } else if instruction_is_conditional_jump(instruction) {
        // we print this out separately because does not modify flags but it relies on them to know when to stop a loop for example so
        // we don't want to clear it but we still want to signal in the print that it does not modify flags.

        // let rm = get_register_state(&rm_register, &registers);
        // We initialize original_value and updated_value with uninitalized because the conditional jump does not modify the registers (only the IP register but we are handling that here with the instruction_pointer variable getting incremented..)
        instruction_details = instruction_data{
            formatted_instruction,
            original_value: Value{value:ValueEnum::Uninitialized, is_signed: false},
            updated_value: Value{value:ValueEnum::Uninitialized, is_signed: false},
            flags: get_all_currently_set_flags(flag_registers),
        };
    }
    else if !instruction_is_immediate_to_register(instruction) && instruction_uses_memory(memory_mode) {
        let reg = get_register_state(&reg_register, registers);
        if reg_is_dest {
            // In this branch the value is in the register.
            instruction_details = instruction_data{
                formatted_instruction,
                original_value: reg.original_value,
                updated_value: reg.updated_value,
                flags: get_all_currently_set_flags(flag_registers),
            }
        } else {
            if register_contains_multiple_registers(&rm_register) {
                let combined_registers_from_rm  = combine_register_containing_multiple_registers(registers, &rm_register);
                let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, combined_registers_from_rm.value.get_usize(), 0, is_word_size, true);
                instruction_details = instruction_data{
                    formatted_instruction,
                    original_value: memory_contents.original_value,
                    updated_value: memory_contents.modified_value,
                    flags: get_all_currently_set_flags(flag_registers),
                }
            } else {
                let rm = get_register_state(&rm_register, registers);
                let memory_contents = load_memory_contents_as_decimal_and_optionally_update_original_value(memory, memory_mode, rm.original_value.value.get_usize(), 0, is_word_size, true);
                instruction_details = instruction_data{
                    formatted_instruction,
                    original_value: memory_contents.original_value,
                    updated_value: memory_contents.modified_value,
                    flags: get_all_currently_set_flags(flag_registers),
                }
            }
        }
    }
    else {
        let rm = get_register_state(&rm_register, &registers);
        instruction_details = instruction_data{
            formatted_instruction,
            original_value: rm.original_value,
            updated_value: rm.updated_value,
            flags: get_all_currently_set_flags(flag_registers),
        }
    }


    if !instruction_is_conditional_jump(instruction) {
        if reg_is_dest && instruction != ImmediateToRegisterMemory || instruction == ImmediateToRegisterMOV {
            let reg = get_register_state(&reg_register, &registers);
            update_original_register_value(reg.register, reg.updated_value.value, registers);
        } else if instruction_uses_memory(memory_mode) {
            // TODO: fill
        } else {
            let rm = get_register_state(&rm_register, &registers);
            update_original_register_value(rm.register, rm.updated_value.value, registers);
        }
    }

    assert_ne!(instruction_details.formatted_instruction, "", "instruction_details struct is not initialized, this should never happen.");

    if instruction_is_conditional_jump(instruction) && simulate {
        perform_conditional_jump(flag_registers, instruction_size, instruction_pointer, second_byte, instruction);
    } else {
        *instruction_pointer += instruction_size;
    }


    return instruction_details;
}

// This function extracts the registers from for example a bx + si instruction into bx, si and returns it.
fn get_registers_from_multiple_register(registers: &Vec<Register>, rm_register: &String) -> (Register, Register) {
    let split_register: Vec<&str> = rm_register.split(|c| c == '+' || c == '-').collect();
    let first_register = get_register_state(split_register[0].trim(), registers);
    let second_register = get_register_state(split_register[1].trim(), registers);
    (first_register, second_register)
}

fn perform_conditional_jump(flag_registers: &mut [FlagRegister; 2], instruction_size: usize, instruction_pointer: &mut usize, second_byte: u8, instruction: InstructionType) {
    let mut jump_happens = false;

    match instruction {
        JE_JUMP | JLE_JUMP | JBE_JUMP => {
            // JLE also has SF<>OF as a condition but we don't handle OF currently.
            // JBE also has CF=1 as a condition but we don't handle CF currently.
            if flag_register_is_set("ZF", flag_registers) {
                jump_happens = true;
            }
        },
        JS_JUMP => {
            if flag_register_is_set("SF", flag_registers) {
                jump_happens = true;
            }
        },
        JNE_JUMP => {
            if !flag_register_is_set("ZF", flag_registers) {
                jump_happens = true;
            }
        },
        JNS => {
            if !flag_register_is_set("SF", flag_registers) {
                jump_happens = true;
            }
        },
        _ => (),
    }
    if jump_happens {
        // let offset = twos_complement(second_byte) as usize;
        // We might need to add logic in case the jump is forwards but
        // that was not in the assignment so I'm not going to worry about that yet.
        // let offset = twos_complement(second_byte);
        let jump = second_byte.wrapping_add(instruction_size as u8);
        let offset = twos_complement(jump) as usize;
        *instruction_pointer -= offset;
    } else {
        *instruction_pointer += instruction_size;
    }
}

fn format_instruction(binary_contents: &Vec<u8>, ip: usize, first_byte: u8, second_byte: u8, instruction: InstructionType, mnemonic: &str, is_word_size: bool, memory_mode: MemoryModeEnum, reg_is_dest: bool, reg_register: &String, rm_register: &String, reg_immediate: Value, rm_immediate: Value, instruction_size: usize) -> String {
    if instruction == ImmediateToRegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if is_word_size {
                return format!("{} word [{}], {}", mnemonic, rm_register, reg_immediate.get_string_number_from_bits());
            } else {
                return format!("{} byte [{}], {}", mnemonic, rm_register, reg_immediate.get_string_number_from_bits());
            }
        } else if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit {
            let displacement = get_displacement(binary_contents, ip, memory_mode);
            if is_word_size {
                return format!("{} word [{} + {}], {}", mnemonic, rm_register, displacement, reg_immediate.get_string_number_from_bits());
            } else {
                return format!("{} byte [{} + {}], {}", mnemonic, rm_register, displacement, reg_immediate.get_string_number_from_bits());
            }
        } else if memory_mode == DirectMemoryOperation {
            let displacement = get_displacement(binary_contents, ip, memory_mode);
            if is_word_size {
                // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                if reg_is_dest {
                    return format!("{} word [{}], {}", mnemonic, displacement, reg_immediate.get_string_number_from_bits());
                } else {
                    return format!("{} word {}, [{}]", mnemonic, reg_immediate.get_string_number_from_bits(), displacement);
                }
            } else {
                // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                if reg_is_dest {
                    // NOTE: in this branch the reg_or_immediate and reg_is_dest have no connection to each other. This is an exception with the direct memory mode address.
                    return format!("{} byte [{}], {}", mnemonic, reg_immediate.get_string_number_from_bits(), displacement);
                } else {
                    return format!("{} byte {}, [{}]", mnemonic, displacement, reg_immediate.get_string_number_from_bits());
                }
            }
        } else if memory_mode == RegisterMode {
            // NOTE: The reason why the destination is always rm_register is because with the
            // ImmediateToRegisterMemory instruction, the destination is always the rm_register.
            return format!("{} {}, {}", mnemonic, rm_register, reg_immediate.get_string_number_from_bits());
        } else {
            panic!("Invalid memory mode {:?}.", memory_mode);
        }
    } else if instruction == ImmediateToRegisterMOV {
        return format!("{} {}, {}", mnemonic, reg_register, rm_immediate.get_string_number_from_bits());
    } else if instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP {

        // NOTE!!!!: with the ImmediateToAccumulator operations, the registers are not specified in the bits,
        // instead, they are hard coded. if W = 1 then the register an immediate is getting moved to is ax, else al.
        // the reason why we are printing the reg_or_immediate variable is because we store the immediate value in there.
        // this is because we don't want to make a new variable for just one operation. The name is misleading but live with it.

        let ax_or_al = get_register(true, instruction, memory_mode, first_byte, second_byte, is_word_size);
        return format!("{} {}, {}", mnemonic, ax_or_al, reg_immediate.get_string_number_from_bits());
    } else if instruction == RegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if reg_is_dest{
                return format!("{} {}, [{}]", mnemonic, reg_register, rm_register)
            } else {
                if mnemonic == "mov" && is_word_size {
                    return format!("{} word [{}], {}", mnemonic, rm_register, reg_register)
                } else {
                    return format!("{} [{}], {}", mnemonic, rm_register, reg_register)
                }
            }
        } else if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit {
            let displacement = get_displacement(binary_contents, ip, memory_mode);
            if reg_is_dest {
                return format!("{} {}, [{} + {}]", mnemonic, reg_register, rm_register, displacement)
            } else {
                return format!("{} [{} + {}], {}", mnemonic, rm_register, displacement, reg_register)
            }
        } else if memory_mode == RegisterMode {
            if reg_is_dest {
                return format!("{} {}, {}", mnemonic, reg_register, rm_register)
            } else {
                return format!("{} {}, {}", mnemonic, rm_register, reg_register)
            }
        } else if memory_mode == DirectMemoryOperation {
            let displacement = get_displacement(binary_contents, ip, memory_mode);
            if is_word_size {
                return format!("{} {}, word [{}]", mnemonic, reg_register, displacement);
            } else {
                return format!("{} {}, byte [{}]", mnemonic, reg_register, displacement);
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
        if number_is_signed(ValueEnum::ByteSize(second_byte)) {
            let instruction_size_cast = i8::try_from(instruction_size).unwrap();
            assert!(instruction_size_cast > 0, "This should never be negative, we're just doing this to fight rust rules.");
            let offset = twos_complement(second_byte).wrapping_sub(instruction_size_cast);
            if offset == 0 {
                return format!("{} {}", mnemonic, offset);
            } else {
                return format!("{} -{}", mnemonic, offset);
            }
        } else {
            return format!("{} {}", mnemonic, second_byte.wrapping_sub(u8::try_from(instruction_size).unwrap()) as usize);
        }
    } else {
        panic!("Unknown instruction: {:?}, did not expect to get here.", instruction);
    }
}


fn combine_register_containing_multiple_registers(registers: &Vec<Register>, rm_register_with_multiple_registers: &String) -> Value {
    let (first_register, second_register) = get_registers_from_multiple_register(registers, rm_register_with_multiple_registers);
    if rm_register_with_multiple_registers.contains("+") {
        // We can turn this into an usize here because it won't change the underlying value. Only
        // the destination type matters.
        let combined_value = first_register.updated_value.wrap_add_and_return_result(second_register.updated_value.value);
        return combined_value;
    } else if rm_register_with_multiple_registers.contains("-") {
        let combined_value = first_register.updated_value.wrap_sub_and_return_result(second_register.updated_value.value);
        return combined_value;
    } else {
        panic!("Function called with a register that did not contain multiple registers.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listing_0038() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0038_many_register_mov").unwrap();

        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<String> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            // let second_byte = binary_contents[instruction_pointer + 1];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, false);

            decoded_instructions.push(decoded_instruction.formatted_instruction);
        }
        let expected_decoded_instructions = "mov cx, bx\nmov ch, ah\nmov dx, bx\nmov si, bx\nmov bx, di\nmov al, cl\nmov ch, ch\nmov bx, ax\nmov bx, si\nmov sp, di\nmov bp, ax";
        assert_eq!(decoded_instructions.join("\n"), expected_decoded_instructions);
    }

    #[test]
    fn test_listing_0039() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0039_more_movs").unwrap();
        let expected_instructions: Vec<instruction_data> = vec![
            instruction_data {
                formatted_instruction: "mov si, bx".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dh, al".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cl, 12".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(12), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov ch, -12".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(244), is_signed: true },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, 12".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(12), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, -12".to_string(),
                original_value: Value { value: ValueEnum::WordSize(12), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(65524), is_signed: true },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dx, 3948".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3948), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dx, -3948".to_string(),
                original_value: Value { value: ValueEnum::WordSize(3948), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(61588), is_signed: true },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov al, [bx + si]".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bx, [bp + di]".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dx, [bp + 0]".to_string(),
                original_value: Value { value: ValueEnum::WordSize(61588), is_signed: true },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov ah, [bx + si + 4]".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov al, [bx + si + 4999]".to_string(),
                original_value: Value { value: ValueEnum::ByteSize(0), is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov word [bx + di], cx".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(65524), is_signed: true },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov [bp + si], cl".to_string(),
                original_value: Value { value: ValueEnum::ByteSize(244), is_signed: true },
                updated_value: Value { value: ValueEnum::ByteSize(12), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov [bp + 0], ch".to_string(),
                original_value: Value { value: ValueEnum::ByteSize(12), is_signed: false },
                updated_value: Value { value: ValueEnum::ByteSize(244), is_signed: true },
                flags: vec![],
            },
        ];
        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        for (index, instruction) in decoded_instructions.iter().enumerate() {
            assert_eq!(instruction.formatted_instruction, expected_instructions[index].formatted_instruction);
            assert_eq!(instruction.original_value, expected_instructions[index].original_value);
            assert_eq!(instruction.updated_value, expected_instructions[index].updated_value);
            assert_eq!(instruction.flags, expected_instructions[index].flags);
        }
        // assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0041() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0041_add_sub_cmp_jnz").unwrap();
        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<String> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, false);
            decoded_instructions.push(decoded_instruction.formatted_instruction);
        }

        let expected_instructions: Vec<&str> = vec![
            // add instructions
            "add bx, [bx + si]",
            "add bx, [bp + 0]",
            "add si, 2",
            "add bp, 2",
            "add cx, 8",
            "add bx, [bp + 0]",
            "add cx, [bx + 2]",
            "add bh, [bp + si + 4]",
            "add di, [bp + di + 6]",
            "add [bx + si], bx",
            "add [bp + 0], bx",
            "add [bp + 0], bx",
            "add [bx + 2], cx",
            "add [bp + si + 4], bh",
            "add [bp + di + 6], di",
            "add byte [bx], 34",
            "add word [bp + si + 1000], 29",
            "add ax, [bp + 0]",
            "add al, [bx + si]",
            "add ax, bx",
            "add al, ah",
            "add ax, 1000",
            "add al, -30",
            "add al, 9",

            // sub instructions
            "sub bx, [bx + si]",
            "sub bx, [bp + 0]",
            "sub si, 2",
            "sub bp, 2",
            "sub cx, 8",
            "sub bx, [bp + 0]",
            "sub cx, [bx + 2]",
            "sub bh, [bp + si + 4]",
            "sub di, [bp + di + 6]",
            "sub [bx + si], bx",
            "sub [bp + 0], bx",
            "sub [bp + 0], bx",
            "sub [bx + 2], cx",
            "sub [bp + si + 4], bh",
            "sub [bp + di + 6], di",
            "sub byte [bx], 34",
            "sub word [bx + di], 29",
            "sub ax, [bp + 0]",
            "sub al, [bx + si]",
            "sub ax, bx",
            "sub al, ah",
            "sub ax, 1000",
            "sub al, -30",
            "sub al, 9",

            // cmp instructions
            "cmp bx, [bx + si]",
            "cmp bx, [bp + 0]",
            "cmp si, 2",
            "cmp bp, 2",
            "cmp cx, 8",
            "cmp bx, [bp + 0]",
            "cmp cx, [bx + 2]",
            "cmp bh, [bp + si + 4]",
            "cmp di, [bp + di + 6]",
            "cmp [bx + si], bx",
            "cmp [bp + 0], bx",
            "cmp [bp + 0], bx",
            "cmp [bx + 2], cx",
            "cmp [bp + si + 4], bh",
            "cmp [bp + di + 6], di",
            "cmp byte [bx], 34",
            "cmp word [4834], 29",
            "cmp ax, [bp + 0]",
            "cmp al, [bx + si]",
            "cmp ax, bx",
            "cmp al, ah",
            "cmp ax, 1000",
            "cmp al, -30",
            "cmp al, 9",

            // labels and jump instructions
            "jnz 0",
            "jnz -2",
            "jnz -4",
            "jnz -2",
            "je 0",
            "jl -2",
            "jle -4",
            "jb -6",
            "jbe -8",
            "jp -10",
            "jo -12",
            "js -14",
            "jnz -16",
            "jnl -18",
            "jg -20",
            "jnb -22",
            "ja -24",
            "jnp -26",
            "jno -28",
            "jns -30",
            "loop -32",
            "loopz -34",
            "loopnz -36",
            "jcxz -38",
        ];

        for (index, instruction) in decoded_instructions.iter().enumerate() {
            assert_eq!(*instruction, expected_instructions[index].to_string());
        }
        // assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0043() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0043_immediate_movs").unwrap();
        let expected_instructions: Vec<instruction_data> = vec![
            instruction_data {
                formatted_instruction: "mov ax, 1".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bx, 2".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, 3".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dx, 4".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov sp, 5".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(5), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bp, 6".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov si, 7".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(7), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov di, 8".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(8), is_signed: false },
                flags: vec![],
            },
        ];
        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }

        // iterate the decoded instruction in a for loop and assert that the values are as expected
        for (index, instruction) in decoded_instructions.iter().enumerate() {
            assert_eq!(instruction.formatted_instruction, expected_instructions[index].formatted_instruction);
            assert_eq!(instruction.original_value, expected_instructions[index].original_value);
            assert_eq!(instruction.updated_value, expected_instructions[index].updated_value);
            assert_eq!(instruction.flags, expected_instructions[index].flags);
        }
        // assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0044() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0044_register_movs").unwrap();
        let expected_instructions: Vec<instruction_data> = vec![
            // Direct value assignments
            instruction_data {
                formatted_instruction: "mov ax, 1".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bx, 2".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, 3".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov dx, 4".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },

            // Moving values between registers
            instruction_data {
                formatted_instruction: "mov sp, ax".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bp, bx".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov si, cx".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov di, dx".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },

            // Moving values back to other registers
            instruction_data {
                formatted_instruction: "mov dx, sp".to_string(),
                original_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, bp".to_string(),
                original_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bx, si".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov ax, di".to_string(),
                original_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },
        ];
        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0046() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0046_add_sub_cmp").unwrap();
        let expected_instructions: Vec<instruction_data> = vec![
            // Direct value assignments
            instruction_data {
                formatted_instruction: "mov bx, -4093".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(61443), is_signed: true },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, 3841".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3841), is_signed: false },
                flags: vec![],
            },

            // Operation on registers
            instruction_data {
                formatted_instruction: "sub bx, cx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(61443), is_signed: true },
                updated_value: Value { value: ValueEnum::WordSize(57602), is_signed: true },
                flags: vec!["SF"],  // You can add relevant flags affected by the operation if needed.
            },

            // Direct value assignments
            instruction_data {
                formatted_instruction: "mov sp, 998".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(998), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bp, 999".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(999), is_signed: false },
                flags: vec![],
            },

            // Compare operation
            instruction_data {
                formatted_instruction: "cmp bp, sp".to_string(),
                original_value: Value { value: ValueEnum::WordSize(999), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(999), is_signed: false },
                flags: vec![],  // You can add relevant flags affected by the compare operation if needed.
            },

            // Arithmetic operations
            instruction_data {
                formatted_instruction: "add bp, 1027".to_string(),
                original_value: Value { value: ValueEnum::WordSize(999), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2026), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "sub bp, 2026".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2026), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec!["ZF"],
            },
        ];
        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0049() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0049_conditional_jumps").unwrap();
        let expected_instructions: Vec<instruction_data> = vec![
            instruction_data {
                formatted_instruction: "mov cx, 3".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bx, 1000".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1000), is_signed: false },
                flags: vec![],
            },

            // First iteration
            instruction_data {
                formatted_instruction: "add bx, 10".to_string(),
                original_value: Value { value: ValueEnum::WordSize(1000), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1010), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "sub cx, 1".to_string(),
                original_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "jnz -6".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec![],
            },
            // Second iteration
            instruction_data {
                formatted_instruction: "add bx, 10".to_string(),
                original_value: Value { value: ValueEnum::WordSize(1010), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1020), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "sub cx, 1".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "jnz -6".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec![],
            },
            // Third iteration
            instruction_data {
                formatted_instruction: "add bx, 10".to_string(),
                original_value: Value { value: ValueEnum::WordSize(1020), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1030), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "sub cx, 1".to_string(),
                original_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec!["ZF"],  // This operation would set the zero flag since result is 0.
            },
            instruction_data {
                formatted_instruction: "jnz -6".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec!["ZF"],
            },
        ];

        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0051() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0051_memory_mov").unwrap();

        let expected_instructions: Vec<instruction_data> = vec![
        instruction_data {
            formatted_instruction: "mov word [1000], 1".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov word [1002], 2".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov word [1004], 3".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(3), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov word [1006], 4".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov bx, 1000".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(1000), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov word [bx + 4], 10".to_string(),
            original_value: Value { value: ValueEnum::WordSize(3), is_signed: false },  // because [1004] was 3
            updated_value: Value { value: ValueEnum::WordSize(10), is_signed: false },
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov bx, word [1000]".to_string(),
            original_value: Value { value: ValueEnum::WordSize(1000), is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(1), is_signed: false },  // because [1000] was 1
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov cx, word [1002]".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },  // because [1002] was 2
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov dx, word [1004]".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(10), is_signed: false },  // because [1004] was changed to 10
            flags: vec![],
        },
        instruction_data {
            formatted_instruction: "mov bp, word [1006]".to_string(),
            original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
            updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },  // because [1006] was 4
            flags: vec![],
        }
    ];

        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        assert_eq!(decoded_instructions, expected_instructions);
    }

    #[test]
    fn test_listing_0052() {
        let expected_instructions: Vec<instruction_data> = vec![
            instruction_data {
                formatted_instruction: "mov dx, 6".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov bp, 1000".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(1000), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov si, 0".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov word [bp + si], si".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec!["SF"],
            },
            instruction_data{
                formatted_instruction: "jnz -7".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec!["SF"]
            },
            instruction_data {
                formatted_instruction: "mov word [bp + si], si".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },
            // Do it for the rest
            instruction_data {
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec!["SF"],
            },
            instruction_data{
                formatted_instruction: "jnz -7".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec!["SF"]
            },
            instruction_data {
                formatted_instruction: "mov word [bp + si], si".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value { value: ValueEnum::WordSize(4), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                flags: vec!["ZF"],
            },
            instruction_data{
                formatted_instruction: "jnz -7".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                flags: vec!["ZF"]
            },
            instruction_data {
                formatted_instruction: "mov bx, 0".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov si, 0".to_string(),
                original_value: Value { value: ValueEnum::WordSize(6), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "mov cx, [bp + si]".to_string(),
                original_value: Value { value: ValueEnum::Uninitialized, is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "add bx, cx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                flags: vec!["ZF"],
            },
            instruction_data {
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value { value: ValueEnum::WordSize(0), is_signed: false },
                updated_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                flags: vec![],
            },
            instruction_data {
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value { value: ValueEnum::WordSize(2), is_signed: false },
                updated_value: Value { value:  ValueEnum::WordSize(2), is_signed: false },
                flags: vec!["SF"],
            },
            instruction_data{
                formatted_instruction: "jnz -9".to_string(),
                original_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                updated_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                flags: vec!["SF"],
            },
            instruction_data{
                formatted_instruction: "mov cx, [bp + si]".to_string(),
                original_value: Value{value: ValueEnum::WordSize(0), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(2), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "add bx, cx".to_string(),
                original_value: Value{value: ValueEnum::WordSize(0), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(2), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value{value: ValueEnum::WordSize(2), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(4), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value{value: ValueEnum::WordSize(4), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(4), is_signed: false},
                flags: vec!["SF"],
            },
            instruction_data{
                formatted_instruction: "jnz -9".to_string(),
                original_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                updated_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                flags: vec!["SF"],
            }, 
            instruction_data{
                formatted_instruction: "mov cx, [bp + si]".to_string(),
                original_value: Value{value: ValueEnum::WordSize(2), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(4), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "add bx, cx".to_string(),
                original_value: Value{value: ValueEnum::WordSize(2), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(6), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "add si, 2".to_string(),
                original_value: Value{value: ValueEnum::WordSize(4), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(6), is_signed: false},
                flags: vec![],
            },
            instruction_data{
                formatted_instruction: "cmp si, dx".to_string(),
                original_value: Value{value: ValueEnum::WordSize(6), is_signed: false},
                updated_value: Value{value: ValueEnum::WordSize(6), is_signed: false},
                flags: vec!["ZF"],
            },
            instruction_data{
                formatted_instruction: "jnz -9".to_string(),
                original_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                updated_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
                flags: vec!["ZF"],
            }, 
        ];

        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0052_memory_add_loop").unwrap();

        let mut memory: [memory_struct; 100000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct { bits: 0, initialized: false }, original_bits: bits_struct { bits: 0, initialized: false } } }; 100000];

        let mut registers = construct_registers();
        let mut flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions: Vec<instruction_data> = Vec::new();

        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, &mut flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }

        // match the expected_instructions and decoded_instructions and see if they match.
        for (index, expected_instruction) in expected_instructions.iter().enumerate() {
            let decoded_instruction = &decoded_instructions[index];
            assert_eq!(expected_instruction.formatted_instruction, decoded_instruction.formatted_instruction, "Instruction not correct. {}, index {}", expected_instruction.formatted_instruction, index);
            assert_eq!(expected_instruction.original_value, decoded_instruction.original_value, "Instruction not correct. {}, index {}", expected_instruction.formatted_instruction, index);
            assert_eq!(expected_instruction.updated_value, decoded_instruction.updated_value, "Instruction not correct. instruction: {}, index {}", expected_instruction.formatted_instruction, index);
            assert_eq!(expected_instruction.flags, decoded_instruction.flags, "Instruction not correct. instruction: {}, index {}", expected_instruction.formatted_instruction, index);
        }
    }
}
