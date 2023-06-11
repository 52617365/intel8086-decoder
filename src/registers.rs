// TODO: Next we need to emulate the instructions, this means that we need to keep state of the values passed into the registers/memory locations.
// it could possibly be wise to create a vector of structs that contain the register name, and the value associated with it.
// we can then do a linear loop through the vector to see if the register we care about matches and get the value out that way.
// it could also have a field that is like og_value that gets updated each time the value gets changed.

use crate::bits::{instruction_is_conditional_jump, instruction_is_immediate_to_register, InstructionType, MemoryModeEnum};
use crate::bits::InstructionType::*;

// we could construct this struct for each register at the start and then just iterate over the collection again and again.
pub struct Register {
   pub register:       &'static str,
   pub updated_value:  usize,
   pub original_value: usize,
}

const REGISTERS: [&str; 16] = [
    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di",
    "al", "cl", "dl", "bl", "ah", "ch", "dh", "bh",
];
// TODO: add all the possible registers in some constant array and then iterate over it in this function and construct a Register struct from each one, appending it to the vector.
// We then linearly search through the vector to find the register we care about and get the value out of it.
// This is done so that we can keep a state of the value of the register.
// we want to emulate the registers and their values during moves etc.
pub fn construct_registers() -> Vec<Register>{
    let mut registers: Vec<Register> = Vec::with_capacity(REGISTERS.len());

    for register in REGISTERS.iter() {
        registers.push( Register {
            register,
            updated_value: 0,
            original_value: 0,
        });
    }
    return registers;
}

pub fn get_register_state<'a>(register: &String, registers: &'a Vec<Register>) -> &'a Register {
    for reg in registers.iter() {
        if reg.register == register {
            return reg
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register);
}

pub fn update_register_value(register_to_update: &'static str, value: usize, registers: &mut Vec<Register>, instruction: InstructionType, memory_mode: MemoryModeEnum, mnemonic: &'static str) -> () {
    for reg in registers.iter_mut() {
        if reg.register == register_to_update {
            match instruction {
                ImmediateToAccumulatorADD => reg.updated_value += value,
                ImmediateToAccumulatorSUB => reg.updated_value -= value,
                ImmediateToRegisterMemory | RegisterMemory => {
                    match memory_mode {
                        MemoryModeEnum::RegisterMode => {
                            match mnemonic {
                                "mov" => reg.updated_value = value,
                                "add" => reg.updated_value += value,
                                "sub" => reg.updated_value -= value,
                                "cmp" => (),
                                _ => panic!("Unknown mnemonic {}", mnemonic),
                            }
                        }
                        MemoryModeEnum::MemoryModeNoDisplacement | MemoryModeEnum::MemoryMode8Bit | MemoryModeEnum::MemoryMode16Bit | MemoryModeEnum::DirectMemoryOperation => (),
                    }
                    return
                },
                ImmediateToRegisterMOV => reg.updated_value = value,
                _ => () // Conditional jumps, CMP instructions.
            }
            return
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register_to_update);
}

pub fn update_original_register_value(register_to_update: &'static str, value: usize, registers: &mut Vec<Register>) -> () {
    for reg in registers.iter_mut() {
        if reg.register == register_to_update {
            reg.original_value = value;
            return
        }
    }
}