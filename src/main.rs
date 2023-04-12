use core::panic;
use std::{env, fs};

fn determine_register(byte: u8, reg: bool, is_word_size: bool) -> &'static str {
    if reg {
        return get_reg_register(byte, is_word_size);
    } else {
        return get_rm_register(byte, is_word_size);
    }
}

fn get_reg_register(byte: u8, is_word_size: bool) -> &'static str {
    const REGISTER_MASK: u8 = 0b00_111_000;
    let result = byte & REGISTER_MASK;
    if is_word_size {
        return match result {
            0b00_000_000 => "ax",
            0b00_001_000 => "cx",
            0b00_010_000 => "dx",
            0b00_011_000 => "bx",
            0b00_100_000 => "sp",
            0b00_101_000 => "bp",
            0b00_110_000 => "si",
            0b00_111_000 => "di",
            _ => panic!("Unknown register"),
        };
    } else {
        return match result {
            0b00_000_000 => "al",
            0b00_001_000 => "cl",
            0b00_010_000 => "dl",
            0b00_011_000 => "bl",
            0b00_100_000 => "ah",
            0b00_101_000 => "ch",
            0b00_110_000 => "dh",
            0b00_111_000 => "bh",
            _ => panic!("Unknown register"),
        };
    }
}
fn get_rm_register(byte: u8, is_word_size: bool) -> &'static str {
    const REGISTER_MASK: u8 = 0b00_000_111;
    let result = byte & REGISTER_MASK;
    if is_word_size {
        return match result {
            0b00_000_000 => "ax",
            0b00_000_001 => "cx",
            0b00_000_010 => "dx",
            0b00_000_011 => "bx",
            0b00_000_100 => "sp",
            0b00_000_101 => "bp",
            0b00_000_110 => "si",
            0b00_000_111 => "di",
            _ => panic!("Unknown register"),
        };
    } else {
        return match result {
            0b00_000_000 => "al",
            0b00_000_001 => "cl",
            0b00_000_010 => "dl",
            0b00_000_011 => "bl",
            0b00_000_100 => "ah",
            0b00_000_101 => "ch",
            0b00_000_110 => "dh",
            0b00_000_111 => "bh",
            _ => panic!("Unknown register"),
        };
    }
}
fn reg_is_dest(byte: u8) -> bool {
    const DEST_REG_MASK: u8 = 0b000000_10; // This is the D bit specified after the instruction operand. It's responsible for specifying the destination and source register.
    return byte & DEST_REG_MASK != 0;
}
fn is_word_size(byte: u8) -> bool {
    const IS_WORD_SIZE_MASK: u8 = 0b000000_01; // This is the W bit and it's responsible for determining the size of the registers (8 or 16 bit).
    return byte & IS_WORD_SIZE_MASK != 0;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();
    for i in (0..binary_contents.len()).step_by(2) {
        let first = binary_contents[i];
        let second = binary_contents[i + 1];
        let is_word_size = is_word_size(first);

        let reg_register = get_reg_register(second, is_word_size);
        let rm_register = get_rm_register(second, is_word_size);

        if reg_is_dest(first) {
            println!("mov {}, {}", reg_register, rm_register);
        } else {
            println!("mov {}, {}", rm_register, reg_register);
        }
    }
}

//           DW_MOD_reg_r/m
// 0b_100010_00_11_000_000
// D = 0 = Instruction source is specified in the REG field
// W = 0 = Instruction operate on byte data
