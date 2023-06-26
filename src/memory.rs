use crate::bits::{MemoryModeEnum, combine_bytes, InstructionType};
use crate::bits::MemoryModeEnum::{DirectMemoryOperation, MemoryModeNoDisplacement};

pub fn load_memory(memory: &[u8], memory_mode: MemoryModeEnum, memory_address: usize, displacement: usize, is_word_size: bool) -> usize {
    assert!(memory_address < memory.len(), "Address was larger than than the available memory.");

    let mut memory_address = memory_address;
    if memory_mode == MemoryModeEnum::MemoryMode8Bit || memory_mode == MemoryModeEnum::MemoryMode16Bit {
        memory_address += displacement;
    }

    if memory_mode == DirectMemoryOperation || memory_mode == MemoryModeEnum::MemoryModeNoDisplacement || memory_mode == MemoryModeEnum::MemoryMode8Bit || memory_mode == MemoryModeEnum::MemoryMode16Bit {
        if is_word_size {
            let first_byte = memory[memory_address];
            let second_byte = memory[memory_address + 1];
            let combined = combine_bytes(second_byte, first_byte);
            return combined as usize;
        } else {
            let address_with_displacement = memory_address + displacement;
            return memory[address_with_displacement] as usize
        }
    } else {
        panic!("load_memory was called when the memory_mode was {:?} and this is unexpected", memory_mode);
    }
}

// TODO: store_memory_value currently has a problem where subtractions can go under 0. this is bad
// because we need to do shift operations on the numbers and it will lead to unexpected shit.
pub fn store_memory_value(memory: &mut [u8; 64000], memory_mode: MemoryModeEnum, memory_address: usize, displacement: usize, value: i64, instruction: InstructionType, mnemonic: &'static str, is_word_size: bool) -> () {
    let mut updated_memory_address = memory_address;
    if memory_mode == MemoryModeEnum::MemoryMode8Bit || memory_mode == MemoryModeEnum::MemoryMode16Bit {
        updated_memory_address += displacement;
    }

    let mut updated_value: i64 = 0;
    if mnemonic == "mov" {
        updated_value = value;
    } else if mnemonic == "add" || mnemonic == "sub" {
        let mut memory_contents: i64;

        if is_word_size {
            memory_contents = combine_bytes(memory[memory_address + 1], memory[memory_address]) as i64;
        } else {
            memory_contents = memory[memory_address] as i64;
        }

        if mnemonic == "add" {
            updated_value = memory_contents + value;
        } else {
            updated_value = memory_contents - value;
        }
    }



    if is_word_size && mnemonic != "cmp" {
        assert!(updated_value > 0, "we're trying to cast a negative value into an usize.");
        assert!(updated_value < u16::MAX as i64, "we're trying to cast a value higher than u16 into an u16.");
        let memory_contents = separate_word_sized_value_into_bytes(updated_value as usize);

        // TODO: make sure this is the correct order.
        memory[memory_address] = memory_contents.lower_byte;
        memory[memory_address + 1] = memory_contents.upper_byte; 
    } else {
        assert!(updated_value < u8::MAX as i64, "we're trying to cast a value higher than u8 into an u8.");
        memory[memory_address] = updated_value as u8;
    }
}

struct word_sized_value_bytes {
    lower_byte: u8,
    upper_byte: u8,
}

// function takes in a 16-bit value usize value and converts it into a 2 u8 bytes so we can store
// it in 2 indices in memory.
fn separate_word_sized_value_into_bytes(value: usize) -> word_sized_value_bytes {
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
    } else if memory_mode == MemoryModeEnum::MemoryMode8Bit {
        return get_8_bit_displacement(binary_contents, i);
    } else if memory_mode == MemoryModeEnum::MemoryMode16Bit || memory_mode == MemoryModeEnum::DirectMemoryOperation {
        return get_16_bit_displacement(binary_contents, i);
    } else {
        panic!("get_displacement was called when the memory_mode was {:?} and this is unexpected", memory_mode);
    }
}

pub fn get_16_bit_displacement(binary_contents: &Vec<u8>, i: usize) -> usize {
    let first_disp = binary_contents[i + 2];
    let second_disp = binary_contents[i + 3];
    let displacement = combine_bytes(second_disp, first_disp);
    displacement as usize
}

pub fn get_8_bit_displacement(binary_contents: &Vec<u8>, i: usize) -> usize {
    let first_disp = binary_contents[i + 2];
    return first_disp as usize
}
