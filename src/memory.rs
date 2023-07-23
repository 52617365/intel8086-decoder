use crate::bits::{MemoryModeEnum, combine_bytes};
use crate::registers::{ValueEnum, Value};
use crate::flag_registers::number_is_signed;



use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryMode16Bit, MemoryMode8Bit, MemoryModeNoDisplacement};

// The memory struct is used by the main loop to simulate memory.
// It's simulated by holding a large array of memory structs.
#[derive(Copy, Clone)]
pub struct memory_struct {
    // pub address: usize,
    pub address_contents: memory_contents
}

// The memory_contents struct holds both the original_bits and modified_bits because
// at the end of the main loop we want to signal to the user how the memory contents
// was modified during the instruction by converting 1 or 2 bytes depending on the
// instruction size into a decimal number.
// To the user the output will then look like: mov [1000], 30 | [1000] 0 -> 30
#[derive(Copy, Clone)]
pub struct memory_contents {
   pub original_bits: u8,
   pub modified_bits: u8,
}


// The fields in decimal_memory_contents get populated with either 1 or 2 bytes depending on the instruction size.
// This is the field that is used to represent the decimal values that have been converted from memory_contents.
pub struct decimal_memory_contents {
    pub original_value: Value,
    pub modified_value: Value,
}

// Some instructions have a displacement which means the memory address is actually the memory address + displacement. We're handling it in this function.
fn adjust_memory_address(memory_mode: MemoryModeEnum, memory_address: usize, displacement: usize) -> usize {
    match memory_mode {
        MemoryMode8Bit | MemoryMode16Bit | DirectMemoryOperation => {
            memory_address + displacement
        },
        _ => memory_address,
    }
}

pub fn load_memory_contents_as_decimal_and_optionally_update_original_value(memory: &mut [memory_struct], memory_mode: MemoryModeEnum, memory_address: usize, displacement: usize, is_word_size: bool, update_original_value: bool) -> decimal_memory_contents {
    assert!(memory_address < memory.len(), "Address was larger than than the available memory.");

    let m_memory_address = adjust_memory_address(memory_mode, memory_address, displacement);

    if memory_mode == DirectMemoryOperation || memory_mode == MemoryModeNoDisplacement || memory_mode == MemoryMode8Bit || memory_mode == MemoryMode16Bit {
        if is_word_size {
            let first_byte = memory[m_memory_address];
            let second_byte = memory[m_memory_address + 1];
            let original_value_combined = combine_bytes(second_byte.address_contents.original_bits, first_byte.address_contents.original_bits);
            let modified_value_combined = combine_bytes(second_byte.address_contents.modified_bits, first_byte.address_contents.modified_bits);

            let original_value = ValueEnum::WordSize(original_value_combined as u16);
            let modified_value = ValueEnum::WordSize(modified_value_combined as u16);

            let decimal_memory_contents =  decimal_memory_contents{
                original_value: Value{value: original_value, is_signed: number_is_signed(original_value)},
                modified_value: Value{value: modified_value, is_signed: number_is_signed(modified_value)}           
            };
            if update_original_value { // This is true only when the destination is a memory location.
                memory[m_memory_address].address_contents.original_bits = memory[m_memory_address].address_contents.modified_bits;
                memory[m_memory_address + 1].address_contents.original_bits = memory[m_memory_address + 1].address_contents.modified_bits;
            }
            return decimal_memory_contents;
        } else {
            let address_with_displacement = m_memory_address + displacement;
            let original_value = memory[address_with_displacement].address_contents.original_bits as usize;
            let modified_value = memory[address_with_displacement].address_contents.modified_bits as usize;

            let original_value = ValueEnum::ByteSize(original_value as u8);
            let modified_value = ValueEnum::ByteSize(modified_value as u8);

            let decimal_memory_contents = decimal_memory_contents{
                original_value: Value{value: original_value, is_signed: number_is_signed(original_value)},
                modified_value: Value{value: modified_value, is_signed: number_is_signed(modified_value)}           
            };

            if update_original_value { // This is true only when the destination is a memory location.
                memory[m_memory_address].address_contents.original_bits = memory[m_memory_address].address_contents.modified_bits;
            }

            return decimal_memory_contents;
        }
    } else {
        panic!("load_memory was called when the memory_mode was {:?} and this is unexpected", memory_mode);
    }
}

pub fn store_memory_value(memory: &mut [memory_struct], memory_mode: MemoryModeEnum, memory_address: usize, displacement: usize, value: Value, mnemonic: &'static str, is_word_size: bool) -> () {
    let mut updated_memory_address = memory_address;
    updated_memory_address += displacement;

    let updated_value: Value;
    if mnemonic == "mov" {
        updated_value = value;
    } else if mnemonic == "add" || mnemonic == "sub" {
        let memory_contents: Value;

        if is_word_size {
            let combined = combine_bytes(memory[memory_address + 1].address_contents.original_bits, memory[memory_address].address_contents.original_bits);
            let val = ValueEnum::WordSize(combined);
            memory_contents = Value{value: val, is_signed: number_is_signed(val)};
        } else {
            let val = ValueEnum::ByteSize(memory[memory_address].address_contents.original_bits);
            memory_contents = Value{value: val, is_signed: number_is_signed(val)};
        }

        if mnemonic == "add" {
            updated_value = memory_contents.wrap_add_and_return_result(value.value);
        } else {
            updated_value = memory_contents.wrap_sub_and_return_result(value.value);
        }



        if let ValueEnum::WordSize(val) = updated_value.value {
            if mnemonic != "cmp" {
                let memory_contents = separate_word_sized_value_into_bytes(usize::try_from(val).unwrap());

                memory[updated_memory_address].address_contents.modified_bits = memory_contents.lower_byte;
                memory[updated_memory_address + 1].address_contents.modified_bits = memory_contents.upper_byte;
            }
        } else if let ValueEnum::ByteSize(val) = updated_value.value {
            memory[updated_memory_address].address_contents.modified_bits = val as u8;
        } else {
            panic!("we should not get here ever");
        }
    }
}

pub struct word_sized_value_bytes {
    pub lower_byte: u8,
    pub upper_byte: u8,
}

// function takes in a 16-bit value usize value and converts it into a 2 u8 bytes so we can store
// it in 2 indices in memory.
pub fn separate_word_sized_value_into_bytes(value: usize) -> word_sized_value_bytes {
    let lower_byte: u8 = (value & 0xFF) as u8;
    let upper_byte: u8 = ((value >> 8) & 0xFF) as u8;

    return word_sized_value_bytes{
        lower_byte,
        upper_byte
    };
}

pub fn get_displacement(binary_contents: &Vec<u8>, i: usize, memory_mode: MemoryModeEnum) -> usize {
    if memory_mode == MemoryModeNoDisplacement {
        return 0;
    } else if memory_mode == MemoryMode8Bit {
        return get_8_bit_displacement(binary_contents, i);
    } else if memory_mode == MemoryMode16Bit || memory_mode == DirectMemoryOperation {
        return get_16_bit_displacement(binary_contents, i);
    } else {
        panic!("get_displacement was called when the memory_mode was {:?} and this is unexpected", memory_mode);
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

