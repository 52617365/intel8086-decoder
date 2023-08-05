mod bits;
mod registers;
mod flag_registers;
mod memory;

/*
TODO: On top of the testing we want to do, we also need to support the old homework because during the newer homework, the old ones broke.
*/

use bits::*;

use crate::memory::{bits_struct, get_displacement, load_memory_contents_as_decimal_and_optionally_update_original_value, memory_contents, memory_struct, store_memory_value};
use crate::bits::combine_bytes; use core::panic; use std::{env, fs};
use crate::bits::InstructionType::{ImmediateToAccumulatorADD, ImmediateToAccumulatorCMP, ImmediateToRegisterMemory, ImmediateToRegisterMOV, ImmediateToAccumulatorSUB, RegisterMemory, JE_JUMP, JL_JUMP, JLE_JUMP, JB_JUMP, JBE_JUMP, JP_JUMP, JO_JUMP, JS_JUMP, JNE_JUMP, JNL_JUMP, LOOP, LOOPZ, JCXZ, LOOPNZ, JNS, JNO_JUMP, JNBE_JUMP, JNP_JUMP, JNB_JUMP, JNLE_JUMP};
use crate::bits::Masks::{D_BITS, IMMEDIATE_TO_REG_MOV_W_BIT};

use crate::flag_registers::number_is_signed;
use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement, RegisterMode};
use crate::registers::{Value, ValueEnum, construct_registers, get_register_state, Register, register_contains_multiple_registers, update_original_register_value, update_register_value};
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
        todo!()
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();

    let mut memory: [memory_struct; 64000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct{bits: 0, initialized: false}, original_bits: bits_struct{bits: 0, initialized: false}} }; 64000];

    let mut registers = construct_registers();
    let op_codes = construct_opcodes();
    let flag_registers = construct_flag_registers();

    let mut old_instruction_pointer: usize = 0;
    let mut instruction_pointer: usize = 0;
    let simulate_code = true;
    while instruction_pointer < binary_contents.len() {
        old_instruction_pointer = instruction_pointer;
        let first_byte = binary_contents[instruction_pointer];
        let instruction = determine_instruction(&op_codes, first_byte);
        let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, flag_registers, &mut memory, &mut instruction_pointer, simulate_code);

        if simulate_code {
            println!("{} | {} -> {} | flags: {:?}, IP: {} -> {}", decoded_instruction.formatted_instruction, decoded_instruction.original_value.get_string_number_from_bits(), decoded_instruction.updated_value.get_string_number_from_bits(), decoded_instruction.flags, old_instruction_pointer, instruction_pointer);
        } else {
            println!("{}", decoded_instruction.formatted_instruction);
        }
    }
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
fn instruction_has_immediate_value_in_rm_register(instruction: InstructionType, memory_mode: MemoryModeEnum) -> bool {
    return memory_mode != DirectMemoryOperation && instruction == ImmediateToRegisterMOV
}

fn instruction_has_immediate_value_in_reg_register(instruction: InstructionType) -> bool {
    return instruction == ImmediateToRegisterMemory || instruction == ImmediateToAccumulatorADD || instruction == ImmediateToAccumulatorSUB || instruction == ImmediateToAccumulatorCMP
}


