use crate::bits::{InstructionType, MemoryModeEnum};
use crate::bits::InstructionType::*;

pub struct Register {
   pub register:       &'static str,
   pub updated_value:  i64,
   pub original_value: i64,
}

const REGISTERS: [&str; 16] = [
    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di",
    "al", "cl", "dl", "bl", "ah", "ch", "dh", "bh",
];

pub fn construct_registers() -> Vec<Register>{
    REGISTERS.iter().map(|&register| Register {
        register,
        updated_value: 0,
        original_value: 0,
    }).collect()
}

pub fn get_register_state<'a>(register: &String, registers: &'a Vec<Register>) -> &'a Register {
    assert!(!register.contains("+"), "Multiple registers provided and it's not handled in the calling branch. Registers provided: {]", register);
    assert!(!register.contains("-"), "Multiple registers provided and it's not handled in the calling branch. Registers provided: {}" ,register);
    for reg in registers.iter() {
        if reg.register == register {
            return reg
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register);
}

pub fn update_register_value(register_to_update: &'static str, value: i64, registers: &mut Vec<Register>, instruction: InstructionType, memory_mode: MemoryModeEnum, mnemonic: &'static str) -> () {
    for register in registers.iter_mut() {
        if register.register == register_to_update {
            match instruction {
                ImmediateToAccumulatorADD => register.updated_value += value,
                ImmediateToAccumulatorSUB => register.updated_value -= value,
                ImmediateToRegisterMemory | RegisterMemory => {
                    match memory_mode {
                        MemoryModeEnum::RegisterMode | MemoryModeEnum::MemoryModeNoDisplacement | MemoryModeEnum::MemoryMode8Bit | MemoryModeEnum::MemoryMode16Bit | MemoryModeEnum::DirectMemoryOperation => {
                            match mnemonic {
                                "mov" => register.updated_value = value,
                                "add" => register.updated_value += value,
                                "sub" => register.updated_value -= value,
                                "cmp" => (),
                                _ => panic!("Unknown mnemonic {}", mnemonic),
                            }
                        }
                    }
                    return
                },
                ImmediateToRegisterMOV => register.updated_value = value,
                _ => () // Conditional jumps, CMP instructions.
            }
            return
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register_to_update);
}

pub fn update_original_register_value(register_to_update: &'static str, value: i64, registers: &mut Vec<Register>) -> () {
    for reg in registers.iter_mut() {
        if reg.register == register_to_update {
            reg.original_value = value;
            return
        }
    }
}