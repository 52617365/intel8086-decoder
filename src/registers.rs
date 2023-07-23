use crate::bits::{InstructionType, MemoryModeEnum};
use crate::bits::InstructionType::*;
use crate::flag_registers::{number_is_signed, twos_complement};

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

impl ValueEnum {
    pub fn get_usize(self) -> usize {
        match self {
            Self::ByteSize(val) => val as usize,
            Self::WordSize(val) => val as usize,
            // Self::Uninitialized => panic!("You're trying to get a usize from a value that is not initialized."),
            Self::Uninitialized => 0,

        }
    }
}
impl Value {
    pub fn wrap_add(&mut self, value_src: ValueEnum) {
        let self_value_to_usize = self.value.get_usize(); // we can actually do this because the source type does not matter if it
                                             // does not change the underlying value.
        match self.value {
            ValueEnum::ByteSize(val) => {
                let result_after_wrap = ValueEnum::ByteSize(val.wrapping_add(u8::try_from(self_value_to_usize).expect("we were sure that the value would fit in u8 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                *self = val
            },
            ValueEnum::WordSize(val) => {
                let result_after_wrap = ValueEnum::WordSize(val.wrapping_add(u16::try_from(self_value_to_usize).expect("we were sure that the value would fit in u16 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                *self = val;
            },
            ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
        }
    }

    pub fn wrap_sub(&mut self, value_src: ValueEnum) {
        let self_value_to_usize = self.value.get_usize(); // we can actually do this because the source type does not matter if it
                                                          //
        match self.value {
            ValueEnum::ByteSize(val) => {
                let result_after_wrap = ValueEnum::ByteSize(val.wrapping_sub(u8::try_from(self_value_to_usize).expect("we were sure that the value would fit in u8 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                *self = val
            },
            ValueEnum::WordSize(val) => {
                let result_after_wrap = ValueEnum::WordSize(val.wrapping_sub(u16::try_from(self_value_to_usize).expect("we were sure that the value would fit in u16 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                *self = val;
            },
            ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
        }
    }

    pub fn wrap_add_and_return_result(self) -> Value {
        let self_value_to_usize = self.value.get_usize(); // we can actually do this because the source type does not matter if it
                                             // does not change the underlying value.
        match self.value {
            ValueEnum::ByteSize(val) => {
                let result_after_wrap = ValueEnum::ByteSize(val.wrapping_add(u8::try_from(self_value_to_usize).expect("we were sure that the value would fit in u8 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                return val;
            },
            ValueEnum::WordSize(val) => {
                let result_after_wrap = ValueEnum::WordSize(val.wrapping_add(u16::try_from(self_value_to_usize).expect("we were sure that the value would fit in u16 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                return val;
            },
            ValueEnum::Uninitialized => panic!("this should not be uninitialized."), // TODO: should we even panic here? I guess it's just normal behavior, right? or maybe we should panic but we should check for uninitialized in the caller?
        }
    }
    pub fn wrap_sub_and_return_result(self) -> Value {
        let self_value_to_usize = self.value.get_usize(); // we can actually do this because the source type does not matter if it
                                                          //
        match self.value {
            ValueEnum::ByteSize(val) => {
                let result_after_wrap = ValueEnum::ByteSize(val.wrapping_sub(u8::try_from(self_value_to_usize).expect("we were sure that the value would fit in u8 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                return val;
            },
            ValueEnum::WordSize(val) => {
                let result_after_wrap = ValueEnum::WordSize(val.wrapping_sub(u16::try_from(self_value_to_usize).expect("we were sure that the value would fit in u16 but it didn't.")));
                let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
                return val;
            },
            ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
        }
    }

    pub fn get_decimal_number_from_bits(self) -> i64 {
        if self.is_signed {
            match self.value {
                ValueEnum::ByteSize(val) => {
                    return i64::try_from(self.twos_complement_8_bit(val)).unwrap();
                },
                ValueEnum::WordSize(val) => {
                    return i64::try_from(self.twos_complement_16_bit(val)).unwrap();
                },
                ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
            }
        } else {
            return i64::try_from(self.value.get_usize()).expect("if we cast here we are losing data since usize -> i64 and i64 unsigned bit is half the size.");
        }
    }

    fn twos_complement_8_bit(self, num: u8) -> i8 {
        (!num).wrapping_add(1) as i8
    }
    fn twos_complement_16_bit(self, num: u16) -> i16 {
        (!num).wrapping_add(1) as i16
    }

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

// pub fn wrap_add_to_out_value(value_src: ValueEnum, value_dst: &mut Value) {
//     let casted_value = match value_src { // Here we cast the value because it does not matter at this
//                                          // point since the number will stay the same.
//         ValueEnum::ByteSize(val)  => {
//             val as usize
//         },
//         ValueEnum::WordSize(val) => {
//             val as usize
//         },
//         ValueEnum::Uninitialized => panic!("value should be initialized"),
//     };
//
//     match value_dst.value {
//         ValueEnum::ByteSize(val) => {
//             let result_after_wrap = ValueEnum::ByteSize(val.wrapping_add(u8::try_from(casted_value).expect("we were sure that the value would fit in u8 but it didn't.")));
//             let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
//             *value_dst = val
//         },
//         ValueEnum::WordSize(val) => {
//             let result_after_wrap = ValueEnum::WordSize(val.wrapping_add(u16::try_from(casted_value).expect("we were sure that the value would fit in u16 but it didn't.")));
//             let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
//             *value_dst = val;
//         },
//         ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
//     }
// }

// pub fn wrap_sub_to_out_value(value_src: ValueEnum, value_dst: &mut Value) {
//     let casted_value = match value_src { // Here we cast the value because it does not matter at this
//                                          // point since the number will stay the same.
//         ValueEnum::ByteSize(val)  => {
//             val as usize
//         },
//         ValueEnum::WordSize(val) => {
//             val as usize
//         },
//         ValueEnum::Uninitialized => panic!("value should be initialized"),
//     };
//
//     match value_dst.value {
//         ValueEnum::ByteSize(val) => {
//             let result_after_wrap = ValueEnum::ByteSize(val.wrapping_sub(u8::try_from(casted_value).expect("we were sure that the value would fit in u8 but it didn't.")));
//             let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
//             *value_dst = val
//         },
//         ValueEnum::WordSize(val) => {
//             let result_after_wrap = ValueEnum::WordSize(val.wrapping_sub(u16::try_from(casted_value).expect("we were sure that the value would fit in u16 but it didn't.")));
//             let val = Value{value: result_after_wrap, is_signed: number_is_signed(result_after_wrap)};
//             *value_dst = val;
//         },
//         ValueEnum::Uninitialized => panic!("this should not be uninitialized."),
//     }
// }
pub fn update_register_value(register_to_update: &str, value: ValueEnum, registers: &mut Vec<Register>, instruction: InstructionType, memory_mode: MemoryModeEnum, mnemonic: &'static str, is_word_size: bool) -> () {
    for register in registers.iter_mut() {
        if register.register == register_to_update {
            match instruction {
                ImmediateToAccumulatorADD => {
                    register.updated_value.wrap_add(value);
                },
                ImmediateToAccumulatorSUB => {
                    register.updated_value.wrap_sub(value);
                }
                ImmediateToRegisterMemory | RegisterMemory => {
                    match memory_mode {
                        MemoryModeEnum::RegisterMode | MemoryModeEnum::MemoryModeNoDisplacement | MemoryModeEnum::MemoryMode8Bit | MemoryModeEnum::MemoryMode16Bit | MemoryModeEnum::DirectMemoryOperation => {
                            match mnemonic {
                                "mov" => register.updated_value = Value{value, is_signed: number_is_signed(value)},
                                "add" => register.updated_value.wrap_add(value),
                                "sub" => register.updated_value.wrap_sub(value),
                                "cmp" => (),
                                _ => panic!("Unknown mnemonic {}", mnemonic),
                            }
                        }
                    }
                    return
                },
                ImmediateToRegisterMOV => register.updated_value = Value{value, is_signed: number_is_signed(value)},
                _ => () // Conditional jumps, CMP instructions.
            }
            return
        }
    }
    panic!("Register not found, this should never happen. Register that was not found was {}", register_to_update);
}

pub fn update_original_register_value(register_to_update: &'static str, value: ValueEnum, registers: &mut Vec<Register>, is_word_size: bool) -> () {
    if let ValueEnum::Uninitialized  = value {return}
    for reg in registers.iter_mut() {
        if reg.register == register_to_update {
            reg.original_value = Value { value, is_signed: number_is_signed(value)
        };
    }
    return
    }
}
