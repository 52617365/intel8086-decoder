use crate::bits::{InstructionType, MemoryModeEnum};
use crate::bits::InstructionType::*;
use crate::flag_registers::number_is_signed;

#[derive(Copy, Clone)]
pub enum ValueEnum {
    ByteSize(u8),
    WordSize(u16),
    Uninitialized,
}

#[derive(Copy, Clone)]
pub struct Value {
    pub value: ValueEnum,
    pub is_signed: bool,
}

#[derive(Copy, Clone)]
pub struct Register {
   pub register:       &'static str,
   pub updated_value:  Value, // Should these be a struct containing signed information instead? 
   pub original_value: Value, // x
}

const REGISTERS: [&str; 16] = [
    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di",
    "al", "cl", "dl", "bl", "ah", "ch", "dh", "bh",
];

pub fn construct_registers() -> Vec<Register>{
    REGISTERS.iter().map(|&register| Register {
        register,
        updated_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
        original_value: Value{value: ValueEnum::Uninitialized, is_signed: false},
    }).collect()
}


pub fn register_contains_multiple_registers(register: &str) -> bool {
    return register.contains("+") || register.contains("-")
}

pub fn get_register_state(register: &str, registers: &Vec<Register>) -> Register {
    assert!(!register_contains_multiple_registers(register), "Register contained multiple registers, it should be handled in the caller.");
    for reg in registers.iter() {
        if reg.register == register {
            return reg.clone()
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register);
}

pub fn update_register_value(register_to_update: &str, value: ValueEnum, registers: &mut Vec<Register>, instruction: InstructionType, memory_mode: MemoryModeEnum, mnemonic: &'static str, is_word_size: bool) -> () {
    // TODO: iterate the value: ValueEnum and look if it's a 8 or 16 bit value.
    for register in registers.iter_mut() {
        if register.register == register_to_update {
            match instruction {
                ImmediateToAccumulatorADD => register.updated_value = Value{value: register.updated_value.value.wrapping_add(value), is_signed: number_is_signed(value, is_word_size)},
                ImmediateToAccumulatorSUB => register.updated_value = Value{value: register.updated_value.value.wrapping_sub(value), is_signed: number_is_signed(value, is_word_size)},
                ImmediateToRegisterMemory | RegisterMemory => {
                    match memory_mode {
                        MemoryModeEnum::RegisterMode | MemoryModeEnum::MemoryModeNoDisplacement | MemoryModeEnum::MemoryMode8Bit | MemoryModeEnum::MemoryMode16Bit | MemoryModeEnum::DirectMemoryOperation => {
                            match mnemonic {
                                "mov" => register.updated_value = Value{value, is_signed: number_is_signed(value, is_word_size)},
                                "add" => register.updated_value = Value{value: register.updated_value.value.wrapping_add(value), is_signed: number_is_signed(value, is_word_size)},
                                "sub" => register.updated_value = Value{value: register.updated_value.value.wrapping_sub(value), is_signed: number_is_signed(value, is_word_size)},
                                "cmp" => (),
                                _ => panic!("Unknown mnemonic {}", mnemonic),
                            }
                        }
                    }
                    return
                },
                ImmediateToRegisterMOV => register.updated_value = Value{value, is_signed: number_is_signed(value, is_word_size)},
                _ => () // Conditional jumps, CMP instructions.
            }
            return
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register_to_update);
}

// TODO: related to the casting problem. Should this also update a struct containing the
// original_value and also information if the number is signed or not? This would help us get away
// from the unnecessary casting?
pub fn update_original_register_value(register_to_update: &'static str, value: ValueEnum, registers: &mut Vec<Register>, is_word_size: bool) -> () {
    for reg in registers.iter_mut() {
        if reg.register == register_to_update {
            reg.original_value = Value{value, is_signed: number_is_signed(value)};
            return
        }
    }
}