fn get_immediate_from_rm_register(instruction: InstructionType, is_word_size: bool, memory_mode: MemoryModeEnum, instruction_pointer: usize, binary_contents: &Vec<u8>) -> Value {
    if memory_mode != DirectMemoryOperation {
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
    }
    panic!("We thought that the rm register contained an immediate when it did not.")
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
                        let value = ValueEnum::WordSize(combined as u16);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };
                    } else if memory_mode == MemoryMode16Bit || memory_mode == DirectMemoryOperation {
                        // the immediate is guaranteed to be 16-bit because the s bit is set to 0 in this branch.
                        let fifth_byte = binary_contents[instruction_pointer + 4];
                        let sixth_byte = binary_contents[instruction_pointer + 5];
                        let combined = combine_bytes(sixth_byte, fifth_byte);
                        let value = ValueEnum::WordSize(combined as u16);
                        return Value{
                            value, 
                            is_signed: number_is_signed(value),
                        };
                    } else {
                        let third_byte = binary_contents[instruction_pointer + 2];
                        let fourth_byte = binary_contents[instruction_pointer + 3];
                        let combined = combine_bytes(fourth_byte, third_byte);

                        let value = ValueEnum::WordSize(combined as u16);
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
                        let value = ValueEnum::ByteSize(fifth_byte as u8);
                        return Value{
                            value,
                            is_signed: number_is_signed(value),
                        };

                    } else {
                        let third_byte = binary_contents[instruction_pointer + 2];

                        let value = ValueEnum::ByteSize(third_byte as u8);
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

fn decode_instruction(binary_contents: &Vec<u8>, instruction: InstructionType, registers: &mut Vec<Register>, mut flag_registers: [FlagRegister; 2], memory: &mut [memory_struct; 64000], instruction_pointer: &mut usize, simulate: bool) -> instruction_data {
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

    if instruction_has_immediate_value_in_rm_register(instruction, memory_mode) {
        rm_immediate = get_immediate_from_rm_register(instruction, is_word_size, memory_mode, *instruction_pointer, &binary_contents)
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
                        if let ValueEnum::Uninitialized = rm.value {} else {
                            let rm_value_casted = match rm.value {
                                ValueEnum::ByteSize(val) => val as usize,
                                ValueEnum::WordSize(val) => val as usize,
                                ValueEnum::Uninitialized => panic!("We should not be initialized here because we checked for this before.")
                            };
                            store_memory_value(memory, rm_value_casted, 0, reg_immediate, mnemonic, is_word_size);
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
                        value = combine_register_containing_multiple_registers(registers, &rm_register).value;
                    } else {
                        let rm = get_register_state(&rm_register, registers);
                        value = rm.updated_value.value;
                    }
                } else if instruction == RegisterMemory && instruction_uses_memory(memory_mode) {
                    if memory_mode == DirectMemoryOperation {
                        if reg_is_dest {
                            let reg = get_register_state(&reg_register, registers);
                            value = reg.updated_value.value;
                        } else {
                            let rm = get_register_state(&rm_register, registers);
                            value = rm.updated_value.value;
                        }
                    }
                } else {
                    if reg_is_dest && instruction != ImmediateToRegisterMemory {
                        let reg = get_register_state(&reg_register, registers);
                        value = reg.updated_value.value;
                    } else {
                        let rm = get_register_state(&rm_register, registers);
                        value = rm.updated_value.value;
                    }
                }
                set_flags(value, &mut flag_registers);
            } else {
                // We don't clear if it's a conditional jump because the jnz conditional jump for example relies on the flags to know when to stop jumping.
                clear_flags_registers(&mut flag_registers);
            }
        }
    }

    let formatted_instruction = format_instruction(&binary_contents, *instruction_pointer, first_byte, second_byte, instruction, mnemonic, is_word_size, memory_mode, reg_is_dest, &reg_register, &rm_register, reg_immediate, rm_immediate);


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


    if reg_is_dest && instruction != ImmediateToRegisterMemory || instruction == ImmediateToRegisterMOV {
        let reg = get_register_state(&reg_register, &registers);
        update_original_register_value(reg.register, reg.updated_value.value, registers);
    } else if instruction_uses_memory(memory_mode) {
        // TODO: fill
        // NOTE: We update the original_bits of memory inside the get_memory_contents_as_decimal_and_update_original_value function that
        //  returns a struct of the original and updated_value, should we do the same with registers? It could be easier that way.
    } else {
        let rm = get_register_state(&rm_register, &registers);
        update_original_register_value(rm.register, rm.updated_value.value, registers);
    }

    assert_ne!(instruction_details.formatted_instruction, "", "instruction_details struct is not initialized, this should never happen.");

    if instruction_is_conditional_jump(instruction) && simulate {
        perform_conditional_jump(&mut flag_registers, instruction_size, instruction_pointer, second_byte, instruction);
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

// TODO: why is perform_conditional_jump making an infinite loop?
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
        let jump = second_byte as usize + instruction_size;
        *instruction_pointer -= jump
    }
}

