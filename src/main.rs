use core::panic;
use std::{env, fs};

fn get_reg_register(byte: u8, is_word_size: bool) -> &'static str {
    const REGISTER_TO_REGISTER_MASK: u8 = 0b00_111_000;
    let result = byte & REGISTER_TO_REGISTER_MASK;
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

fn get_rm_register(byte: u8, is_word_size: bool, op: Operation) -> &'static str {
    const RM_MASK: u8 = 0b00_000_111; // this is used to get the contents of the R/M field
    let result = byte & RM_MASK;
    match op {
        Operation::REGISTER_MODE => {
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
        Operation::MEMORY_MODE_NONE | Operation::MEMORY_MODE_8 | Operation::MEMORY_MODE_16 => {
            match result {
                0b00_000_000 => "bx + si",
                0b00_000_001 => "bx + di",
                0b00_000_010 => "bp + si",
                0b00_000_011 => "bp + di",
                0b00_000_100 => "si",
                0b00_000_101 => "di",
                0b00_000_110 => "bp",
                0b00_000_111 => "bx",
                _ => panic!("unknwon instruction detected"),
            }
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    REGISTER_MODE,    // no displacement
    MEMORY_MODE_8,    // 8 bit displacement
    MEMORY_MODE_16,   // 16 bit displacement
    MEMORY_MODE_NONE, // No displacement expect if R/M is 110, then it's 16 bit direct address.
}

fn get_mod_operation(second_byte: u8) -> Operation {
    const MOD_MASK: u8 = 0b_11_000_000;
    return match second_byte & MOD_MASK {
        0b_11_000_000 => Operation::REGISTER_MODE,
        0b_01_000_000 => Operation::MEMORY_MODE_8,
        0b_10_000_000 => Operation::MEMORY_MODE_16,
        0b_00_000_000 => {
            const RM_MASK: u8 = 0b_00_000_111; // we are masking the R/M bits here because (MOD = 00 + R/M 110) = 16 bit displacement.
            let res = second_byte & RM_MASK;
            if res == 0b_00_000_110 {
                Operation::MEMORY_MODE_16
            } else {
                Operation::MEMORY_MODE_NONE
            }
        }
        _ => panic!("Unknown operation - get_mod_operation line: {}", line!()),
    };
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

    let mut i: usize = 0;
    while i < binary_contents.len() {
        let first = binary_contents[i];
        let second = binary_contents[i + 1];

        let reg_is_dest = reg_is_dest(first);
        let op = get_mod_operation(second);

        let is_word_size = is_word_size(first);

        let reg_register = get_reg_register(second, is_word_size);
        let rm_register = get_rm_register(second, is_word_size, op);

        let disp: Option<usize> = match op {
            Operation::MEMORY_MODE_8 => {
                let displacement = binary_contents[i + 2];
                i += 1; // adding one to not go off course in the loop.
                Some(displacement as usize)
            }
            Operation::MEMORY_MODE_16 => {
                let first_byte = binary_contents[i + 2];
                let second_byte = binary_contents[i + 3];
                let combined: u16 = ((first_byte as u16) << 8) | (second_byte as u16);

                i += 2; // adding two to not go off course in the loop.

                Some(combined as usize)
            }
            Operation::REGISTER_MODE | Operation::MEMORY_MODE_NONE => None,
        };

        if reg_is_dest {
            match disp {
                Some(disp) => println!("mov {}, [{} + {}]", reg_register, rm_register, disp),
                None => println!("mov {}, [{}]", reg_register, rm_register),
            }
        } else {
            match disp {
                Some(disp) => println!("mov [{} + {}], {}", rm_register, disp, reg_register),
                None => println!("mov [{}], {}", rm_register, reg_register),
            }
        }
        i += 2; // each iteration is 8 bits, a instruction is minimum 16 bits.
    }
}
//   mov   _DW_MOD_REG_R/M
//0b_100010_10_11_010_010
