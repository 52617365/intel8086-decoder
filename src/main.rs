use core::panic;
use std::{env, fs};

// All the different instruction operations we're looking to handle at the moment.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Operation {
    REGISTER_MODE,            // no displacement
    MEMORY_MODE_8,            // 8 bit displacement
    MEMORY_MODE_16,           // 16 bit displacement
    MEMORY_MODE_NONE, // No displacement expect if R/M is 110, then it's 16 bit direct address.
    MEMORY_MODE_DIRECT, // This is mod 00 with r/m 110 16 bit displacement into a direct memory address
    IMMEDIATE_TO_REGISTER_8, // The first byte is set to 10110... and the instruction is 2 bytes wide. (last byte is the immediate)
    IMMEDIATE_TO_REGISTER_16, // The first byte is set to 10111... and the instruction is 3 bytes wide. (last 2 bytes is the immediate)
}

fn get_reg_register(
    first_byte: u8,
    second_byte: u8,
    is_word_size: bool,
    op: Operation,
) -> &'static str {
    if op == Operation::IMMEDIATE_TO_REGISTER_16 || op == Operation::IMMEDIATE_TO_REGISTER_8 {
        // Immediate to register reg field is in the first byte. Normally it's in the second byte.
        // This is why we have to have separate logic for this case.
        const IMMEDIATE_REG_MASK: u8 = 0b_00_000_111;

        let mask_result = first_byte & IMMEDIATE_REG_MASK;

        return match (is_word_size, mask_result) {
            (true, 0b_00_000_000) => "ax",
            (true, 0b_00_000_001) => "cx",
            (true, 0b_00_000_010) => "dx",
            (true, 0b_00_000_011) => "bx",
            (true, 0b_00_000_100) => "sp",
            (true, 0b_00_000_101) => "bp",
            (true, 0b_00_000_110) => "si",
            (true, 0b_00_000_111) => "di",
            //
            (false, 0b_00_000_000) => "al",
            (false, 0b_00_000_001) => "cl",
            (false, 0b_00_000_010) => "dl",
            (false, 0b_00_000_011) => "bl",
            (false, 0b_00_000_100) => "ah",
            (false, 0b_00_000_101) => "ch",
            (false, 0b_00_000_110) => "dh",
            (false, 0b_00_000_111) => "bh",
            _ => panic!("Unknown register"),
        };
    } else {
        const REGISTER_MEMORY_REG_MASK: u8 = 0b00_111_000; // this is only used for register to register / memory to register and vica verca operations.
        let result = second_byte & REGISTER_MEMORY_REG_MASK;
        return match (is_word_size, result) {
            (true, 0b00_000_000) => "ax",
            (true, 0b00_001_000) => "cx",
            (true, 0b00_010_000) => "dx",
            (true, 0b00_011_000) => "bx",
            (true, 0b00_100_000) => "sp",
            (true, 0b00_101_000) => "bp",
            (true, 0b00_110_000) => "si",
            (true, 0b00_111_000) => "di",
            //
            (false, 0b00_000_000) => "al",
            (false, 0b00_001_000) => "cl",
            (false, 0b00_010_000) => "dl",
            (false, 0b00_011_000) => "bl",
            (false, 0b00_100_000) => "ah",
            (false, 0b00_101_000) => "ch",
            (false, 0b00_110_000) => "dh",
            (false, 0b00_111_000) => "bh",
            _ => panic!("Unknown register"),
        };
    }
}

fn get_rm_register(byte: u8, is_word_size: bool, op: Operation) -> &'static str {
    const RM_MASK: u8 = 0b00_000_111; // this is used to get the contents of the R/M field
    let result = byte & RM_MASK;
    if op == Operation::REGISTER_MODE {
        return match (is_word_size, result) {
            (true, 0b00_000_000) => "ax",
            (true, 0b00_000_001) => "cx",
            (true, 0b00_000_010) => "dx",
            (true, 0b00_000_011) => "bx",
            (true, 0b00_000_100) => "sp",
            (true, 0b00_000_101) => "bp",
            (true, 0b00_000_110) => "si",
            (true, 0b00_000_111) => "di",
            //
            (false, 0b00_000_000) => "al",
            (false, 0b00_000_001) => "cl",
            (false, 0b00_000_010) => "dl",
            (false, 0b00_000_011) => "bl",
            (false, 0b00_000_100) => "ah",
            (false, 0b00_000_101) => "ch",
            (false, 0b00_000_110) => "dh",
            (false, 0b00_000_111) => "bh",
            _ => panic!("Unknown register"),
        };
    } else {
        match result {
            0b00_000_000 => "bx + si",
            0b00_000_001 => "bx + di",
            0b00_000_010 => "bp + si",
            0b00_000_011 => "bp + di",
            0b00_000_100 => "si",
            0b00_000_101 => "di",
            0b00_000_110 => {
                if op != Operation::MEMORY_MODE_DIRECT {
                    "bp"
                } else {
                    "" // direct address instead of a register.
                }
            }

            0b00_000_111 => "bx",
            _ => panic!("unknwon instruction detected"),
        }
    }
}