fn format_instruction(binary_contents: &Vec<u8>, ip: usize, first_byte: u8, second_byte: u8, instruction: InstructionType, mnemonic: &str, is_word_size: bool, memory_mode: MemoryModeEnum, reg_is_dest: bool, reg_register: &String, rm_register: &String, reg_immediate: Value, rm_immediate: Value) -> String {
    if instruction == ImmediateToRegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if is_word_size {
                return format!("{} word [{}], {}", mnemonic, rm_register, reg_immediate.get_string_number_from_bits());
            } else {
                return format!("{} byte [{}], {}", mnemonic, rm_register, reg_immediate.get_string_number_from_bits());
            }
        } else if memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit {
            let displacement = get_displacement(binary_contents, ip, memory_mode);
            // let displacement = get_8_bit_displacement(binary_contents, ip);
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
        return format!("{} {}, {}", mnemonic, ax_or_al, reg_immediate.value.get_usize());
    } else if instruction == RegisterMemory {
        if memory_mode == MemoryModeNoDisplacement {
            if reg_is_dest {
                return format!("{} {}, [{}]", mnemonic, reg_register, rm_register)
            } else {
                return format!("{} [{}], {}", mnemonic, rm_register, reg_register)
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
        return format!("{} {}", mnemonic, second_byte as usize);
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
    use crate::memory::word_sized_value_bytes;
    use super::*;

    #[test]
    fn test_listing_0038() {
        let binary_contents = fs::read("/Users/rase/dev/intel8086-decoder/computer_enhance/perfaware/part1/listing_0038_many_register_mov").unwrap();

        let mut memory: [memory_struct; 64000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct{bits: 0, initialized: false}, original_bits: bits_struct{bits: 0, initialized: false}} }; 64000];

        let mut registers = construct_registers();
        let flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions : Vec<String> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            // let second_byte = binary_contents[instruction_pointer + 1];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, flag_registers, &mut memory, &mut instruction_pointer, false);

            decoded_instructions.push(decoded_instruction.formatted_instruction);

            // println!("{} | {} -> {} | flags: {:?}, IP: {} -> {}", decoded_instruction.formatted_instruction, decoded_instruction.original_value, decoded_instruction.updated_value, get_all_currently_set_flags(flag_registers), decoded_instruction.original_instruction_pointer, decoded_instruction.updated_instruction_pointer);

            // if instruction_is_conditional_jump(instruction) {
            //     perform_conditional_jump(&mut flag_registers, instruction_size, &mut instruction_pointer, second_byte, instruction);
            // }
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
                formatted_instruction: "mov [bx + di], cx".to_string(),
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
        let mut memory: [memory_struct; 64000] = [memory_struct { address_contents: memory_contents { modified_bits: bits_struct{bits: 0, initialized: false}, original_bits: bits_struct{bits: 0, initialized: false}} }; 64000];

        let mut registers = construct_registers();
        let flag_registers = construct_flag_registers();
        let op_codes = construct_opcodes();
        let mut instruction_pointer: usize = 0;

        let mut decoded_instructions : Vec<instruction_data> = Vec::new();
        while instruction_pointer < binary_contents.len() {
            let first_byte = binary_contents[instruction_pointer];
            let instruction = determine_instruction(&op_codes, first_byte);
            let decoded_instruction = decode_instruction(&binary_contents, instruction, &mut registers, flag_registers, &mut memory, &mut instruction_pointer, true);
            decoded_instructions.push(decoded_instruction);
        }
        assert_eq!(decoded_instructions, expected_instructions);

        // let expected_decoded_instructions = "mov si, bx\nmov dh, al\nmov cl, 12\nmov ch, -12\nmov cx, 12\nmov cx, -12\nmov dx, 3948\nmov dx, -3948\nmov al, [bx + si]\nmov bx, [bp + di]\nmov dx, [bp + 0]\nmov ah, [bx + si + 4]\nmov al, [bx + si + 4999]\nmov [bx + di], cx\nmov [bp + si], cl\nmov [bp + 0], ch";
        // assert_eq!(decoded_instructions.join("\n"), expected_decoded_instructions);
    }
}