// In this function we have to check both the first byte and second byte because the first byte determines the contents of the second byte.
fn get_operation(first_byte: u8, second_byte: u8) -> Operation {
    const IMMEDIATE_TO_REGISTER_MASK: u8 = 0b_11111000;
    const MOD_MASK: u8 = 0b_11_000_000;

    return match (
        first_byte & IMMEDIATE_TO_REGISTER_MASK,
        second_byte & MOD_MASK,
    ) {
        (0b_1011_1000, _) => Operation::IMMEDIATE_TO_REGISTER_16, // 16 bit immediate to register because first byte is different from others and w bit is set to 1.
        (0b_1011_0000, _) => Operation::IMMEDIATE_TO_REGISTER_8, // 8 bit immediate to register because first byte is different from others and w bit is set to 0.
        (_, 0b_11_000_000) => Operation::REGISTER_MODE,
        (_, 0b_01_000_000) => Operation::MEMORY_MODE_8,
        (_, 0b_10_000_000) => Operation::MEMORY_MODE_16,
        (_, 0b_00_000_000) => {
            const RM_MASK: u8 = 0b_00_000_111; // we are masking the R/M bits here because (MOD = 00 + R/M 110) = 16 bit displacement.
            let res = second_byte & RM_MASK;
            if res == 0b_00_000_110 {
                Operation::MEMORY_MODE_DIRECT
            } else {
                Operation::MEMORY_MODE_NONE
            }
        }
        _ => panic!("Unknown operation - get_operation line: {}", line!()),
    };
}
fn reg_is_dest(byte: u8) -> bool {
    const DEST_REG_MASK: u8 = 0b000000_10; // This is the D bit specified after the instruction operand. It's responsible for specifying the destination and source register.
    return byte & DEST_REG_MASK != 0;
}
fn is_word_size(byte: u8, op: Operation) -> bool {
    if op == Operation::IMMEDIATE_TO_REGISTER_16 || op == Operation::IMMEDIATE_TO_REGISTER_8 {
        const IMMEDIATE_TO_REGISTER_W_MASK: u8 = 0b0000_1_000; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        return byte & IMMEDIATE_TO_REGISTER_W_MASK != 0;
    } else {
        const IS_WORD_SIZE_MASK: u8 = 0b000000_01; // This is the W bit of a memory to register, register to memory and register to register move and it's responsible for determining the size of the registers (8 or 16 bit).
        return byte & IS_WORD_SIZE_MASK != 0;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let binary_contents = fs::read(binary_path).unwrap();

    let mut i: usize = 0;
    while i < binary_contents.len() {
        let first_byte = binary_contents[i];
        let second_byte = binary_contents[i + 1];

        let op = get_operation(first_byte, second_byte);

        let reg_is_dest = reg_is_dest(first_byte);
        let is_word_size = is_word_size(first_byte, op);

        let reg_register = get_reg_register(first_byte, second_byte, is_word_size, op);
        let rm_register = get_rm_register(second_byte, is_word_size, op);

        let mut disp: Option<usize> = match op {
            Operation::MEMORY_MODE_8 => {
                let displacement = binary_contents[i + 2];
                i += 1; // adding one to not go off course in the loop.
                Some(displacement as usize)
            }
            Operation::MEMORY_MODE_16 | Operation::MEMORY_MODE_DIRECT => {
                let third_byte = binary_contents[i + 2];
                let fourth_byte = binary_contents[i + 3];
                let combined_bytes: u16 = ((fourth_byte as u16) << 8) | (third_byte as u16);

                i += 2; // adding two to not go off course in the loop. Because we went forward 2x with the third and fourth_byte index.

                Some(combined_bytes as usize)
            }
            Operation::IMMEDIATE_TO_REGISTER_16 => {
                let third_byte = binary_contents[i + 2];
                let combined_bytes: u16 = ((third_byte as u16) << 8) | (second_byte as u16);

                i += 1; // adding one to not go off course in the loop. Because we went forward with the third_byte index.

                Some(combined_bytes as usize)
            }
            Operation::IMMEDIATE_TO_REGISTER_8 => Some(second_byte as usize),
            Operation::REGISTER_MODE | Operation::MEMORY_MODE_NONE => None,
        };

        // Handling the case where for example there is a displacement like mov [bp + 0], ch which is an useless displacement.
        if disp == Some(0) {
            disp = None;
        }

        // When dealing immediate to register instructions, reg is always on the lefthand side so we don't have to check for it.
        // We are also unwrapping disp because we have covered the cases on the previous branch and are sure that it contains a value.
        if op == Operation::IMMEDIATE_TO_REGISTER_8 || op == Operation::IMMEDIATE_TO_REGISTER_16 {
            println!(
                "mov {}, {}",
                reg_register,
                disp.expect(
                    "unwrapped disp because we thought we were sure it had a value inside."
                )
            );
        } else {
            match (reg_is_dest, disp) {
                (true, Some(disp)) => {
                    if op == Operation::MEMORY_MODE_DIRECT {
                        println!("mov {}, [{}]", reg_register, disp);
                    } else {
                        println!("mov {}, [{} + {}]", reg_register, rm_register, disp);
                    }
                }
                (false, Some(disp)) => {
                    println!("mov [{} + {}], {}", rm_register, disp, reg_register);
                }
                (true, None) => {
                    if op == Operation::REGISTER_MODE {
                        println!("mov {}, {}", reg_register, rm_register);
                    } else {
                        println!("mov {}, [{}]", reg_register, rm_register);
                    }
                }
                (false, None) => {
                    if op == Operation::REGISTER_MODE {
                        println!("mov {}, {}", rm_register, reg_register);
                    } else {
                        println!("mov [{}], {}", rm_register, reg_register);
                    }
                }
            }
        }
        i += 2; // each iteration is 1 byte, a instruction is minimum 2 bytes.
    }
}
//   mov   _DW_MOD_REG_R/M
//0b_100010_10_11_010_010
